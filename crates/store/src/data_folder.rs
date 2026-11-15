//! The app's data folder: an absolute, real folder private to the user, and the plain file names inside it.

use std::ffi::OsStr;
use std::fs::{self, DirBuilder, Permissions};
use std::io;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};

/// The mode of a data folder the store creates: only the user can read, write or enter it.
const PRIVATE_MODE: u32 = 0o700;

/// The folder that holds every file the app writes for itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataFolder {
    path: PathBuf,
}

/// Why the data folder or a file in it could not be used. No path is held: paths are not logged.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum DataFolderError {
    #[error("the data folder must be an absolute path")]
    Relative,
    #[error("the data folder is a symbolic link")]
    Symlink,
    #[error("the data folder is not a folder")]
    NotAFolder,
    /// A file name of the store itself that is not one plain name (a separator, `.` or `..`).
    #[error("`{name}` is not a plain file name")]
    BadFileName { name: String },
    #[error("the data folder could not be read or created: {kind}")]
    Io { kind: io::ErrorKind },
}

impl DataFolder {
    /// Opens the data folder at `path`, creating it (and its parents) private to the user when it is missing.
    ///
    /// Refuses a relative path and a path that is itself a symbolic link or not a folder.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, DataFolderError> {
        let path = path.into();
        if !path.is_absolute() {
            return Err(DataFolderError::Relative);
        }
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => Err(DataFolderError::Symlink),
            Ok(metadata) if !metadata.is_dir() => Err(DataFolderError::NotAFolder),
            Ok(_) => Ok(Self { path }),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                create_private(&path)?;
                Ok(Self { path })
            }
            Err(error) => Err(io_error(&error)),
        }
    }

    /// The absolute path of the folder.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The path of the file `name` in the folder; `name` must be one plain file name.
    pub fn file(&self, name: &str) -> Result<PathBuf, DataFolderError> {
        let mut components = Path::new(name).components();
        match (components.next(), components.next()) {
            (Some(Component::Normal(plain)), None) if plain == OsStr::new(name) && !name.contains('\0') => {
                Ok(self.path.join(plain))
            }
            _ => Err(DataFolderError::BadFileName { name: name.to_owned() }),
        }
    }
}

fn create_private(path: &Path) -> Result<(), DataFolderError> {
    DirBuilder::new().recursive(true).mode(PRIVATE_MODE).create(path).map_err(|error| io_error(&error))?;
    // The umask may have taken bits away; the folder ends up exactly private, never wider.
    fs::set_permissions(path, Permissions::from_mode(PRIVATE_MODE)).map_err(|error| io_error(&error))
}

pub(crate) fn io_error(error: &io::Error) -> DataFolderError {
    DataFolderError::Io { kind: error.kind() }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fs;
        use std::os::unix::fs::symlink;

        use tempfile::TempDir;

        use crate::{DataFolder, DataFolderError};

        #[test]
        fn refuses_a_relative_data_folder() {
            assert_eq!(DataFolder::open("dev.macfresher.app"), Err(DataFolderError::Relative));
        }

        #[test]
        fn refuses_a_data_folder_that_is_a_symlink() {
            let root = TempDir::new().unwrap();
            let real = root.path().join("real");
            fs::create_dir(&real).unwrap();
            let link = root.path().join("link");
            symlink(&real, &link).unwrap();

            assert_eq!(DataFolder::open(&link), Err(DataFolderError::Symlink));
        }

        #[test]
        fn refuses_a_data_folder_that_is_a_file() {
            let root = TempDir::new().unwrap();
            let file = root.path().join("file");
            fs::write(&file, b"").unwrap();

            assert_eq!(DataFolder::open(&file), Err(DataFolderError::NotAFolder));
        }

        #[test]
        fn refuses_a_file_name_with_a_separator() {
            let root = TempDir::new().unwrap();
            let folder = DataFolder::open(root.path()).unwrap();

            for name in ["", ".", "..", "a/b", "../settings.json", "/settings.json", "settings.json/", "a\0b"] {
                assert_eq!(folder.file(name), Err(DataFolderError::BadFileName { name: name.to_owned() }), "{name:?}");
            }
        }

        #[test]
        fn reports_a_folder_it_cannot_create() {
            let root = TempDir::new().unwrap();
            let file = root.path().join("file");
            fs::write(&file, b"").unwrap();

            let error = DataFolder::open(file.join("data")).unwrap_err();

            assert!(matches!(error, DataFolderError::Io { .. }), "{error:?}");
        }
    }

    mod positive {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        use tempfile::TempDir;

        use crate::DataFolder;

        #[test]
        fn creates_the_folder_private_to_the_user() {
            let root = TempDir::new().unwrap();
            let path = root.path().join("Application Support").join("dev.macfresher.app");

            let folder = DataFolder::open(&path).unwrap();

            assert_eq!(folder.path(), path);
            assert!(path.is_dir());
            assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o700);
        }

        #[test]
        fn opens_an_existing_folder_as_it_is() {
            let root = TempDir::new().unwrap();
            fs::write(root.path().join("settings.json"), b"{}").unwrap();

            let folder = DataFolder::open(root.path()).unwrap();

            assert_eq!(fs::read(folder.file("settings.json").unwrap()).unwrap(), b"{}");
        }

        #[test]
        fn joins_a_plain_file_name() {
            let root = TempDir::new().unwrap();
            let folder = DataFolder::open(root.path()).unwrap();

            assert_eq!(folder.file("settings.broken.json").unwrap(), root.path().join("settings.broken.json"));
        }
    }
}
