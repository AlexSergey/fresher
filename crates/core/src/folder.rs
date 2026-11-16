//! Reading one folder: the port through which the scanner lists the disk, in OS-neutral words.

use std::ops::Range;
use std::path::Path;

use crate::{EntryKind, Sizes, Timestamp};

/// Reads folders in as few system calls as the OS allows. Never follows a link.
pub trait FolderReader: Send + Sync {
    /// The identity of a root folder; never follows a link.
    fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError>;
    /// Every entry of the folder at `path`, which must still be `expected`; names go into `names`.
    fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError>;
    /// The first `limit` bytes of the regular file `name` in the folder; never follows a link.
    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError>;
}

/// What makes a folder the same folder: its device and its file id there.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FolderIdentity {
    pub device: u64,
    pub file_id: u64,
}

/// One entry of a folder, as the reader saw it; its name is kept by the [`FolderListing`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawEntry {
    pub kind: EntryKind,
    pub device: u64,
    pub file_id: u64,
    /// The hard link count of a file; meaningless for a folder.
    pub links: u32,
    pub sizes: Sizes,
    /// The clone group of a file that may share its blocks with a clone; `None` for any other entry.
    pub clone: Option<u64>,
    pub modified: Option<Timestamp>,
    /// A cloud file whose data is not on this Mac: never read, it takes no allocated space.
    pub cloud_only: bool,
    /// The error of this entry alone; the rest of the folder was read.
    pub error: Option<ReadError>,
}

/// The entries of one folder, with every name in one shared byte buffer. Reused between folders: `clear` keeps the
/// memory, so a reader fills it with no allocation per entry. Names are bytes: a file name is not always UTF-8.
#[derive(Clone, Debug, Default)]
pub struct FolderListing {
    names: Vec<u8>,
    entries: Vec<(Range<usize>, RawEntry)>,
}

impl FolderListing {
    pub fn new() -> Self {
        Self::default()
    }

    /// Forgets every entry and keeps the memory.
    pub fn clear(&mut self) {
        self.names.clear();
        self.entries.clear();
    }

    /// Adds an entry named `name`. A name that is empty, `.`, `..`, or holds a `/` or a NUL is not a single name: it
    /// is refused as `ReadError::Failed` and nothing is stored.
    pub fn push(&mut self, name: &[u8], entry: RawEntry) -> Result<(), ReadError> {
        if name.is_empty() || name == b"." || name == b".." || name.iter().any(|&byte| byte == b'/' || byte == 0) {
            return Err(ReadError::Failed);
        }
        let start = self.names.len();
        self.names.extend_from_slice(name);
        self.entries.push((start..self.names.len(), entry));
        Ok(())
    }

    /// Every entry with its name, in the order pushed.
    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &RawEntry)> {
        self.entries.iter().map(|(range, entry)| (self.names.get(range.clone()).unwrap_or_default(), entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Why a folder, a file or one entry could not be read. No path is held: paths are not logged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, thiserror::Error)]
pub enum ReadError {
    /// Its POSIX permissions refuse the app (`EACCES`).
    #[error("permission denied")]
    NoPermission,
    /// The system's privacy protection refuses the app (`EPERM`).
    #[error("refused by the system's privacy protection")]
    Guarded,
    /// It is gone.
    #[error("it no longer exists")]
    Vanished,
    /// Something else now stands at its path (another folder, a link, a file).
    #[error("it was replaced by something else")]
    Replaced,
    /// The read was interrupted too many times.
    #[error("the read was interrupted")]
    Interrupted,
    /// Any other error (a broken disk, a malformed answer of the system).
    #[error("the read failed")]
    Failed,
}

#[cfg(test)]
mod tests {
    use crate::{EntryKind, RawEntry, Sizes};

    fn entry() -> RawEntry {
        RawEntry {
            kind: EntryKind::File,
            device: 1,
            file_id: 2,
            links: 1,
            sizes: Sizes::ZERO,
            clone: None,
            modified: None,
            cloud_only: false,
            error: None,
        }
    }

    mod negative {
        use super::entry;
        use crate::{FolderListing, ReadError};

        #[test]
        fn a_listing_refuses_a_name_with_a_separator_or_a_nul() {
            let mut listing = FolderListing::new();

            for name in [&b""[..], b".", b"..", b"a/b", b"/", b"a\0b"] {
                assert_eq!(listing.push(name, entry()), Err(ReadError::Failed), "{name:?}");
            }

            assert_eq!(listing.len(), 0);
            assert_eq!(listing.names, Vec::<u8>::new());
        }
    }

    mod positive {
        use super::entry;
        use crate::{FolderListing, RawEntry};

        #[test]
        fn a_listing_keeps_names_as_bytes_in_order() {
            let mut listing = FolderListing::new();
            let other = RawEntry { file_id: 3, ..entry() };

            listing.push(b"b", entry()).unwrap();
            listing.push(b"\xffnot utf-8", other).unwrap();

            let listed: Vec<_> = listing.iter().collect();
            assert_eq!(listed, [(&b"b"[..], &entry()), (&b"\xffnot utf-8"[..], &other)]);
            assert_eq!(listing.len(), 2);
            assert!(!listing.is_empty());
        }

        #[test]
        fn a_listing_is_reused_without_reallocating() {
            let mut listing = FolderListing::new();
            for index in 0..100_u64 {
                listing.push(format!("name-{index}").as_bytes(), RawEntry { file_id: index, ..entry() }).unwrap();
            }
            let (names, entries) = (listing.names.capacity(), listing.entries.capacity());

            listing.clear();
            listing.push(b"again", entry()).unwrap();

            assert_eq!(listing.len(), 1);
            assert_eq!((listing.names.capacity(), listing.entries.capacity()), (names, entries));
        }
    }
}
