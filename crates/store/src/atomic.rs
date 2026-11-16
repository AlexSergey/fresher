//! Whole-file reads and atomic writes in the data folder: a reader sees the old file or the new one, never a part.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

use fresher_core::StoreError;
use tempfile::{Builder, NamedTempFile};

use crate::DataFolder;

/// Writes `bytes` as the file `name`: a temporary file in the same folder, `fsync`, a rename over the old file, then
/// `fsync` of the folder. On any failure before the rename the old file is untouched and the temporary file is gone.
///
/// The folder's `fsync` comes after the rename, when the new file is already in place: its failure is logged, not
/// reported, because the write did happen and a retry would change nothing.
pub(crate) fn write_atomic(folder: &DataFolder, name: &str, bytes: &[u8]) -> Result<(), StoreError> {
    let staged = stage(folder, name, bytes)?;
    commit(folder, staged)
}

/// The first half of `write_atomic`: `bytes` written and synced to a temporary file for `name`, not yet in place.
/// Dropping it removes the temporary file and leaves the folder as it was.
#[derive(Debug)]
pub(crate) struct Staged {
    temporary: NamedTempFile,
    target: PathBuf,
}

pub(crate) fn stage(folder: &DataFolder, name: &str, bytes: &[u8]) -> Result<Staged, StoreError> {
    let target = file_path(folder, name)?;
    let mut temporary = temporary_file(folder, name)?;
    temporary.write_all(bytes).map_err(|error| io_error(&error))?;
    temporary.as_file().sync_all().map_err(|error| io_error(&error))?;
    Ok(Staged { temporary, target })
}

/// The second half of `write_atomic`: the staged file renamed over its target, then the folder synced.
pub(crate) fn commit(folder: &DataFolder, staged: Staged) -> Result<(), StoreError> {
    // A failed rename hands the temporary file back inside the error; dropping it removes it.
    staged.temporary.persist(&staged.target).map_err(|error| io_error(&error.error))?;
    if let Err(error) = File::open(folder.path()).and_then(|opened| opened.sync_all()) {
        tracing::warn!(kind = %error.kind(), "the data folder could not be synced after a write");
    }
    Ok(())
}

/// A new temporary file for `name`, named `.<name>.<random>.tmp`, so that one left by a crash is recognized as ours.
fn temporary_file(folder: &DataFolder, name: &str) -> Result<NamedTempFile, StoreError> {
    let prefix = format!(".{name}.");
    Builder::new().prefix(&prefix).suffix(".tmp").tempfile_in(folder.path()).map_err(|error| io_error(&error))
}

/// Reads the whole file `name`; `None` when there is no such file.
pub(crate) fn read_file(folder: &DataFolder, name: &str) -> Result<Option<Vec<u8>>, StoreError> {
    match fs::read(file_path(folder, name)?) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_error(&error)),
    }
}

/// The path of the store's own file `name`; a name that is not one plain file name is refused as invalid input.
pub(crate) fn file_path(folder: &DataFolder, name: &str) -> Result<PathBuf, StoreError> {
    folder.file(name).map_err(|_| StoreError::Io { kind: io::ErrorKind::InvalidInput })
}

pub(crate) fn io_error(error: &io::Error) -> StoreError {
    StoreError::Io { kind: error.kind() }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use crate::DataFolder;

    fn folder() -> (TempDir, DataFolder) {
        let root = TempDir::new().unwrap();
        let folder = DataFolder::open(root.path()).unwrap();
        (root, folder)
    }

    fn names(path: &Path) -> Vec<String> {
        let mut names: Vec<String> =
            fs::read_dir(path).unwrap().map(|entry| entry.unwrap().file_name().into_string().unwrap()).collect();
        names.sort();
        names
    }

    mod negative {
        use std::fs;
        use std::io;
        use std::os::unix::fs::PermissionsExt;

        use fresher_core::StoreError;

        use super::{folder, names};
        use crate::atomic::{read_file, write_atomic};

        #[test]
        fn a_failed_write_leaves_the_old_file_intact() {
            let (root, folder) = folder();
            write_atomic(&folder, "settings.json", b"old").unwrap();
            fs::set_permissions(root.path(), fs::Permissions::from_mode(0o500)).unwrap();

            let result = write_atomic(&folder, "settings.json", b"new");

            fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
            assert_eq!(result, Err(StoreError::Io { kind: io::ErrorKind::PermissionDenied }));
            assert_eq!(read_file(&folder, "settings.json").unwrap().unwrap(), b"old");
        }

        #[test]
        fn leaves_no_temporary_file_behind() {
            let (root, folder) = folder();
            // A non-empty folder in the target's place makes the rename fail after the temporary file is written.
            fs::create_dir_all(root.path().join("settings.json").join("inside")).unwrap();

            let result = write_atomic(&folder, "settings.json", b"new");

            assert!(matches!(result, Err(StoreError::Io { .. })), "{result:?}");
            assert_eq!(names(root.path()), ["settings.json"]);
        }

        #[test]
        fn refuses_a_file_name_with_a_separator() {
            let (_root, folder) = folder();
            let invalid = Err(StoreError::Io { kind: io::ErrorKind::InvalidInput });

            assert_eq!(write_atomic(&folder, "../settings.json", b"new"), invalid);
            assert_eq!(read_file(&folder, "../settings.json"), invalid.map(|()| None));
        }

        #[test]
        fn a_folder_that_cannot_be_synced_after_the_rename_is_not_a_failed_write() {
            let (root, folder) = folder();
            write_atomic(&folder, "settings.json", b"old").unwrap();
            // Write and enter, but not read: the rename works, opening the folder to sync it does not.
            fs::set_permissions(root.path(), fs::Permissions::from_mode(0o300)).unwrap();

            let result = write_atomic(&folder, "settings.json", b"new");

            fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
            assert_eq!(result, Ok(()));
            assert_eq!(read_file(&folder, "settings.json").unwrap().unwrap(), b"new");
            assert_eq!(names(root.path()), ["settings.json"]);
        }

        #[test]
        fn a_missing_file_reads_as_nothing() {
            let (_root, folder) = folder();

            assert_eq!(read_file(&folder, "settings.json"), Ok(None));
        }

        #[test]
        fn reports_a_file_it_cannot_read() {
            let (root, folder) = folder();
            fs::create_dir(root.path().join("settings.json")).unwrap();

            assert!(matches!(read_file(&folder, "settings.json"), Err(StoreError::Io { .. })));
        }
    }

    mod positive {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        use super::{folder, names};
        use crate::atomic::{read_file, temporary_file, write_atomic};

        #[test]
        fn writes_and_reads_back() {
            let (root, folder) = folder();

            write_atomic(&folder, "settings.json", b"first").unwrap();
            write_atomic(&folder, "settings.json", b"second").unwrap();

            assert_eq!(read_file(&folder, "settings.json").unwrap().unwrap(), b"second");
            assert_eq!(names(root.path()), ["settings.json"]);
        }

        #[test]
        fn a_temporary_file_is_named_after_its_target() {
            let (root, folder) = folder();

            let temporary = temporary_file(&folder, "settings.json").unwrap();

            let path = temporary.path();
            assert_eq!(path.parent(), Some(root.path()));
            assert!(path.file_name().unwrap().to_str().unwrap().starts_with(".settings.json."), "{path:?}");
            assert_eq!(path.extension().unwrap(), "tmp");
        }

        #[test]
        fn the_written_file_is_private_to_the_user() {
            let (root, folder) = folder();

            write_atomic(&folder, "settings.json", b"{}").unwrap();

            let mode = fs::metadata(root.path().join("settings.json")).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }
}
