//! `StdReader`: the portable `FolderReader`, with `std::fs` only. The reader of the tests, and the fallback on a system
//! without its own adapter.

use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read as _};
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::{MetadataExt as _, OpenOptionsExt as _};
use std::path::Path;

use fresher_core::{
    ByteSize, EntryKind, FolderIdentity, FolderListing, FolderReader, RawEntry, ReadError, Sizes, Timestamp,
};

/// How many times a call interrupted by a signal is tried again.
const RETRIES: usize = 3;
const FOLDER_FLAGS: i32 = libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;

/// Lists a folder with `read_dir` and one `symlink_metadata` per entry. Never follows a link: the folder is opened
/// with `O_NOFOLLOW | O_DIRECTORY` and checked against the identity its parent's listing announced, before and after
/// the listing. It cannot see clones: every file's reclaimable size is its allocated size.
#[derive(Clone, Copy, Debug, Default)]
pub struct StdReader;

impl StdReader {
    pub const fn new() -> Self {
        Self
    }
}

impl FolderReader for StdReader {
    fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError> {
        identity(&open_folder(path)?)
    }

    /// Clears `out` first. A folder that is no longer `expected` is `Replaced`; one removed while read, `Vanished`.
    fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
        out.clear();
        let folder = open_folder(path)?;
        if identity(&folder)? != expected {
            return Err(ReadError::Replaced);
        }
        for entry in retrying(|| fs::read_dir(path))? {
            let entry = entry.map_err(|error| read_error(&error))?;
            let raw = match fs::symlink_metadata(entry.path()) {
                Ok(metadata) => raw_entry(&metadata),
                Err(error) => unreadable_entry(read_error(&error)),
            };
            out.push(entry.file_name().as_bytes(), raw)?;
        }
        if folder.metadata().map_err(|error| read_error(&error))?.nlink() == 0 {
            return Err(ReadError::Vanished);
        }
        let now = fs::symlink_metadata(path).map_err(|error| read_error(&error))?;
        if (FolderIdentity { device: now.dev(), file_id: now.ino() }) != expected {
            return Err(ReadError::Replaced);
        }
        Ok(())
    }

    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        if name.is_empty() || name.contains('/') {
            return Err(ReadError::Failed);
        }
        open_folder(folder)?;
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(folder.join(name))
            .map_err(|error| match error.raw_os_error() {
                Some(libc::ELOOP) => ReadError::Replaced,
                _ => read_error(&error),
            })?;
        if !file.metadata().map_err(|error| read_error(&error))?.is_file() {
            return Err(ReadError::Replaced);
        }
        let mut head = Vec::new();
        file.take(u64::try_from(limit).unwrap_or(u64::MAX))
            .read_to_end(&mut head)
            .map_err(|error| read_error(&error))?;
        Ok(head)
    }
}

/// An entry as `symlink_metadata` sees it. A folder's own size is not counted; a file's allocated size is its blocks.
fn raw_entry(metadata: &Metadata) -> RawEntry {
    let file_type = metadata.file_type();
    let kind = if file_type.is_symlink() {
        EntryKind::Symlink
    } else if file_type.is_dir() {
        EntryKind::Folder
    } else {
        EntryKind::File
    };
    let sizes = if kind == EntryKind::Folder {
        Sizes::ZERO
    } else {
        Sizes::complete(ByteSize::new(metadata.len()), ByteSize::new(metadata.blocks().saturating_mul(512)))
    };
    RawEntry {
        kind,
        device: metadata.dev(),
        file_id: metadata.ino(),
        links: u32::try_from(metadata.nlink()).unwrap_or(u32::MAX),
        sizes,
        clone: None,
        modified: u64::try_from(metadata.mtime()).ok().map(Timestamp::from_secs),
        cloud_only: false,
        error: None,
    }
}

const fn unreadable_entry(error: ReadError) -> RawEntry {
    RawEntry {
        kind: EntryKind::File,
        device: 0,
        file_id: 0,
        links: 1,
        sizes: Sizes::ZERO,
        clone: None,
        modified: None,
        cloud_only: false,
        error: Some(error),
    }
}

fn identity(folder: &File) -> Result<FolderIdentity, ReadError> {
    let metadata = folder.metadata().map_err(|error| read_error(&error))?;
    Ok(FolderIdentity { device: metadata.dev(), file_id: metadata.ino() })
}

