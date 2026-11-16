//! `MacFolderReader`: the core's `FolderReader` over the bulk read.

use std::cell::RefCell;
use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read as _};
use std::os::fd::{AsFd as _, BorrowedFd};
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::{MetadataExt as _, OpenOptionsExt as _};
use std::path::{Component, Path};

use fresher_core::{FolderIdentity, FolderListing, FolderReader, ReadError};

use super::parse::{self, COMMON, EXTENDED, FILE};
use crate::errors::read_error;
use crate::ffi::bulk;

/// The buffer of one bulk read: 16, 64 and 256 KiB read alike in 01; 64 KiB taken.
const BUFFER_SIZE: usize = 64 * 1024;
/// How many times a call interrupted by a signal is tried again.
const RETRIES: usize = 3;
const FOLDER_FLAGS: libc::c_int = libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;

const LIST: libc::attrlist = libc::attrlist {
    bitmapcount: libc::ATTR_BIT_MAP_COUNT,
    reserved: 0,
    commonattr: COMMON,
    volattr: 0,
    dirattr: 0,
    fileattr: FILE,
    forkattr: EXTENDED,
};
const OPTIONS: u32 = libc::FSOPT_PACK_INVAL_ATTRS | libc::FSOPT_ATTR_CMN_EXTENDED | libc::FSOPT_NOFOLLOW;

/// `ATTR_CMNEXT_PRIVATESIZE`, asked per clone: the bytes of the file no clone shares.
const PRIVATE_SIZE: libc::attrlist = libc::attrlist {
    bitmapcount: libc::ATTR_BIT_MAP_COUNT,
    reserved: 0,
    commonattr: 0,
    volattr: 0,
    dirattr: 0,
    fileattr: 0,
    forkattr: libc::ATTR_CMNEXT_PRIVATESIZE,
};

