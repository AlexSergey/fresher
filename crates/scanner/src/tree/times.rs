//! The times of an entry beyond its modification time, and what a folder claimed whole holds. A match needs them, so
//! the tree keeps them for the nodes with hits only: a node stays small.

use fresher_core::{EntryKind, Inside, RawEntry, Timestamp};

/// The status change and the access of an entry, as the reader saw them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Times {
    pub(crate) changed: Option<Timestamp>,
    /// `None` for a folder: listing it moves it.
    pub(crate) accessed: Option<Timestamp>,
}

impl Times {
    /// Nothing known: a scan root, or a folder inside an opaque subtree.
    pub(crate) const NONE: Self = Self { changed: None, accessed: None };

    pub(crate) fn of(entry: &RawEntry) -> Self {
        let accessed = if entry.kind == EntryKind::Folder { None } else { entry.accessed };
        Self { changed: entry.changed, accessed }
    }
}

/// What the tree keeps of a node with hits: its entry's times and, for a folder claimed whole, its inside.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Stamp {
    pub(crate) times: Times,
    pub(crate) inside: Option<Inside>,
}
