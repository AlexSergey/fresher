//! The app's own data folder, tidied when it opens: the stale temporary files of an interrupted write go.

use fresher_cleaner::remove_stale_temporaries;
use fresher_store::{CachePolicy, DataFolder, atomically_written_files};

/// Removes the stale temporary files in `folder`. It reads no user data, only the app's own folder; a failure is
/// logged and never stops the start.
pub(crate) fn clear_stale_temporaries(folder: &DataFolder) {
    match remove_stale_temporaries(folder.path(), &atomically_written_files(CachePolicy::DEFAULT)) {
        Ok(done) if done.failed > 0 => {
            tracing::warn!(
                removed = done.removed,
                failed = done.failed,
                "stale temporary files stay in the data folder"
            );
        }
        Ok(done) if done.removed > 0 => tracing::info!(removed = done.removed, "stale temporary files were removed"),
        Ok(_) => {}
        Err(error) => tracing::warn!(%error, "the data folder's stale temporary files could not be looked for"),
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        use fresher_store::DataFolder;
        use tempfile::TempDir;

        use crate::own_data::clear_stale_temporaries;

        #[test]
        fn a_folder_that_cannot_be_read_ends_without_a_panic_and_changes_nothing() {
            let root = TempDir::new().unwrap();
            let path = fs::canonicalize(root.path()).unwrap().join("data");
            let folder = DataFolder::open(&path).unwrap();
            fs::write(path.join(".settings.json.a1B2c3.tmp"), b"half").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();

            clear_stale_temporaries(&folder);

            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
            assert!(path.join(".settings.json.a1B2c3.tmp").exists());
        }

        #[test]
        fn the_users_files_and_the_stores_own_files_are_kept() {
            let root = TempDir::new().unwrap();
            let path = fs::canonicalize(root.path()).unwrap();
            let folder = DataFolder::open(&path).unwrap();
            for name in ["settings.json", "scan-1.bin", "notes.txt", ".settings.json.tmp"] {
                fs::write(path.join(name), b"x").unwrap();
            }

            clear_stale_temporaries(&folder);

            assert_eq!(fs::read_dir(&path).unwrap().count(), 4);
        }
    }

    mod positive {
        use std::fs;

        use fresher_store::DataFolder;
        use tempfile::TempDir;

        use crate::own_data::clear_stale_temporaries;

        #[test]
        fn a_stale_temporary_file_of_the_settings_and_of_a_cache_slot_is_removed() {
            let root = TempDir::new().unwrap();
            let path = fs::canonicalize(root.path()).unwrap();
            let folder = DataFolder::open(&path).unwrap();
            fs::write(path.join("settings.json"), b"{}").unwrap();
            fs::write(path.join(".settings.json.a1B2c3.tmp"), b"half").unwrap();
            fs::write(path.join(".scan-2.bin.x9Y8z7.tmp"), b"half").unwrap();

            clear_stale_temporaries(&folder);

            assert_eq!(fs::read_dir(&path).unwrap().count(), 1);
            assert!(path.join("settings.json").exists());
        }
    }
}