thread_local! {
    /// One buffer per thread, kept between folders.
    static BUFFER: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Reads a folder's entries with their attributes in one system call per batch (`getattrlistbulk`). Never follows a
/// link, never downloads a cloud file (with the IO policy set), never blocks on a FIFO.
#[derive(Clone, Copy, Debug)]
pub struct MacFolderReader {
    buffer_size: usize,
}

impl MacFolderReader {
    pub const fn new() -> Self {
        Self { buffer_size: BUFFER_SIZE }
    }

    #[cfg(test)]
    const fn with_buffer_size(buffer_size: usize) -> Self {
        Self { buffer_size }
    }
}

impl Default for MacFolderReader {
    fn default() -> Self {
        Self::new()
    }
}

impl FolderReader for MacFolderReader {
    fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError> {
        identity(&open_folder(path)?)
    }

    /// Clears `out` first. A folder that is no longer `expected` (a link or another folder now stands at its path) is
    /// `Replaced`; a folder removed while it was read is `Vanished`.
    fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
        out.clear();
        let folder = open_folder(path)?;
        if identity(&folder)? != expected {
            return Err(ReadError::Replaced);
        }
        BUFFER.with_borrow_mut(|buffer| {
            buffer.resize(self.buffer_size, 0);
            read_all(folder.as_fd(), buffer, out)
        })?;
        if out.is_empty() && folder.metadata().map_err(|error| read_error(&error))?.nlink() == 0 {
            return Err(ReadError::Vanished);
        }
        Ok(())
    }

    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        if name.contains('/') {
            return Err(ReadError::Failed);
        }
        let name = CString::new(name).map_err(|_| ReadError::Failed)?;
        let folder = open_folder(folder)?;
        let file = bulk::open_at(folder.as_fd(), &name, libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .map_err(|error| read_error(&error))?;
        let file = File::from(file);
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

fn read_all(folder: BorrowedFd<'_>, buffer: &mut [u8], out: &mut FolderListing) -> Result<(), ReadError> {
    loop {
        let count = retrying(|| bulk::read_entries(folder, &LIST, buffer, u64::from(OPTIONS)))?;
        if count == 0 {
            return Ok(());
        }
        for record in parse::records(buffer, count) {
            let record = record?;
            let private = if record.may_share_blocks() { private_size(folder, record.name) } else { None };
            out.push(record.name, record.entry(private))?;
        }
    }
}

/// The private size of the clone `name`; `None` (counted as allocated) if it cannot be read.
fn private_size(folder: BorrowedFd<'_>, name: &[u8]) -> Option<u64> {
    let name = CString::new(name).ok()?;
    let mut buffer = [0_u8; 16];
    let options = libc::FSOPT_NOFOLLOW | libc::FSOPT_ATTR_CMN_EXTENDED;
    match bulk::attributes_at(folder, &name, &PRIVATE_SIZE, &mut buffer, options) {
        Ok(()) => buffer.get(4..12).and_then(|bytes| bytes.try_into().ok()).map(u64::from_ne_bytes),
        Err(error) => {
            tracing::debug!(%error, "the private size of a clone is unknown");
            None
        }
    }
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

fn identity(folder: &File) -> Result<FolderIdentity, ReadError> {
    let metadata = folder.metadata().map_err(|error| read_error(&error))?;
    Ok(FolderIdentity { device: metadata.dev(), file_id: metadata.ino() })
}

/// Opens the folder at `path` without following a link. A path past `PATH_MAX` is opened one name at a time.
fn open_folder(path: &Path) -> Result<File, ReadError> {
    let opened = OpenOptions::new().read(true).custom_flags(FOLDER_FLAGS).open(path);
    let opened = match opened {
        Err(error) if error.raw_os_error() == Some(libc::ENAMETOOLONG) => open_in_steps(path),
        opened => opened,
    };
    opened.map_err(|error| match error.raw_os_error() {
        // Something that is not a folder stands at the path (a link, a file), or the path itself is gone.
        Some(libc::ENOTDIR) if fs::symlink_metadata(path).is_ok() => ReadError::Replaced,
        _ => read_error(&error),
    })
}

fn open_in_steps(path: &Path) -> io::Result<File> {
    let mut components = path.components();
    if components.next() != Some(Component::RootDir) {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    let mut folder = OpenOptions::new().read(true).custom_flags(FOLDER_FLAGS).open("/")?;
    for component in components {
        let Component::Normal(name) = component else {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        };
        let name = CString::new(name.as_bytes()).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
        folder = File::from(bulk::open_at(folder.as_fd(), &name, libc::O_RDONLY | FOLDER_FLAGS)?);
    }
    Ok(folder)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use fresher_core::{FolderListing, FolderReader, RawEntry, ReadError};

    use super::MacFolderReader;

    pub(super) fn listing(reader: MacFolderReader, path: &Path) -> Result<BTreeMap<Vec<u8>, RawEntry>, ReadError> {
        let mut out = FolderListing::new();
        reader.read(path, reader.identify(path)?, &mut out)?;
        Ok(out.iter().map(|(name, entry)| (name.to_vec(), *entry)).collect())
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::{PermissionsExt as _, symlink};
        use std::process::Command;

        use fresher_core::{FolderListing, FolderReader, ReadError};

        use crate::MacFolderReader;

        #[test]
        fn a_folder_replaced_by_a_link_is_refused() {
            let root = tempfile::tempdir().unwrap();
            let (folder, moved) = (root.path().join("folder"), root.path().join("moved"));
            fs::create_dir(&folder).unwrap();
            let reader = MacFolderReader::new();
            let identity = reader.identify(&folder).unwrap();

            fs::rename(&folder, &moved).unwrap();
            symlink(&moved, &folder).unwrap();

            assert_eq!(reader.read(&folder, identity, &mut FolderListing::new()), Err(ReadError::Replaced));
            assert_eq!(reader.identify(&folder), Err(ReadError::Replaced));
        }

        #[test]
        fn a_folder_replaced_by_another_folder_is_refused() {
            let root = tempfile::tempdir().unwrap();
            let folder = root.path().join("folder");
            fs::create_dir(&folder).unwrap();
            fs::write(folder.join("kept"), b"x").unwrap();
            let reader = MacFolderReader::new();
            let identity = reader.identify(&folder).unwrap();

            fs::rename(&folder, root.path().join("moved")).unwrap();
            fs::create_dir(&folder).unwrap();
            let mut out = FolderListing::new();

            assert_eq!(reader.read(&folder, identity, &mut out), Err(ReadError::Replaced));
            assert!(out.is_empty());
        }

        #[test]
        fn a_folder_replaced_by_a_file_is_refused() {
            let root = tempfile::tempdir().unwrap();
            let folder = root.path().join("folder");
            fs::create_dir(&folder).unwrap();
            let reader = MacFolderReader::new();
            let identity = reader.identify(&folder).unwrap();

            fs::rename(&folder, root.path().join("moved")).unwrap();
            fs::write(&folder, b"a file").unwrap();

            assert_eq!(reader.read(&folder, identity, &mut FolderListing::new()), Err(ReadError::Replaced));
        }

        #[test]
        fn a_folder_with_mode_000_is_no_permission() {
            let root = tempfile::tempdir().unwrap();
            let folder = root.path().join("locked");
            fs::create_dir(&folder).unwrap();
            fs::set_permissions(&folder, fs::Permissions::from_mode(0o000)).unwrap();

            let identified = MacFolderReader::new().identify(&folder);

            fs::set_permissions(&folder, fs::Permissions::from_mode(0o755)).unwrap();
            assert_eq!(identified, Err(ReadError::NoPermission));
        }

        #[test]
        fn a_vanished_folder_is_reported_vanished() {
            let root = tempfile::tempdir().unwrap();
            let folder = root.path().join("folder");
            fs::create_dir(&folder).unwrap();
            let reader = MacFolderReader::new();
            let identity = reader.identify(&folder).unwrap();

            fs::rename(&folder, root.path().join("moved")).unwrap();

            assert_eq!(reader.read(&folder, identity, &mut FolderListing::new()), Err(ReadError::Vanished));
            assert_eq!(reader.identify(&folder.join("below")), Err(ReadError::Vanished));
        }

        #[test]
        fn head_does_not_follow_a_link() {
            let root = tempfile::tempdir().unwrap();
            fs::write(root.path().join("real"), b"Signature: 8a477f597d28d172789f06886806bc55").unwrap();
            symlink(root.path().join("real"), root.path().join("link")).unwrap();

            assert_eq!(MacFolderReader::new().head(root.path(), "link", 43), Err(ReadError::Replaced));
        }

        #[test]
        fn head_does_not_block_on_a_fifo() {
            let root = tempfile::tempdir().unwrap();
            let status = Command::new("/usr/bin/mkfifo").arg(root.path().join("fifo")).status().unwrap();
            assert!(status.success());

            assert_eq!(MacFolderReader::new().head(root.path(), "fifo", 43), Err(ReadError::Replaced));
        }

        #[test]
        fn head_refuses_a_name_that_is_not_one_name() {
            let root = tempfile::tempdir().unwrap();
            let reader = MacFolderReader::new();

            assert_eq!(reader.head(root.path(), "a/b", 8), Err(ReadError::Failed));
            assert_eq!(reader.head(root.path(), "a\0b", 8), Err(ReadError::Failed));
            assert_eq!(reader.head(root.path(), "missing", 8), Err(ReadError::Vanished));
        }

        #[test]
        fn a_relative_path_past_path_max_is_refused() {
            let long = "a/".repeat(600);

            assert_eq!(MacFolderReader::new().identify(long.as_ref()), Err(ReadError::Failed));
        }
    }

    mod positive {
        use std::fs::{self, File};
        use std::os::unix::ffi::OsStrExt as _;
        use std::os::unix::fs::{MetadataExt as _, symlink};
        use std::path::{Path, PathBuf};

        use fresher_core::{ByteSize, EntryKind, FolderListing, FolderReader};

        use super::listing;
        use crate::MacFolderReader;

        fn many_files(folder: &Path, count: usize) {
            fs::create_dir(folder).unwrap();
            for index in 0..count {
                File::create(folder.join(format!("file-{index:05}"))).unwrap();
            }
        }

        #[test]
        fn lists_the_same_entries_as_std() {
            let root = tempfile::tempdir().unwrap();
            let base = root.path();
            fs::write(base.join("empty"), b"").unwrap();
            fs::write(base.join("small"), b"hello").unwrap();
            fs::write(base.join("large"), vec![9_u8; 3 * 1024 * 1024 + 17]).unwrap();
            File::create(base.join("sparse")).unwrap().set_len(1 << 30).unwrap();
            fs::hard_link(base.join("small"), base.join("hard")).unwrap();
            fs::create_dir(base.join("folder")).unwrap();
            fs::create_dir(base.join("folder").join("inner")).unwrap();
            symlink("small", base.join("link")).unwrap();
            symlink("/nowhere/at/all", base.join("dangling")).unwrap();
            fs::write(base.join("\u{e9}t\u{e9} \u{1f600}"), b"utf-8 name").unwrap();
            many_files(&base.join("many"), 10_000);
            let reader = MacFolderReader::new();

            for folder in [base.to_path_buf(), base.join("many")] {
                let listed = listing(reader, &folder).unwrap();
                let expected: Vec<PathBuf> =
                    fs::read_dir(&folder).unwrap().map(|entry| entry.unwrap().path()).collect();

                assert_eq!(listed.len(), expected.len());
                for path in expected {
                    let metadata = fs::symlink_metadata(&path).unwrap();
                    let entry = listed[path.file_name().unwrap().as_bytes()];
                    let kind = if metadata.is_dir() {
                        EntryKind::Folder
                    } else if metadata.is_symlink() {
                        EntryKind::Symlink
                    } else {
                        EntryKind::File
                    };
                    assert_eq!(entry.kind, kind, "{path:?}");
                    assert_eq!((entry.device, entry.file_id), (metadata.dev(), metadata.ino()), "{path:?}");
                    assert_eq!(entry.error, None);
                    assert!(entry.modified.is_some());
                    if kind != EntryKind::Folder {
                        assert_eq!(u64::from(entry.links), metadata.nlink(), "{path:?}");
                        assert_eq!(entry.sizes.logical, ByteSize::new(metadata.len()), "{path:?}");
                        assert_eq!(entry.sizes.allocated, ByteSize::new(metadata.blocks() * 512), "{path:?}");
                    }
                }
            }
        }

        #[test]
        fn reads_a_large_folder_in_several_calls() {
            let root = tempfile::tempdir().unwrap();
            let folder = root.path().join("many");
            many_files(&folder, 2_000);

            let listed = listing(MacFolderReader::with_buffer_size(1024), &folder).unwrap();

            assert_eq!(listed.len(), 2_000);
            assert!(listed.contains_key(b"file-01999".as_slice()));
        }

        #[test]
        fn a_listing_is_cleared_before_a_read() {
            let root = tempfile::tempdir().unwrap();
            fs::write(root.path().join("only"), b"x").unwrap();
            let reader = MacFolderReader::new();
            let mut out = FolderListing::new();
            out.push(b"stale", reader_entry()).unwrap();

            reader.read(root.path(), reader.identify(root.path()).unwrap(), &mut out).unwrap();

            let names: Vec<_> = out.iter().map(|(name, _)| name.to_vec()).collect();
            assert_eq!(names, [b"only".to_vec()]);
        }

        fn reader_entry() -> fresher_core::RawEntry {
            fresher_core::RawEntry {
                kind: EntryKind::File,
                device: 0,
                file_id: 0,
                links: 1,
                sizes: fresher_core::Sizes::ZERO,
                clone: None,
                modified: None,
                cloud_only: false,
                error: None,
            }
        }

        #[test]
        fn a_clone_reports_its_clone_id_and_private_size() {
            let root = tempfile::tempdir().unwrap();
            fs::write(root.path().join("original"), vec![5_u8; 256 * 1024]).unwrap();
            crate::fixtures::clone_file(&root.path().join("original"), &root.path().join("clone")).unwrap();
            fs::write(root.path().join("plain"), vec![5_u8; 256 * 1024]).unwrap();

            let listed = listing(MacFolderReader::new(), root.path()).unwrap();

            let (original, clone, plain) = (listed[&b"original"[..]], listed[&b"clone"[..]], listed[&b"plain"[..]]);
            assert!(clone.clone.is_some());
            assert_eq!(original.clone, clone.clone);
            assert_eq!(plain.clone, None);
            assert_eq!(clone.sizes.allocated, ByteSize::new(256 * 1024));
            assert_eq!(clone.sizes.reclaimable, ByteSize::ZERO);
            assert_eq!(plain.sizes.reclaimable, plain.sizes.allocated);
        }

        #[test]
        fn head_reads_at_most_the_limit() {
            let root = tempfile::tempdir().unwrap();
            fs::write(root.path().join("CACHEDIR.TAG"), b"Signature: 8a477f597d28d172789f06886806bc55 and more")
                .unwrap();
            let reader = MacFolderReader::new();

            assert_eq!(reader.head(root.path(), "CACHEDIR.TAG", 9).unwrap(), b"Signature");
            assert_eq!(reader.head(root.path(), "CACHEDIR.TAG", 1000).unwrap().len(), 52);
        }

        #[test]
        fn a_folder_past_path_max_is_read_in_steps() {
            let root = tempfile::tempdir().unwrap();
            let base = fs::canonicalize(root.path()).unwrap();
            let step = "d".repeat(200);
            let chain = |top: &Path| (0..3).fold(top.to_path_buf(), |path, _| path.join(&step));
            let first = chain(&base.join("first"));
            fs::create_dir_all(&first).unwrap();
            let second = chain(&base.join("second"));
            fs::create_dir_all(&second).unwrap();
            fs::write(second.join("deep"), b"x").unwrap();
            fs::rename(base.join("second"), first.join("second")).unwrap();
            let deep = chain(&first.join("second"));
            assert!(deep.as_os_str().len() > 1024);

            let listed = listing(MacFolderReader::new(), &deep).unwrap();

            assert_eq!(listed.keys().collect::<Vec<_>>(), [&b"deep".to_vec()]);
        }
    }
}
