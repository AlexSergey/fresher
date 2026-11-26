//! The one removal outside a confirmed plan: the app's own stale temporary files in its data folder.
//!
//! An atomic write of the store leaves `.<target>.<random>.tmp` behind when the app dies between creating it and
//! renaming it. Only those are removed: entries directly in the data folder (no recursion), regular files (never a
//! link), named exactly so for a target the store names. They are the app's scratch files, not the user's, so they are
//! deleted for good; the folder is held open through no link and each entry is checked right before it goes. No user
//! data is read, and nothing else in the folder is touched.

use std::ffi::OsStr;
use std::path::Path;

use fresher_core::EntryKind;

use crate::beneath::{self, entries};
use crate::remove::unlink;

/// What a clearing of stale temporary files did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StaleTemporaries {
    /// How many stale files were removed.
    pub removed: usize,
    /// How many stale files could not be removed and stay.
    pub failed: usize,
}

/// Why the data folder could not be cleared.
#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub enum OwnDataError {
    #[error("the data folder could not be opened")]
    Open,
    #[error("the data folder could not be listed")]
    List,
}

/// Removes the stale temporary files of `targets` (the file names the store writes atomically) directly inside
/// `folder`, the app's data folder. A file that cannot be removed is counted and left; the others are still removed.
pub fn remove_stale_temporaries(folder: &Path, targets: &[String]) -> Result<StaleTemporaries, OwnDataError> {
    let held = beneath::open_folder(folder).map_err(|_| OwnDataError::Open)?;
    let listed = entries(&held).map_err(|_| OwnDataError::List)?;
    let mut done = StaleTemporaries::default();
    for entry in listed {
        let Some(identity) = entry.identity else { continue };
        if identity.kind() != EntryKind::File || !is_stale_temporary(&entry.name, targets) {
            continue;
        }
        match unlink(&held, &entry.name, identity) {
            Ok(()) => done.removed += 1,
            Err(_) => done.failed += 1,
        }
    }
    Ok(done)
}