/// Opens the folder at `path` without following a link; something else at the path is `Replaced`.
fn open_folder(path: &Path) -> Result<File, ReadError> {
    retrying(|| OpenOptions::new().read(true).custom_flags(FOLDER_FLAGS).open(path)).map_err(|error| match error {
        ReadError::Vanished if fs::symlink_metadata(path).is_ok() => ReadError::Replaced,
        other => other,
    })
}

/// Runs `call` again when a signal interrupted it, up to `RETRIES` times.
fn retrying<T>(mut call: impl FnMut() -> io::Result<T>) -> Result<T, ReadError> {
    let mut tries = 0;
    loop {
        match call() {
            Err(error) if error.kind() == io::ErrorKind::Interrupted && tries < RETRIES => tries += 1,
            result => return result.map_err(|error| read_error(&error)),
        }
    }
}

/// What an error of the system means for a read: `EPERM` is the privacy protection, `EACCES` the POSIX permissions.
pub(crate) fn read_error(error: &io::Error) -> ReadError {
    match error.raw_os_error() {
        Some(libc::EPERM) => ReadError::Guarded,
        Some(libc::EACCES) => ReadError::NoPermission,
        Some(libc::ENOENT | libc::ENOTDIR) => ReadError::Vanished,
        Some(libc::ELOOP) => ReadError::Replaced,
        Some(libc::EINTR) => ReadError::Interrupted,
        _ => ReadError::Failed,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use fresher_core::{FolderIdentity, FolderListing, FolderReader, RawEntry, ReadError};

    use super::StdReader;

    fn read(path: &Path) -> Result<Vec<(String, RawEntry)>, ReadError> {
        let reader = StdReader::new();
        let identity = reader.identify(path)?;
        let mut listing = FolderListing::new();
        reader.read(path, identity, &mut listing)?;
        let mut entries: Vec<_> =
            listing.iter().map(|(name, entry)| (String::from_utf8_lossy(name).into_owned(), *entry)).collect();
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        Ok(entries)
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::symlink;

        use fresher_core::{FolderIdentity, FolderListing, FolderReader, ReadError};
        use fresher_testing::FixtureTree;

        use super::read;
        use crate::walk::std_reader::{StdReader, read_error};

        #[test]
        fn the_std_reader_refuses_a_folder_swapped_for_a_link() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"a\"\n[[dir]]\npath = \"b\"\n").unwrap();
            let reader = StdReader::new();
            let identity = reader.identify(&tree.root().join("a")).unwrap();
            fs::rename(tree.root().join("a"), tree.root().join("moved")).unwrap();
            symlink(tree.root().join("b"), tree.root().join("a")).unwrap();

            let mut listing = FolderListing::new();
            let swapped = reader.read(&tree.root().join("a"), identity, &mut listing);

            assert_eq!(swapped, Err(ReadError::Replaced));
            assert_eq!(reader.identify(&tree.root().join("a")), Err(ReadError::Replaced));
        }

        #[test]
        fn the_std_reader_refuses_a_folder_swapped_for_another() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"a\"\n[[dir]]\npath = \"b\"\n").unwrap();
            let reader = StdReader::new();
            let identity = reader.identify(&tree.root().join("a")).unwrap();
            fs::rename(tree.root().join("a"), tree.root().join("moved")).unwrap();
            fs::rename(tree.root().join("b"), tree.root().join("a")).unwrap();

            let swapped = reader.read(&tree.root().join("a"), identity, &mut FolderListing::new());

            assert_eq!(swapped, Err(ReadError::Replaced));
        }

        #[test]
        fn a_missing_folder_or_a_file_is_not_a_folder() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"file\"\n").unwrap();
            let reader = StdReader::new();
            let anything = FolderIdentity { device: 0, file_id: 0 };

            assert_eq!(reader.identify(&tree.root().join("missing")), Err(ReadError::Vanished));
            assert_eq!(reader.identify(&tree.root().join("file")), Err(ReadError::Replaced));
            assert_eq!(
                reader.read(&tree.root().join("missing"), anything, &mut FolderListing::new()),
                Err(ReadError::Vanished)
            );
            assert!(read(&tree.root().join("missing")).is_err());
        }

        #[test]
        fn head_refuses_a_link_a_folder_and_a_path() {
            let tree =
                FixtureTree::from_toml_str("[[file]]\npath = \"a/real\"\n[[dir]]\npath = \"a/inner\"\n").unwrap();
            let folder = tree.root().join("a");
            symlink(folder.join("real"), folder.join("link")).unwrap();
            let reader = StdReader::new();

            assert_eq!(reader.head(&folder, "link", 43), Err(ReadError::Replaced));
            assert_eq!(reader.head(&folder, "inner", 43), Err(ReadError::Replaced));
            assert_eq!(reader.head(&folder, "x/y", 43), Err(ReadError::Failed));
            assert_eq!(reader.head(&folder, "", 43), Err(ReadError::Failed));
            assert_eq!(reader.head(&folder, "missing", 43), Err(ReadError::Vanished));
        }

        #[test]
        fn errors_are_told_apart_by_their_number() {
            for (errno, meaning) in [
                (libc::EPERM, ReadError::Guarded),
                (libc::EACCES, ReadError::NoPermission),
                (libc::ENOENT, ReadError::Vanished),
                (libc::ELOOP, ReadError::Replaced),
                (libc::EINTR, ReadError::Interrupted),
                (libc::EIO, ReadError::Failed),
            ] {
                assert_eq!(read_error(&std::io::Error::from_raw_os_error(errno)), meaning, "{errno}");
            }
            assert_eq!(read_error(&std::io::Error::other("no number")), ReadError::Failed);
        }

        #[test]
        fn a_folder_without_permission_is_refused_by_kind() {
            use std::os::unix::fs::PermissionsExt;
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"locked/a\"\n").unwrap();
            let locked = tree.root().join("locked");
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

            let result = read(&locked);
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();

            assert_eq!(result.err(), Some(ReadError::NoPermission));
        }
    }

    mod positive {
        use std::os::unix::fs::symlink;

        use fresher_core::{ByteSize, EntryKind, FolderReader, Timestamp};
        use fresher_testing::FixtureTree;

        use super::{FolderIdentity, read};
        use crate::walk::std_reader::StdReader;

        #[test]
        fn the_std_reader_lists_a_folder_with_its_sizes() {
            let tree = FixtureTree::from_toml_str(
                "[[file]]\npath = \"big.bin\"\nsize = 5000000\nmodified = 1700000000\n[[dir]]\npath = \"sub\"\n\
                 [[symlink]]\npath = \"out\"\ntarget = \"/usr\"\n",
            )
            .unwrap();

            let entries = read(tree.root()).unwrap();

            let names: Vec<_> = entries.iter().map(|(name, entry)| (name.as_str(), entry.kind)).collect();
            assert_eq!(names, [("big.bin", EntryKind::File), ("out", EntryKind::Symlink), ("sub", EntryKind::Folder)]);
            let big = entries[0].1;
            assert_eq!(big.sizes.logical, ByteSize::new(5_000_000));
            assert!(big.sizes.allocated < big.sizes.logical, "the fixture's file is sparse");
            assert_eq!(big.sizes.reclaimable, big.sizes.allocated);
            assert_eq!((big.modified, big.links, big.clone), (Some(Timestamp::from_secs(1_700_000_000)), 1, None));
            assert_eq!(entries[2].1.sizes, fresher_core::Sizes::ZERO);
            assert!(entries.iter().all(|(_, entry)| entry.error.is_none()));
        }

        #[test]
        fn the_std_reader_reports_hard_links() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a\"\nsize = 10\n").unwrap();
            std::fs::hard_link(tree.root().join("a"), tree.root().join("b")).unwrap();

            let entries = read(tree.root()).unwrap();

            assert_eq!(entries.len(), 2);
            assert!(entries.iter().all(|(_, entry)| entry.links == 2));
            assert_eq!(entries[0].1.file_id, entries[1].1.file_id);
        }

        #[test]
        fn head_reads_the_first_bytes_of_a_regular_file() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"a\"\n").unwrap();
            std::fs::write(tree.root().join("a/tag"), b"0123456789").unwrap();
            symlink("/usr", tree.root().join("a/elsewhere")).unwrap();

            let head = StdReader::new().head(&tree.root().join("a"), "tag", 4).unwrap();

            assert_eq!(head, b"0123");
        }

        #[test]
        fn identify_names_the_folder_by_device_and_file_id() {
            use std::os::unix::fs::MetadataExt;
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"a\"\n").unwrap();
            let metadata = std::fs::metadata(tree.root().join("a")).unwrap();

            let identity = StdReader::new().identify(&tree.root().join("a")).unwrap();

            assert_eq!(identity, FolderIdentity { device: metadata.dev(), file_id: metadata.ino() });
        }
    }
}
