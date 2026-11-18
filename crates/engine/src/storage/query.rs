//! The engine's queries of the storage map, in its three phases: before any scan, while one runs, and of the last
//! committed state. Each takes what it reads under the results' lock and lets the lock go before it builds: none blocks
//! a scan, and the volumes are never read here but before the first scan. The by-meaning view and the places answer for
//! the map of the phase the map query answers.

use std::path::{Path, PathBuf};

use fresher_core::{
    Access, AccessStatus, Capability, CommandError, Descent, FindingsQuery, MapPhase, MapPlace, MeaningGroup, NodeId,
    NotScanned, Operation, PlaceView, Sizes, StorageMap, Text, Unreadable, UnreadablePlace, VolumeMap,
};

use crate::Engine;
use crate::error::{EngineError, lock};
use crate::results::Committed;
use crate::storage::meaning::{append_volume_parts, meaning};
use crate::storage::place::place_view;
use crate::storage::roots::{RootVolumes, expand_home};
use crate::storage::volume::{RootOnVolume, VolumeInput, volume_map};

/// The most rows one place page holds, like every disk query.
const MAX_LIMIT: u32 = FindingsQuery::MAX_LIMIT;

/// What a map query read under the results' lock.
enum Snapshot {
    NoScan,
    Scanning { roots: Vec<PathBuf>, explained: Vec<Sizes>, volumes: RootVolumes },
    Complete(Committed),
}

impl Engine {
    /// The map of every volume that holds a scan root, the home's first: before any scan, the configured roots'
    /// volumes, all "Not scanned yet"; during a scan, the findings so far as "Explained"; after a commit, the full map
    /// of the last state, with its roots' volumes as read at its commit.
    pub fn storage_map(&self) -> Result<StorageMap, CommandError> {
        let (phase, volumes) = match self.snapshot()? {
            Snapshot::NoScan => (MapPhase::NoScan, self.before_any_scan()?),
            Snapshot::Scanning { roots, explained, volumes } => {
                (MapPhase::Scanning, unscanned(&volumes, &roots, &explained, MapPhase::Scanning)?)
            }
            Snapshot::Complete(committed) => {
                let phase = MapPhase::Complete { state: committed.state.id() };
                let maps = complete(&committed, &self.shared.access.status())?;
                (phase, maps.into_iter().map(|(map, _)| map).collect())
            }
        };
        Ok(StorageMap { phase, volumes })
    }

    /// The by-meaning groups of volume `volume` of the map `storage_map` answers now. After a commit: the apps, the
    /// categories and the user's folders of that volume, ending with its parts that are not files of the scan. During
    /// a scan there is no tree to group yet: the volume's parts as the scanning map has them, in its order. `NoScan`
    /// before any scan, `UnknownVolume` for an index past the map's volumes.
    pub fn storage_meaning(&self, volume: usize) -> Result<Vec<MeaningGroup>, CommandError> {
        match self.snapshot()? {
            Snapshot::NoScan => Err(CommandError::NoScan),
            Snapshot::Scanning { roots, explained, volumes } => {
                let maps = unscanned(&volumes, &roots, &explained, MapPhase::Scanning)?;
                let map = maps.into_iter().nth(volume).ok_or(CommandError::UnknownVolume)?;
                Ok(map.parts().iter().cloned().map(MeaningGroup::Part).collect())
            }
            Snapshot::Complete(committed) => {
                let maps = complete(&committed, &self.shared.access.status())?;
                let (map, roots) = maps.into_iter().nth(volume).ok_or(CommandError::UnknownVolume)?;
                let state = &*committed.state;
                let mut groups = meaning(state, committed.shares(), &committed.places, &committed.findings, &roots);
                append_volume_parts(&mut groups, &map);
                Ok(groups)
            }
        }
    }

    /// One page of the place `id` of the last state, largest first, opened through the folders that are almost all
    /// one child; at most 500 rows. Only the complete map has places: `Busy` while a scan runs (its map has none
    /// yet), `NoScan` before any commit, `StaleNode` for a node of another state.
    pub fn storage_place(
        &self,
        id: NodeId,
        descent: Descent,
        offset: u32,
        limit: u32,
    ) -> Result<PlaceView, CommandError> {
        let committed = match self.snapshot()? {
            Snapshot::NoScan => return Err(CommandError::NoScan),
            Snapshot::Scanning { .. } => return Err(CommandError::Busy { running: Operation::Scan }),
            Snapshot::Complete(committed) => committed,
        };
        if id.state() != committed.state.id() {
            return Err(CommandError::StaleNode);
        }
        place_view(&*committed.state, committed.shares(), id, descent, offset, limit.min(MAX_LIMIT))
            .ok_or(CommandError::StaleNode)
    }

    /// The phase and what it reads, taken under the results' lock: the home is located before, so no port is asked
    /// under it.
    fn snapshot(&self) -> Result<Snapshot, CommandError> {
        let home = self.shared.home();
        let results = lock(&self.shared.results)?;
        if results.is_scanning() {
            let roots = expand_home(&results.walked_roots(), home.as_deref());
            let explained = results.explained_under(&roots);
            return Ok(Snapshot::Scanning { roots, explained, volumes: results.volumes().clone() });
        }
        Ok(results.committed().map_or(Snapshot::NoScan, Snapshot::Complete))
    }

