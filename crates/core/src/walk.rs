//! The disk walk: one pass over the scan roots for every module's interests.

use std::collections::BTreeMap;
use std::io;
use std::path::PathBuf;

use crate::{ByteSize, CancelToken, InterestSet, Match};

/// Walks the scan roots once and reports every entry that matches an interest. Never follows a symbolic link and
/// never leaves a root.
pub trait DiskWalk: Send + Sync {
    fn walk(
        &self,
        roots: &[PathBuf],
        interests: &InterestSet,
        cancel: &CancelToken,
        on_match: &mut dyn FnMut(Match),
        on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<WalkSummary, WalkError>;
}

/// How far the walk has got.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WalkProgress {
    pub entries: u64,
    pub bytes: ByteSize,
}

/// What the walk saw, once it is done.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WalkSummary {
    pub entries: u64,
    /// The folders that could not be read, counted by why.
    pub unreadable: BTreeMap<Unreadable, u64>,
}

impl WalkSummary {
    /// `true` when every folder was read.
    pub fn is_complete(&self) -> bool {
        self.unreadable.values().all(|&count| count == 0)
    }
}

/// Why a folder could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Unreadable {
    /// Its permissions refuse the app.
    NoPermission,
    /// macOS protects it and the app has no Full Disk Access.
    NeedsFullDiskAccess,
    /// Any other error (it vanished during the walk, a broken disk).
    Failed,
}

/// Why the walk stopped. No path is held: paths are not logged.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum WalkError {
    #[error("the walk was cancelled")]
    Cancelled,
    #[error("scan root {index} could not be read: {kind}")]
    RootUnreadable { index: usize, kind: io::ErrorKind },
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{Unreadable, WalkSummary};

        #[test]
        fn a_summary_with_an_unreadable_folder_is_not_complete() {
            let summary = WalkSummary { entries: 3, unreadable: [(Unreadable::NeedsFullDiskAccess, 1)].into() };

            assert!(!summary.is_complete());
        }
    }

    mod positive {
        use crate::WalkSummary;

        #[test]
        fn an_empty_summary_is_complete() {
            assert!(WalkSummary::default().is_complete());
        }
    }
}
