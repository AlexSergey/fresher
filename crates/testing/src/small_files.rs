//! Small-file readers of a test: the fixture tree's own files on the disk, or none at all.

use std::fs::OpenOptions;
use std::io::{self, Read};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

use fresher_core::{ReadError, SmallFiles};

/// Reads the files of a fixture tree on the disk with the standard library: a regular file only, never through a
/// link (`symlink_metadata` first, and the opened file must be the entry it saw).
#[derive(Clone, Copy, Debug, Default)]
pub struct DiskFiles;

impl SmallFiles for DiskFiles {
    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        if name.is_empty() || name.contains('/') {
            return Err(ReadError::Failed);
        }
        let path = folder.join(name);
        let entry = std::fs::symlink_metadata(&path).map_err(|error| read_error(&error))?;
        if !entry.file_type().is_file() {
            return Err(ReadError::Replaced);
        }
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(&path)
            .map_err(|error| read_error(&error))?;
        let opened = file.metadata().map_err(|error| read_error(&error))?;
        if (opened.dev(), opened.ino()) != (entry.dev(), entry.ino()) {
            return Err(ReadError::Replaced);
        }
        let mut head = Vec::new();
        file.take(u64::try_from(limit).unwrap_or(u64::MAX))
            .read_to_end(&mut head)
            .map_err(|error| read_error(&error))?;
        Ok(head)
    }
}

fn read_error(error: &io::Error) -> ReadError {
    if error.raw_os_error() == Some(libc::ELOOP) {
        return ReadError::Replaced;
    }
    match error.kind() {
        io::ErrorKind::NotFound => ReadError::Vanished,
        io::ErrorKind::PermissionDenied => ReadError::NoPermission,
        _ => ReadError::Failed,
    }
}

/// Reads no file: every read fails, so every small file is `Unreadable`. For tests whose modules read none.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoFiles;

impl SmallFiles for NoFiles {
    fn head(&self, _folder: &Path, _name: &str, _limit: usize) -> Result<Vec<u8>, ReadError> {
        Err(ReadError::Failed)
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fs;
        use std::os::unix::fs::symlink;

        use fresher_core::{ReadError, SmallFiles};

        use crate::{DiskFiles, NoFiles};

        #[test]
        fn a_link_to_a_regular_file_is_replaced() {
            let root = tempfile::tempdir().unwrap();
            fs::write(root.path().join("target"), "20\n").unwrap();
            symlink(root.path().join("target"), root.path().join(".nvmrc")).unwrap();

            assert_eq!(DiskFiles.head(root.path(), ".nvmrc", 8), Err(ReadError::Replaced));
        }

        #[test]
        fn a_folder_is_replaced() {
            let root = tempfile::tempdir().unwrap();
            fs::create_dir(root.path().join(".nvmrc")).unwrap();

            assert_eq!(DiskFiles.head(root.path(), ".nvmrc", 8), Err(ReadError::Replaced));
        }

        #[test]
        fn a_missing_name_is_vanished() {
            let root = tempfile::tempdir().unwrap();

            assert_eq!(DiskFiles.head(root.path(), ".nvmrc", 8), Err(ReadError::Vanished));
        }

        #[test]
        fn a_name_that_is_not_one_name_fails() {
            let root = tempfile::tempdir().unwrap();
            fs::create_dir(root.path().join("a")).unwrap();
            fs::write(root.path().join("a").join("b"), "x").unwrap();

            assert_eq!(DiskFiles.head(root.path(), "a/b", 8), Err(ReadError::Failed));
            assert_eq!(DiskFiles.head(root.path(), "", 8), Err(ReadError::Failed));
        }

        #[test]
        fn no_files_reads_nothing() {
            let root = tempfile::tempdir().unwrap();
            fs::write(root.path().join(".nvmrc"), "20\n").unwrap();

            assert_eq!(NoFiles.head(root.path(), ".nvmrc", 8), Err(ReadError::Failed));
        }
    }

    mod positive {
        use std::fs;

        use fresher_core::SmallFiles;

        use crate::DiskFiles;

        #[test]
        fn a_regular_file_gives_its_first_bytes() {
            let root = tempfile::tempdir().unwrap();
            fs::write(root.path().join(".nvmrc"), "v20.11.0\n").unwrap();

            assert_eq!(DiskFiles.head(root.path(), ".nvmrc", 3), Ok(b"v20".to_vec()));
            assert_eq!(DiskFiles.head(root.path(), ".nvmrc", 64), Ok(b"v20.11.0\n".to_vec()));
        }
    }
}
