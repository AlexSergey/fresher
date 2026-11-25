//! The size a scan will most likely walk, taken once at its start: what every `ScanProgress` of the scan carries.

use std::path::{Path, PathBuf};

use fresher_core::{ByteSize, DiskState};

use crate::storage::roots::RootVolumes;

/// The estimate of a scan of `roots` (absolute, `~` filled in), in the measure of the walk's progress bytes (logical):
/// when every root that no other root holds is a root of `state`, the last committed state, the sum of their logical
/// sizes; otherwise the sum of the used space of the distinct volumes holding the roots, when every one of them is
/// known; otherwise `None`.
pub(crate) fn expected(state: Option<&dyn DiskState>, roots: &[PathBuf], volumes: &RootVolumes) -> Option<ByteSize> {
    state.and_then(|state| by_state(state, roots)).or_else(|| by_volumes(roots, volumes))
}

/// The logical size of the roots in `state`; `None` when a root is not one of its roots or there are no roots.
fn by_state(state: &dyn DiskState, roots: &[PathBuf]) -> Option<ByteSize> {
    let known: Vec<(PathBuf, ByteSize)> =
        state.roots().into_iter().filter_map(|id| Some((state.path(id)?, state.node(id)?.sizes.logical))).collect();
    let mut total = 0_u64;
    let mut any = false;
    for root in outermost(roots) {
        let (_, size) = known.iter().find(|(path, _)| path == root)?;
        total = total.saturating_add(size.bytes());
        any = true;
    }
    any.then(|| ByteSize::new(total))
}

/// The used space of the volumes holding `roots`; `None` when one of them is not known or there are no roots.
fn by_volumes(roots: &[PathBuf], volumes: &RootVolumes) -> Option<ByteSize> {
    let groups = volumes.group(roots);
    if groups.is_empty() {
        return None;
    }
    groups
        .iter()
        .try_fold(0_u64, |total, group| group.space.map(|space| total.saturating_add(space.used.bytes())))
        .map(ByteSize::new)
}

/// The roots no other root holds, once each, as the walk scans them.
fn outermost(roots: &[PathBuf]) -> impl Iterator<Item = &Path> {
    roots.iter().enumerate().filter_map(|(position, root)| {
        let covered = roots
            .iter()
            .enumerate()
            .any(|(other, outer)| other != position && root.starts_with(outer) && (outer != root || other < position));
        (!covered).then_some(root.as_path())
    })
}

#[cfg(test)]
#[path = "estimate_tests.rs"]
mod tests;
