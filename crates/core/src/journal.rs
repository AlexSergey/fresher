//! The change journal: which folders of a volume changed since a position, so a rescan reads only those.

use std::path::PathBuf;

use serde::Serialize;

use crate::{CancelToken, VolumeId};

/// The system's record of the folders that changed on a volume.
pub trait ChangeJournal: Send + Sync {
    /// Where the journal of `volume` stands now; `None` when the volume keeps no journal.
    fn position(&self, volume: &VolumeId) -> Option<JournalPosition>;
    /// The folders changed on `volume` since `since`, up to now.
    fn changes_since(
        &self,
        volume: &VolumeId,
        since: &JournalPosition,
        cancel: &CancelToken,
    ) -> Result<Changes, JournalGap>;
}

/// A point in one journal: which journal (it changes when the journal is reset) and how far into it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JournalPosition {
    pub journal: u128,
    pub cursor: u64,
}

/// What changed between two positions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Changes {
    /// Folders whose own entries changed.
    pub folders: Vec<PathBuf>,
    /// Folders whose whole subtree must be read again.
    pub subtrees: Vec<PathBuf>,
    /// The position the changes reach.
    pub end: JournalPosition,
}

/// Why the journal cannot tell what changed; the rescan becomes a full scan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, thiserror::Error)]
#[serde(rename_all = "kebab-case")]
pub enum JournalGap {
    #[error("the volume keeps no journal")]
    NoJournal,
    #[error("the journal was reset")]
    Reset,
    #[error("the journal dropped changes")]
    Dropped,
    #[error("the journal's positions wrapped around")]
    Wrapped,
    #[error("a scan root itself changed")]
    RootChanged,
    #[error("the position is older than the journal can answer for")]
    TooOld,
    #[error("the read was cancelled")]
    Cancelled,
    #[error("the journal could not be read")]
    Failed,
}