    /// The maps before any scan: the configured roots, `~` filled in, on their volumes, read now when they are not
    /// of these roots and kept until a scan reads them again.
    fn before_any_scan(&self) -> Result<Vec<VolumeMap>, CommandError> {
        let home = self.shared.home();
        let configured = lock(&self.shared.settings)?.scan_roots();
        let roots = expand_home(&configured, home.as_deref());
        let known = {
            let results = lock(&self.shared.results)?;
            (results.volumes().asked() == roots).then(|| results.volumes().clone())
        };
        let volumes = if let Some(volumes) = known {
            volumes
        } else {
            let volumes = self.shared.read_volumes(&roots);
            let mut results = lock(&self.shared.results)?;
            if !results.is_scanning() && results.committed().is_none() {
                results.keep_volumes(volumes.clone());
            }
            volumes
        };
        unscanned(&volumes, &roots, &[], MapPhase::NoScan)
    }
}

/// The maps of `roots` before a commit: no tree yet, so each volume's used space is "Not scanned yet" but for what
/// `explained` (by root, empty for none) holds.
fn unscanned(
    volumes: &RootVolumes,
    roots: &[PathBuf],
    explained: &[Sizes],
    phase: MapPhase,
) -> Result<Vec<VolumeMap>, CommandError> {
    volumes
        .group(roots)
        .into_iter()
        .map(|group| {
            let roots = group
                .roots
                .iter()
                .filter_map(|&position| {
                    Some(RootOnVolume {
                        path: roots.get(position)?.clone(),
                        node: None,
                        sizes: Sizes::ZERO,
                        explained: explained.get(position).copied().unwrap_or(Sizes::ZERO),
                    })
                })
                .collect();
            let input = VolumeInput { space: group.space.cloned(), roots, unreadable_folders: 0, needs: None, phase };
            volume_map(input).map_err(|error| CommandError::from(EngineError::StorageMap(error)))
        })
        .collect()
}

/// The full maps of the committed state, each with the roots it holds: the tree's sizes, the shares, and the folders
/// on the volume the scan could not read.
fn complete(committed: &Committed, access: &AccessStatus) -> Result<Vec<(VolumeMap, Vec<NodeId>)>, CommandError> {
    let state = &*committed.state;
    let shares = committed.shares();
    let roots: Vec<(NodeId, PathBuf)> =
        state.roots().into_iter().filter_map(|root| Some((root, state.path(root)?))).collect();
    let paths: Vec<PathBuf> = roots.iter().map(|(_, path)| path.clone()).collect();
    let unreadable: Vec<(PathBuf, UnreadablePlace)> =
        state.unreadable(u32::MAX).into_iter().filter_map(|place| Some((state.path(place.node.id)?, place))).collect();
    let phase = MapPhase::Complete { state: state.id() };
    let mut maps = Vec::new();
    for group in committed.volumes.group(&paths) {
        let on_volume: Vec<&(NodeId, PathBuf)> =
            group.roots.iter().filter_map(|&position| roots.get(position)).collect();
        let under = |path: &Path| on_volume.iter().any(|(_, root)| path.starts_with(root));
        let places: Vec<&UnreadablePlace> =
            unreadable.iter().filter(|(path, _)| under(path)).map(|(_, place)| place).collect();
        let (unreadable_folders, needs) = unread(&places, access);
        let roots_on_volume = on_volume
            .iter()
            .map(|&&(root, ref path)| RootOnVolume {
                path: path.clone(),
                node: Some(root),
                sizes: state.node(root).map_or(Sizes::ZERO, |node| node.sizes),
                explained: shares
                    .roots()
                    .iter()
                    .find(|share| share.root == root)
                    .map_or(Sizes::ZERO, |share| share.explained),
            })
            .collect();
        let input =
            VolumeInput { space: group.space.cloned(), roots: roots_on_volume, unreadable_folders, needs, phase };
        let map = volume_map(input).map_err(|error| CommandError::from(EngineError::StorageMap(error)))?;
        let named = on_volume.iter().filter_map(|&&(root, ref path)| {
            let place = committed.walked.iter().find(|place| place.path == *path)?;
            Some(MapPlace { root, name: Text::new(place.name_key()) })
        });
        let map = map.with_places(named.collect());
        maps.push((map, on_volume.iter().map(|(root, _)| *root).collect()));
    }
    Ok(maps)
}

/// How many folders `places` hold that the scan did not read (each place and the folders hidden under it; a place on
/// another volume is not unread), and the permission that would show them: every file, when one is guarded and that
/// is not granted. The system's own items are unread too, but no permission shows them.
fn unread(places: &[&UnreadablePlace], access: &AccessStatus) -> (u64, Option<Capability>) {
    let unread = places.iter().filter(|place| place.reason != NotScanned::OtherVolume);
    let folders =
        unread.clone().fold(0_u64, |total, place| total.saturating_add(place.hidden_folders).saturating_add(1));
    let guarded = unread
        .clone()
        .any(|place| matches!(place.reason, NotScanned::Unreadable(Unreadable::Guarded) | NotScanned::Avoided));
    let shown = matches!(access.access(&Capability::AllFiles), Access::Granted | Access::NotApplicable);
    let needs = (guarded && !shown).then_some(Capability::AllFiles);
    (folders, needs)
}

#[cfg(test)]
#[path = "query_tests.rs"]
mod tests;
