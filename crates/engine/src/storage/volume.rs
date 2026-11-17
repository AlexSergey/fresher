//! A volume's map: the scanned parts, the remainder with its reasons, the neighbouring volumes, the disk's own space
//! and Free, adding up to the space the volume shares. Pure: the volume's facts and the roots' sizes come in.

use std::path::PathBuf;

use fresher_core::{
    ByteSize, Capability, DiskSpace, MapPart, MapPhase, MountKind, NeighbourRole, NodeId, PartKind, RemainderReason,
    Sizes, StorageMapError, VolumeMap,
};

use crate::claims::minus;
use crate::results::add;

/// What one volume's map is built from.
#[derive(Clone, Debug)]
pub(crate) struct VolumeInput {
    /// The volume's facts; `None` without a `Volumes` port or when it failed.
    pub(crate) space: Option<DiskSpace>,
    /// The scan roots on the volume.
    pub(crate) roots: Vec<RootOnVolume>,
    /// The folders on the volume the scan could not read.
    pub(crate) unreadable_folders: u64,
    /// The permission that would show them, from the access status.
    pub(crate) needs: Option<Capability>,
    pub(crate) phase: MapPhase,
}

/// One scan root on the volume.
#[derive(Clone, Debug)]
pub(crate) struct RootOnVolume {
    pub(crate) path: PathBuf,
    /// The root's node; `None` before a commit.
    pub(crate) node: Option<NodeId>,
    /// The tree's sizes of the root; zero before a commit.
    pub(crate) sizes: Sizes,
    /// What the counted owners explain under it (`Shares`), or the findings so far during a scan.
    pub(crate) explained: Sizes,
}

/// The map of one volume. Its parts come in the order of [`PartKind`], parts of zero size kept, and add up to its
/// capacity: when the volume reports less than the parts hold, the capacity is taken as their sum. An error only when a
/// sum does not fit in a size.
pub(crate) fn volume_map(input: VolumeInput) -> Result<VolumeMap, StorageMapError> {
    let VolumeInput { space, roots, unreadable_folders, needs, phase } = input;
    let explained = roots.iter().fold(Sizes::ZERO, |total, root| add(total, root.explained));
    let nodes: Vec<NodeId> = roots.iter().filter_map(|root| root.node).collect();
    let complete = matches!(phase, MapPhase::Complete { .. });

    let Some(space) = space else {
        let mut parts = vec![MapPart { kind: PartKind::Explained, sizes: explained }];
        if complete {
            parts.push(MapPart { kind: PartKind::YourFolders, sizes: your_folders(&roots) });
        } else {
            parts.push(MapPart { kind: PartKind::NotScannedYet, sizes: Sizes::ZERO });
        }
        let capacity = sum(parts.iter().map(|part| part.sizes.allocated))?;
        return VolumeMap::new(root_name(&roots), MountKind::Internal, capacity, parts, None, nodes, false);
    };

    let mut parts = vec![MapPart { kind: PartKind::Explained, sizes: explained }];
    let used = if complete {
        let your_folders = your_folders(&roots);
        let scanned = sum([explained.allocated, your_folders.allocated])?;
        parts.push(MapPart { kind: PartKind::YourFolders, sizes: your_folders });
        let outside_scan = !roots.iter().any(|root| root.path.components().eq(space.mount_point.components()));
        let reasons = reasons(unreadable_folders, needs);
        let remainder = space.used.bytes().saturating_sub(scanned.bytes());
        parts.push(MapPart { kind: PartKind::Remainder { outside_scan, reasons }, sizes: volume_part(remainder) });
        scanned.max(space.used)
    } else {
        let not_scanned = space.used.bytes().saturating_sub(explained.allocated.bytes());
        parts.push(MapPart { kind: PartKind::NotScannedYet, sizes: volume_part(not_scanned) });
        explained.allocated.max(space.used)
    };
    // Only a complete scan says the disk changed: during a scan the findings so far are counted at once, before the
    // walk has reached the rest, so they may run past the used space without anything having changed.
    let disk_changed = complete && used > space.used;

    let used_by = |role: NeighbourRole| {
        sum(space.neighbours.iter().filter(|next| next.role == role).map(|next| next.used)).map(ByteSize::bytes)
    };
    parts.push(MapPart { kind: PartKind::MacOs, sizes: volume_part(used_by(NeighbourRole::System)?) });
    parts.push(MapPart { kind: PartKind::SwapAndSleep, sizes: volume_part(used_by(NeighbourRole::VirtualMemory)?) });
    parts.extend(space.neighbours.iter().filter(|next| next.role == NeighbourRole::Other).map(|next| MapPart {
        kind: PartKind::OtherVolume { name: next.name.clone() },
        sizes: volume_part(next.used.bytes()),
    }));

    let taken = sum([used, space.free, sum(space.neighbours.iter().map(|next| next.used))?])?;
    let (capacity, disk_own) = match space.capacity.bytes().checked_sub(taken.bytes()) {
        Some(disk_own) => (space.capacity, disk_own),
        None => (taken, 0),
    };
    parts.push(MapPart { kind: PartKind::DiskOwn, sizes: volume_part(disk_own) });
    parts.push(MapPart { kind: PartKind::Free, sizes: volume_part(space.free.bytes()) });

    VolumeMap::new(space.name, space.kind, capacity, parts, space.purgeable, nodes, disk_changed)
}

/// The roots' bytes no counted owner explains.
fn your_folders(roots: &[RootOnVolume]) -> Sizes {
    roots.iter().fold(Sizes::ZERO, |total, root| add(total, minus(root.sizes, root.explained)))
}

/// The remainder's reasons, in their fixed order: the unread folders when there are any, then the snapshots and the
/// file system's bookkeeping, and the changes during the scan, always.
fn reasons(unreadable_folders: u64, needs: Option<Capability>) -> Vec<RemainderReason> {
    let unreadable =
        (unreadable_folders > 0).then_some(RemainderReason::Unreadable { folders: unreadable_folders, needs });
    unreadable
        .into_iter()
        .chain([RemainderReason::SnapshotsAndBookkeeping, RemainderReason::ChangedDuringScan])
        .collect()
}

/// The name of a map drawn without the volume's facts: the first root's folder name, or its whole path (`/`).
fn root_name(roots: &[RootOnVolume]) -> String {
    roots
        .first()
        .map(|root| match root.path.file_name() {
            Some(name) => name.to_string_lossy().into_owned(),
            None => root.path.display().to_string(),
        })
        .unwrap_or_default()
}

/// A part of the volume that is not files of the scan: removing it is not a choice, so it frees nothing.
fn volume_part(bytes: u64) -> Sizes {
    Sizes::new(ByteSize::new(bytes), ByteSize::new(bytes), ByteSize::ZERO)
}

/// The sum of `sizes`; `Overflow` when it does not fit.
fn sum(sizes: impl IntoIterator<Item = ByteSize>) -> Result<ByteSize, StorageMapError> {
    sizes.into_iter().try_fold(ByteSize::ZERO, ByteSize::checked_add).ok_or(StorageMapError::Overflow)
}

#[cfg(test)]
#[path = "volume_tests.rs"]
mod tests;
