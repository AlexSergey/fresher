//! Acting beneath held folders: `fresher-macos`'s calls that name one entry of an open folder, never a path, and the
//! two steps the walk and the deletion share — listing a held folder and opening a folder below it. Off macOS nothing
//! opens, so nothing is acted on (fail closed).

use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::MetadataExt as _;

use fresher_core::{FolderListing, ItemIdentity, SkipReason};

use crate::guard::identity;
use crate::paths::open_failure;

#[cfg(target_os = "macos")]
pub(crate) use fresher_macos::beneath::{identity_at, open_folder, open_folder_at, read_folder, remove_at};

#[cfg(not(target_os = "macos"))]
pub(crate) use elsewhere::{identity_at, open_folder, open_folder_at, read_folder, remove_at};

/// One entry of a held folder as it was listed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Listed {
    pub(crate) name: OsString,
    /// Its volume, file id and kind; `None` when the read could not describe it.
    pub(crate) identity: Option<ItemIdentity>,
    /// The bytes it allocates (0 for a folder: a folder's own blocks are read when it is opened).
    pub(crate) allocated: u64,
    /// Its hard link count (meaningless for a folder).
    pub(crate) links: u32,
}

/// The entries of the held folder `folder`, sorted by name.
pub(crate) fn entries(folder: &File) -> Result<Vec<Listed>, SkipReason> {
    let mut listing = FolderListing::new();
    read_folder(folder, &mut listing).map_err(|_| SkipReason::CheckFailed)?;
    let mut entries: Vec<Listed> = listing
        .iter()
        .map(|(name, entry)| Listed {
            name: OsStr::from_bytes(name).to_os_string(),
            identity: entry.error.is_none().then(|| ItemIdentity::new(entry.device, entry.file_id, entry.kind)),
            allocated: entry.sizes.allocated.bytes(),
            links: entry.links,
        })
        .collect();
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(entries)
}

/// Opens the folder `name` beneath the held folder `folder` through no link, and checks it is still `expected` and
/// on `device`.
pub(crate) fn descend(folder: &File, name: &OsStr, expected: ItemIdentity, device: u64) -> Result<File, SkipReason> {
    let child = open_folder_at(folder, name).map_err(|error| open_failure(&error))?;
    let metadata = child.metadata().map_err(|_| SkipReason::CheckFailed)?;
    if metadata.dev() != device {
        return Err(SkipReason::OtherVolume);
    }
    if identity::of(&metadata) != expected {
        return Err(SkipReason::Replaced);
    }
    Ok(child)
}

/// Off macOS every call fails: the cleaner acts on nothing there.
#[cfg(not(target_os = "macos"))]
mod elsewhere {
    use std::ffi::OsStr;
    use std::fs::File;
    use std::io;
    use std::os::fd::BorrowedFd;
    use std::path::Path;

    use fresher_core::{EntryKind, FolderListing, ItemIdentity, ReadError};

    fn unsupported<T>() -> io::Result<T> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }

    pub(crate) fn open_folder(_path: &Path) -> io::Result<File> {
        unsupported()
    }

    pub(crate) fn open_folder_at(_folder: &File, _name: &OsStr) -> io::Result<File> {
        unsupported()
    }

    pub(crate) fn identity_at(_folder: &File, _name: &OsStr) -> io::Result<ItemIdentity> {
        unsupported()
    }

    pub(crate) fn read_folder(_folder: &File, _out: &mut FolderListing) -> Result<(), ReadError> {
        Err(ReadError::Failed)
    }

    pub(crate) fn remove_at(_folder: BorrowedFd<'_>, _name: &OsStr, _kind: EntryKind) -> io::Result<()> {
        unsupported()
    }
}
