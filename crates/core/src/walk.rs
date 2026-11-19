//! The disk walk: one pass over the scan roots for every module's interests.

use std::collections::BTreeMap;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use ts_rs::TS;

use crate::{
    ByteSize, CancelToken, CategoryId, DiskState, InterestSet, JournalGap, KnownFolder, Match, NotScanned, ScanKind,
    Sizes, TextKey, Timestamp,
};

/// Walks the scan roots once and reports every entry that matches an interest. Never follows a symbolic link and
/// never leaves a root.
pub trait DiskWalk: Send + Sync {
    fn walk(
        &self,
        request: &WalkRequest<'_>,
        cancel: &CancelToken,
        on_match: &mut dyn FnMut(Match),
        on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<WalkSummary, WalkError>;
    /// Records the scan's claims and keeps its state; called only after a walk that was not cancelled.
    fn commit(&self, claims: &[ClaimedPath]) -> Result<Arc<dyn DiskState>, WalkError>;
    /// What the last walk saw below each of `queries`, in their order, from its tree before `commit`: no disk is
    /// read (a restore answers from the cached tree). `None` for a path that is not a folder of that tree (never
    /// walked, inside a folder claimed whole, a link, a file) and for every query before any walk.
    fn subtrees(&self, queries: &[SubtreeQuery]) -> Vec<Option<Subtree>>;
}

/// A folder of the last walk to describe, and the folders below it whose times are left out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubtreeQuery {
    pub folder: PathBuf,
    /// Folders below `folder` left out of `newest` and `complete`, with everything inside them; their sizes count.
    pub skip: Vec<PathBuf>,
}

/// What the last walk saw below one folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Subtree {
    /// The folder and everything inside it, as the walk summed it.
    pub sizes: Sizes,
    /// The newest modification time of the folder and of every folder below it, the skipped ones and everything
    /// inside them left out; `None` when none has one.
    pub newest: Option<Timestamp>,
    /// Every folder counted in `newest` was read in full: none unreadable, avoided, on another volume or collapsed, and
    /// none below `folder` claimed whole (nothing inside it is kept).
    pub complete: bool,
}

/// What one walk is asked to do.
#[derive(Clone, Copy, Debug)]
pub struct WalkRequest<'a> {
    pub roots: &'a [PathBuf],
    pub interests: &'a InterestSet,
    /// Places the walk must not open (they would raise a system prompt); each is reported as not scanned.
    pub avoid: &'a [PathBuf],
    /// Known folders outside the scan roots that a module's interest names: each is walked as a root of its own, for
    /// the interests anchored in it only, never entering a folder of another user. Always read from the disk: a rescan
    /// walks them again in full and a restore emits nothing inside them.
    pub places: &'a [Place],
    pub mode: WalkMode,
}

/// A known folder walked as a root of its own, outside the scan roots.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Place {
    pub folder: KnownFolder,
    /// Its canonical path.
    pub path: PathBuf,
}

impl Place {
    /// Its name as the storage map shows it: `core.place.<folder>.name`.
    pub fn name_key(&self) -> TextKey {
        TextKey::from_valid(format!("core.place.{}.name", self.folder.as_str()))
    }
}

/// How the walk reads the disk.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WalkMode {
    /// Every folder, whatever the cache holds.
    Full,
    /// Only the folders changed since the cached scan when the cache allows it; a full scan otherwise.
    Auto,
    /// The cache only, no disk: the last state at launch.
    Restore,
}

/// A path the scan's findings claim, with the category that owns it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimedPath {
    pub path: PathBuf,
    pub category: CategoryId,
}

