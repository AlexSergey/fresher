//! The volumes that hold the scan roots, with their facts: read at a scan's start, at its commit (the restore's too),
//! and before the first scan for the map with no scan, never per query.

use std::path::{Component, Path, PathBuf};

use fresher_core::{DiskSpace, VolumeId, Volumes};

/// The scan roots as the settings write them, `~` and `~/…` filled in with `home`, as the walk fills them in.
pub(crate) fn expand_home(roots: &[PathBuf], home: Option<&Path>) -> Vec<PathBuf> {
    roots
        .iter()
        .map(|root| {
            let mut parts = root.components();
            match (parts.next(), home) {
                (Some(Component::Normal(first)), Some(home)) if first == "~" => home.join(parts.as_path()),
                _ => root.clone(),
            }
        })
        .collect()
}

/// The volumes of a list of roots, the home's first, then the others in the order of their first root.
#[derive(Clone, Debug, Default)]
pub(crate) struct RootVolumes {
    /// The roots these volumes were read for, in the order asked.
    asked: Vec<PathBuf>,
    volumes: Vec<RootVolume>,
}

/// One volume and the roots it holds.
#[derive(Clone, Debug)]
struct RootVolume {
    roots: Vec<PathBuf>,
    /// `None` when the volume's space could not be read (logged) or there is no `Volumes` port.
    space: Option<DiskSpace>,
}

/// The roots of one volume of a map, as positions in the list they were grouped from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Group<'a> {
    pub(crate) space: Option<&'a DiskSpace>,
    pub(crate) roots: Vec<usize>,
}

impl RootVolumes {
    /// Reads the volumes of `roots` through `volumes`: one `identify` per root and one `space` per volume. A root
    /// whose volume cannot be identified is a volume of its own; without a port every root is, without facts.
    pub(crate) fn read(volumes: Option<&dyn Volumes>, home: Option<&Path>, roots: &[PathBuf]) -> Self {
        let Some(port) = volumes else {
            let volumes = roots.iter().map(|root| RootVolume { roots: vec![root.clone()], space: None }).collect();
            return Self { asked: roots.to_vec(), volumes };
        };
        let identify = |path: &Path| port.identify(path).ok().map(|facts| facts.id);
        let mut found: Vec<(Option<VolumeId>, Vec<PathBuf>)> = Vec::new();
        for root in roots {
            let id = identify(root);
            match found.iter_mut().find(|(known, _)| known.is_some() && *known == id) {
                Some((_, held)) => held.push(root.clone()),
                None => found.push((id, vec![root.clone()])),
            }
        }
        let home = home.and_then(identify);
        found.sort_by_key(|(id, _)| home.is_none() || *id != home);
        let volumes = found
            .into_iter()
            .map(|(_, roots)| {
                let space = roots.first().and_then(|root| match port.space(root) {
                    Ok(space) => Some(space),
                    Err(error) => {
                        tracing::warn!(%error, "the storage map shows this volume without its facts");
                        None
                    }
                });
                RootVolume { roots, space }
            })
            .collect();
        Self { asked: roots.to_vec(), volumes }
    }

    /// The roots these volumes were read for.
    pub(crate) fn asked(&self) -> &[PathBuf] {
        &self.asked
    }

    /// `roots` grouped by the volumes read, in their order; a root not read is a volume of its own, without facts,
    /// after them in the order of `roots`.
    pub(crate) fn group(&self, roots: &[PathBuf]) -> Vec<Group<'_>> {
        let mut groups: Vec<Group<'_>> =
            self.volumes.iter().map(|volume| Group { space: volume.space.as_ref(), roots: Vec::new() }).collect();
        for (position, root) in roots.iter().enumerate() {
            let volume = self.volumes.iter().position(|volume| volume.roots.contains(root));
            match volume.and_then(|volume| groups.get_mut(volume)) {
                Some(group) => group.roots.push(position),
                None => groups.push(Group { space: None, roots: vec![position] }),
            }
        }
        groups.retain(|group| !group.roots.is_empty());
        groups
    }
}

#[cfg(test)]
#[path = "roots_tests.rs"]
mod tests;
