//! The rescan: the cached tree is loaded and checked against the scan asked for, the change journal names the folders
//! that changed since the cached position, only those are read again, and any doubt means a full scan.

pub(crate) mod change_set;
pub(crate) mod check;
pub(crate) mod emit;
pub(crate) mod frozen;
pub(crate) mod load;
pub(crate) mod patch;
pub(crate) mod summary;

use std::path::PathBuf;

use fresher_core::{
    CancelToken, ChangeJournal, FolderIdentity, FullScanReason, InterestSet, JournalGap, Match, Pattern, ScanCache,
    ScanKind, ScopeKey, Timestamp, VolumeId, WalkError, WalkProgress, WalkSummary,
};

use crate::rescan::change_set::{ChangeSet, Gather};
use crate::rescan::load::{Cached, Use};
use crate::tree::arena::Tree;
use crate::walk::Walk;

/// What a scan is of, as the cache must match it: the scope key, the canonical roots with their volumes and
/// identities, the avoid list and the interests' fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Scope {
    pub(crate) key: ScopeKey,
    pub(crate) roots: Vec<ScopeRoot>,
    /// Every place the walk does not open, sorted.
    pub(crate) avoid: Vec<PathBuf>,
    pub(crate) fingerprint: u64,
}

/// One canonical root of a scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScopeRoot {
    pub(crate) path: PathBuf,
    pub(crate) volume: VolumeId,
    pub(crate) identity: FolderIdentity,
}

/// Why a rescan did not give a tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Stop {
    /// A full scan is needed instead.
    Full(FullScanReason),
    /// The walk itself stopped (cancelled, out of indices): no full scan either.
    Walk(WalkError),
}

impl From<WalkError> for Stop {
    fn from(error: WalkError) -> Self {
        Self::Walk(error)
    }
}

/// What a rescan reads from.
#[derive(Clone, Copy)]
pub(crate) struct Sources<'a> {
    pub(crate) cache: &'a dyn ScanCache,
    pub(crate) journal: &'a dyn ChangeJournal,
    pub(crate) now: Timestamp,
    /// Draws the check's sample.
    pub(crate) seed: u64,
    /// The most folders a rescan reads again, in percent of the tree's folders.
    pub(crate) limit_percent: u64,
}

/// Rescans `scope` through `walk`: the cached tree patched with the journal's changes, its matches emitted, the summary
/// rebuilt from it.
pub(crate) fn rescan(
    scope: &Scope,
    walk: &Walk<'_>,
    sources: Sources<'_>,
    cancel: &CancelToken,
    on_match: &mut dyn FnMut(Match),
    on_progress: &mut dyn FnMut(WalkProgress),
) -> Result<(Tree, WalkSummary), Stop> {
    let cached = Cached::load(sources.cache, scope, Use::Rescan).map_err(Stop::Full)?;
    let gather = Gather {
        journal: sources.journal,
        now: sources.now,
        sibling_markers: sibling_markers(walk.interests),
        limit_percent: sources.limit_percent,
        cancel,
    };
    let set = match ChangeSet::gather(&cached, gather) {
        Ok(set) => set,
        Err(FullScanReason::JournalGap(JournalGap::Cancelled)) if cancel.is_cancelled() => {
            return Err(Stop::Walk(WalkError::Cancelled));
        }
        Err(reason) => return Err(Stop::Full(reason)),
    };
    let changed = u32::try_from(set.reread.len() + set.rewalk.len()).unwrap_or(u32::MAX);
    let patched = patch::patch(&cached.tree, &set, walk, cancel, on_progress)?;
    check::check(&patched.tree, &patched.unchanged, walk, sources.seed, cancel)?;
    emit::emit(&patched.tree, walk.matcher, walk.interests, on_match);
    let summary = summary::summarize(&patched.tree, ScanKind::Rescan { changed_folders: changed });
    Ok((patched.tree, summary))
}

/// Whether some marker interest has sibling names.
fn sibling_markers(interests: &InterestSet) -> bool {
    interests
        .iter()
        .any(|(_, interest)| matches!(interest.pattern(), Pattern::MarkerFile(_)) && !interest.siblings().is_empty())
}

#[cfg(test)]
pub(crate) mod testing;
#[cfg(test)]
mod tests;
