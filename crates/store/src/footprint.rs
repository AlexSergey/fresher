//! The app's own footprint: what its data folder takes on disk, against its budget. Measured only, never cleared.

use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use fresher_core::ByteSize;

use crate::data_folder::io_error;
use crate::{DataFolder, DataFolderError};

/// The most the app's own data may take on disk: 50 MB.
pub const FOOTPRINT_BUDGET: ByteSize = ByteSize::new(50_000_000);

/// The size of a `stat` block, the unit of the allocated size.
const BLOCK_SIZE: u64 = 512;

/// What the data folder takes on disk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Footprint {
    /// The allocated size of every file and link in the folder and below.
    pub total: ByteSize,
    pub budget: ByteSize,
    /// How many files and links were counted; folders are walked, not counted.
    pub files: u32,
}

impl Footprint {
    /// The data takes more than its budget.
    pub fn over(&self) -> bool {
        self.total > self.budget
    }
}

impl DataFolder {
    /// Measures the folder: the allocated size of every file below it. A link is counted as a link, never followed.
    pub fn footprint(&self) -> Result<Footprint, DataFolderError> {
        let mut footprint = Footprint { total: ByteSize::ZERO, budget: FOOTPRINT_BUDGET, files: 0 };
        add_folder(self.path(), &mut footprint)?;
        Ok(footprint)
    }
}

fn add_folder(folder: &Path, footprint: &mut Footprint) -> Result<(), DataFolderError> {
    for entry in fs::read_dir(folder).map_err(|error| io_error(&error))? {
        let entry = entry.map_err(|error| io_error(&error))?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(|error| io_error(&error))?;
        if metadata.is_dir() {
            add_folder(&entry.path(), footprint)?;
        } else {
            let allocated = ByteSize::new(metadata.blocks().saturating_mul(BLOCK_SIZE));
            footprint.total = footprint.total.checked_add(allocated).unwrap_or(ByteSize::new(u64::MAX));
            footprint.files = footprint.files.saturating_add(1);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::DataFolder;

    fn folder() -> (TempDir, DataFolder) {
        let root = TempDir::new().unwrap();
        let folder = DataFolder::open(root.path().join("data")).unwrap();
        (root, folder)
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::{PermissionsExt, symlink};

        use fresher_core::ByteSize;

        use super::folder;
        use crate::{DataFolderError, FOOTPRINT_BUDGET, Footprint};

        #[test]
        fn does_not_follow_a_symlink_out_of_the_folder() {
            let (root, folder) = folder();
            let outside = root.path().join("outside");
            fs::create_dir(&outside).unwrap();
            fs::write(outside.join("big"), vec![1_u8; 1_000_000]).unwrap();
            symlink(&outside, folder.path().join("to-a-folder")).unwrap();
            symlink(outside.join("big"), folder.path().join("to-a-file")).unwrap();

            let footprint = folder.footprint().unwrap();

            assert_eq!(footprint.files, 2);
            assert!(footprint.total < ByteSize::new(1_000_000), "{footprint:?}");
        }

        #[test]
        fn an_empty_folder_is_zero() {
            let (_root, folder) = folder();

            let footprint = folder.footprint().unwrap();

            assert_eq!(footprint, Footprint { total: ByteSize::ZERO, budget: FOOTPRINT_BUDGET, files: 0 });
            assert!(!footprint.over());
        }

        #[test]
        fn exactly_the_budget_is_not_over() {
            let footprint = Footprint { total: FOOTPRINT_BUDGET, budget: FOOTPRINT_BUDGET, files: 1 };

            assert!(!footprint.over());
        }

        #[test]
        fn reports_a_folder_it_cannot_read() {
            let (_root, folder) = folder();
            fs::set_permissions(folder.path(), fs::Permissions::from_mode(0o000)).unwrap();

            let result = folder.footprint();

            fs::set_permissions(folder.path(), fs::Permissions::from_mode(0o700)).unwrap();
            assert!(matches!(result, Err(DataFolderError::Io { .. })), "{result:?}");
        }
    }

    mod positive {
        use std::fs;

        use fresher_core::ByteSize;

        use super::folder;
        use crate::{FOOTPRINT_BUDGET, Footprint};

        #[test]
        fn sums_every_file_in_the_folder() {
            let (_root, folder) = folder();
            fs::write(folder.path().join("settings.json"), vec![1_u8; 10_000]).unwrap();
            fs::create_dir(folder.path().join("cache")).unwrap();
            fs::write(folder.path().join("cache").join("scan.json"), vec![1_u8; 20_000]).unwrap();

            let footprint = folder.footprint().unwrap();

            assert_eq!(footprint.files, 2);
            assert!(footprint.total >= ByteSize::new(30_000), "{footprint:?}");
            assert_eq!(footprint.budget, FOOTPRINT_BUDGET);
        }

        #[test]
        fn reports_over_budget() {
            let footprint = Footprint { total: ByteSize::new(50_000_001), budget: FOOTPRINT_BUDGET, files: 3 };

            assert!(footprint.over());
            assert_eq!(FOOTPRINT_BUDGET, ByteSize::new(50_000_000));
        }
    }
}