/// Why a walk asked for a rescan read every folder instead. Logged by the engine, never shown with a path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FullScanReason {
    NoCache,
    BrokenCache,
    /// The interests compiled into the cache are not today's.
    OtherFingerprint,
    OtherRoots,
    OtherAvoidList,
    /// A root is on another volume than when the cache was written.
    OtherVolume,
    JournalGap(JournalGap),
    TooManyChanges,
    /// The sampled check of unchanged folders found a change the journal did not report.
    CheckFailed,
    AskedByUser,
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
    /// How the disk was read.
    pub kind: ScanKind,
    /// Why a rescan became a full scan; `None` when it did not.
    pub fallback: Option<FullScanReason>,
    /// The folders deliberately not entered (avoided places, other volumes), counted by why.
    pub not_scanned: BTreeMap<NotScanned, u64>,
    /// What the walk saw of each anchored interest's place (index in the interest set; only interests with one
    /// place): not the walked places of the request.
    pub places: BTreeMap<usize, PlaceSeen>,
}

impl WalkSummary {
    /// `true` when every folder was read; a place not entered on purpose (`not_scanned`) does not count.
    pub fn is_complete(&self) -> bool {
        self.unreadable.values().all(|&count| count == 0)
    }
}

/// What the walk saw of the folder an anchored interest matches in (its "place": the pattern's folder without its
/// last segment, `~/Library/Caches` for `Under(Home, "Library/Caches/*")`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlaceSeen {
    /// Listed in full: every entry of its own listing was read (what is inside a matched folder is the match's).
    Read,
    /// Not there: its parent was read and holds no such folder.
    Absent,
    /// It, or a folder on the way to it, could not be read or is claimed whole, or an entry of its listing could not be
    /// read.
    Unread,
    /// Not entered on purpose (avoided, another volume, another user's) or outside every scan root.
    NotScanned,
    /// Not known: during the walk, a restored scan, or an interest without one place.
    Unknown,
}

impl PlaceSeen {
    /// Whether the walk saw the place in full: it was read, or it is not there.
    pub const fn is_complete(self) -> bool {
        matches!(self, Self::Read | Self::Absent)
    }
}

/// Why a folder could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Unreadable {
    /// Its POSIX permissions refuse the app (`EACCES`).
    NoPermission,
    /// The system's privacy protection refuses the app (`EPERM`): on macOS, a place that needs Full Disk Access.
    Guarded,
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
    /// A `Restore` walk found no cache to restore.
    #[error("there is no cached scan to restore")]
    NothingToRestore,
    #[error("the scan could not be committed: {kind}")]
    Commit { kind: io::ErrorKind },
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{PlaceSeen, Unreadable, WalkSummary};

        #[test]
        fn a_summary_with_an_unreadable_folder_is_not_complete() {
            let summary =
                WalkSummary { entries: 3, unreadable: [(Unreadable::Guarded, 1)].into(), ..Default::default() };

            assert!(!summary.is_complete());
        }

        #[test]
        fn a_place_not_seen_in_full_is_not_complete() {
            for seen in [PlaceSeen::Unread, PlaceSeen::NotScanned, PlaceSeen::Unknown] {
                assert!(!seen.is_complete(), "{seen:?}");
            }
        }
    }

    mod positive {
        use crate::{NotScanned, PlaceSeen, ScanKind, Unreadable, WalkSummary};

        #[test]
        fn an_empty_summary_is_a_complete_full_scan() {
            let summary = WalkSummary::default();

            assert!(summary.is_complete());
            assert_eq!((summary.kind, summary.fallback), (ScanKind::Full, None));
        }

        #[test]
        fn a_place_not_entered_on_purpose_leaves_the_summary_complete() {
            let summary = WalkSummary { not_scanned: [(NotScanned::Avoided, 2)].into(), ..WalkSummary::default() };

            assert!(summary.is_complete());
        }

        #[test]
        fn a_place_read_or_absent_is_complete() {
            assert!(PlaceSeen::Read.is_complete());
            assert!(PlaceSeen::Absent.is_complete());
        }

        #[test]
        fn the_places_seen_leave_the_summary_completeness_alone() {
            let unread = [(0, PlaceSeen::Unread)].into();
            let summary = WalkSummary { places: unread, ..WalkSummary::default() };
            let read = [(0, PlaceSeen::Read)].into();
            let guarded =
                WalkSummary { unreadable: [(Unreadable::Guarded, 1)].into(), places: read, ..summary.clone() };

            assert!(summary.is_complete());
            assert!(!guarded.is_complete());
        }
    }
}