/// Whether `name` is exactly `.<target>.<random>.tmp` for one of `targets`, the random part being one or more ASCII
/// letters or digits.
fn is_stale_temporary(name: &OsStr, targets: &[String]) -> bool {
    let Some(rest) = name.to_str().and_then(|name| name.strip_prefix('.')) else { return false };
    targets.iter().any(|target| {
        rest.strip_prefix(target.as_str())
            .and_then(|rest| rest.strip_prefix('.'))
            .and_then(|rest| rest.strip_suffix(".tmp"))
            .is_some_and(|random| !random.is_empty() && random.chars().all(|char| char.is_ascii_alphanumeric()))
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    /// A temporary folder named without a link in any component (the system's temporary folder is behind one).
    pub(crate) struct Root {
        path: PathBuf,
        _folder: TempDir,
    }

    impl Root {
        pub(crate) fn new() -> Self {
            let folder = TempDir::new().unwrap();
            Self { path: fs::canonicalize(folder.path()).unwrap(), _folder: folder }
        }

        pub(crate) fn path(&self) -> &Path {
            &self.path
        }
    }

    fn targets() -> Vec<String> {
        vec!["settings.json".to_owned(), "scan-1.bin".to_owned()]
    }

    fn names(folder: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::{PermissionsExt, symlink};

        use tempfile::TempDir;

        use super::{Root, names, targets};
        use crate::own_data::{OwnDataError, StaleTemporaries, remove_stale_temporaries};

        #[test]
        fn a_link_named_like_a_temporary_file_is_kept() {
            let root = Root::new();
            let outside = TempDir::new().unwrap();
            fs::write(outside.path().join("user.txt"), b"mine").unwrap();
            symlink(outside.path().join("user.txt"), root.path().join(".settings.json.a1B2c3.tmp")).unwrap();

            let done = remove_stale_temporaries(root.path(), &targets()).unwrap();

            assert_eq!(done, StaleTemporaries::default());
            assert_eq!(names(root.path()), [".settings.json.a1B2c3.tmp"]);
            assert_eq!(fs::read(outside.path().join("user.txt")).unwrap(), b"mine");
        }

        #[test]
        fn a_folder_named_like_a_temporary_file_is_kept_with_what_is_inside() {
            let root = Root::new();
            let folder = root.path().join(".settings.json.a1B2c3.tmp");
            fs::create_dir(&folder).unwrap();
            fs::write(folder.join("inside.txt"), b"x").unwrap();

            let done = remove_stale_temporaries(root.path(), &targets()).unwrap();

            assert_eq!(done, StaleTemporaries::default());
            assert!(folder.join("inside.txt").exists());
        }

        #[test]
        fn a_file_of_another_name_is_kept() {
            let root = Root::new();
            let kept = [
                "settings.json",
                ".settings.json.tmp",
                ".settings.json..tmp",
                ".other.json.a1B2c3.tmp",
                "settings.json.a1B2c3.tmp",
                ".settings.json.a1B2c3.tmp.bak",
                ".settings.json.a1-2c3.tmp",
                ".settings.json.a1.2c3.tmp",
                ".scan-1.previous.bin.a1B2c3.tmp",
                "notes.txt",
            ];
            for name in kept {
                fs::write(root.path().join(name), b"x").unwrap();
            }

            let done = remove_stale_temporaries(root.path(), &targets()).unwrap();

            assert_eq!(done, StaleTemporaries::default());
            let mut expected = kept.map(str::to_owned).to_vec();
            expected.sort();
            assert_eq!(names(root.path()), expected);
        }

        #[test]
        fn a_file_in_a_subfolder_is_kept() {
            let root = Root::new();
            fs::create_dir(root.path().join("sub")).unwrap();
            fs::write(root.path().join("sub").join(".settings.json.a1B2c3.tmp"), b"x").unwrap();

            let done = remove_stale_temporaries(root.path(), &targets()).unwrap();

            assert_eq!(done, StaleTemporaries::default());
            assert!(root.path().join("sub").join(".settings.json.a1B2c3.tmp").exists());
        }

        #[test]
        fn a_missing_folder_is_an_error_and_nothing_is_created() {
            let root = Root::new();

            let result = remove_stale_temporaries(&root.path().join("missing"), &targets());

            assert_eq!(result, Err(OwnDataError::Open));
            assert_eq!(names(root.path()), Vec::<String>::new());
        }

        #[test]
        fn a_link_in_place_of_the_folder_is_not_followed() {
            let root = Root::new();
            let real = root.path().join("real");
            fs::create_dir(&real).unwrap();
            fs::write(real.join(".settings.json.a1B2c3.tmp"), b"x").unwrap();
            symlink(&real, root.path().join("data")).unwrap();

            let result = remove_stale_temporaries(&root.path().join("data"), &targets());

            assert_eq!(result, Err(OwnDataError::Open));
            assert!(real.join(".settings.json.a1B2c3.tmp").exists());
        }

        #[test]
        fn a_folder_that_cannot_be_read_is_an_error_not_a_panic() {
            let root = Root::new();
            let locked = root.path().join("locked");
            fs::create_dir(&locked).unwrap();
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

            let result = remove_stale_temporaries(&locked, &targets());

            fs::set_permissions(&locked, fs::Permissions::from_mode(0o700)).unwrap();
            assert!(result.is_err());
        }
    }

    mod positive {
        use std::fs;

        use super::{Root, names, targets};
        use crate::own_data::{StaleTemporaries, remove_stale_temporaries};

        #[test]
        fn a_stale_temporary_file_of_a_known_target_is_removed() {
            let root = Root::new();
            fs::write(root.path().join("settings.json"), b"{}").unwrap();
            fs::write(root.path().join(".settings.json.a1B2c3.tmp"), b"half").unwrap();
            fs::write(root.path().join(".settings.json.Zz9.tmp"), b"half").unwrap();
            fs::write(root.path().join(".scan-1.bin.QqQqQq.tmp"), b"half").unwrap();

            let done = remove_stale_temporaries(root.path(), &targets()).unwrap();

            assert_eq!(done, StaleTemporaries { removed: 3, failed: 0 });
            assert_eq!(names(root.path()), ["settings.json"]);
        }

        #[test]
        fn an_empty_folder_has_nothing_to_remove() {
            let root = Root::new();

            assert_eq!(remove_stale_temporaries(root.path(), &targets()), Ok(StaleTemporaries::default()));
        }
    }
}
