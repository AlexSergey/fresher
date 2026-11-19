//! `MacInstallRecords`: the installer receipts the system keeps and the version of macOS.

use std::fs;
use std::path::PathBuf;

use fresher_core::{InstallRecords, Receipt, SystemError, SystemVersion, Timestamp};

use crate::bundle::read_capped;
use crate::macos_version::macos_version;
use crate::plist::read_strings;

/// Where the system keeps its installer receipts, under the root.
const RECEIPTS: &str = "var/db/receipts";

/// The most receipts read: more means something is wrong with the folder.
const MAX_RECEIPTS: usize = 10_000;

/// Reads the installer receipts (`<root>/var/db/receipts/*.plist`, the app passes `/`) and the version of macOS. Reads
/// only: never opens a package.
#[derive(Clone, Debug)]
pub struct MacInstallRecords {
    root: PathBuf,
    max_receipts: usize,
}

impl MacInstallRecords {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into(), max_receipts: MAX_RECEIPTS }
    }

    /// A reader that accepts at most `max_receipts` receipts, for the tests.
    #[cfg(test)]
    pub(crate) fn limited(root: impl Into<PathBuf>, max_receipts: usize) -> Self {
        Self { root: root.into(), max_receipts }
    }
}

impl InstallRecords for MacInstallRecords {
    fn receipts(&self) -> Result<Vec<Receipt>, SystemError> {
        let unreadable = || SystemError { reason: "the installer receipts cannot be read".into() };
        let folder = self.root.join(RECEIPTS);
        let mut files: Vec<PathBuf> = fs::read_dir(&folder)
            .map_err(|_| unreadable())?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "plist"))
            .take(self.max_receipts.saturating_add(1))
            .collect();
        if files.len() > self.max_receipts {
            return Err(SystemError { reason: "there are too many installer receipts".into() });
        }
        files.sort();
        Ok(files.iter().filter_map(|file| receipt(file)).collect())
    }

    fn system_version(&self) -> Result<SystemVersion, SystemError> {
        macos_version(&self.root)
            .map(|version| SystemVersion { major: version.major, minor: version.minor })
            .ok_or_else(|| SystemError { reason: "the version of macOS cannot be read".into() })
    }
}

/// The receipt in `file`, or `None` (logged by its file name) when it cannot be read, names no package or has no time.
fn receipt(file: &std::path::Path) -> Option<Receipt> {
    let read = || {
        let strings = read_strings(&read_capped(file)?).ok()?;
        let package_file = strings.get("PackageFileName")?.clone();
        let modified = fs::symlink_metadata(file).ok()?.modified().ok()?;
        Some(Receipt { package_file, installed: Timestamp::from_system_time(modified) })
    };
    let receipt = read();
    if receipt.is_none() {
        tracing::warn!(file = ?file.file_name(), "an installer receipt is skipped");
    }
    receipt
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fs;

        use fresher_core::InstallRecords;

        use crate::MacInstallRecords;
        use crate::plist::fixture::dict_of_strings;

        fn receipts_folder(root: &std::path::Path) -> std::path::PathBuf {
            let folder = root.join("var/db/receipts");
            fs::create_dir_all(&folder).unwrap();
            folder
        }

        #[test]
        fn a_missing_receipts_folder_is_an_error() {
            let root = tempfile::tempdir().unwrap();

            let error = MacInstallRecords::new(root.path()).receipts().unwrap_err();

            assert_eq!(error.reason, "the installer receipts cannot be read");
        }

        #[test]
        fn a_receipt_without_a_package_name_or_unreadable_is_skipped() {
            let root = tempfile::tempdir().unwrap();
            let folder = receipts_folder(root.path());
            fs::write(folder.join("a.plist"), dict_of_strings(&[("PackageIdentifier", "com.example.a")])).unwrap();
            fs::write(folder.join("b.plist"), "not a plist").unwrap();
            fs::write(folder.join("c.bom"), "ignored").unwrap();

            assert_eq!(MacInstallRecords::new(root.path()).receipts(), Ok(Vec::new()));
        }

        #[test]
        fn more_receipts_than_the_limit_is_an_error() {
            let root = tempfile::tempdir().unwrap();
            let folder = receipts_folder(root.path());
            for name in ["a.plist", "b.plist", "c.plist"] {
                fs::write(folder.join(name), dict_of_strings(&[("PackageFileName", "T.pkg")])).unwrap();
            }

            assert!(MacInstallRecords::limited(root.path(), 3).receipts().is_ok());
            assert!(MacInstallRecords::limited(root.path(), 2).receipts().is_err());
        }

        #[test]
        fn a_missing_system_version_is_an_error() {
            let root = tempfile::tempdir().unwrap();

            assert!(MacInstallRecords::new(root.path()).system_version().is_err());
        }
    }

    mod positive {
        use std::fs::{self, File};
        use std::time::{Duration, UNIX_EPOCH};

        use fresher_core::{InstallRecords, Receipt, SystemVersion, Timestamp};

        use crate::MacInstallRecords;
        use crate::macos_version::fixture::system_version;
        use crate::plist::fixture::dict_of_strings;

        fn set_time(path: &std::path::Path, seconds: u64) {
            File::options()
                .write(true)
                .open(path)
                .unwrap()
                .set_modified(UNIX_EPOCH + Duration::from_secs(seconds))
                .unwrap();
        }

        #[test]
        fn receipts_are_read_with_their_package_name_and_time() {
            let root = tempfile::tempdir().unwrap();
            let folder = root.path().join("var/db/receipts");
            fs::create_dir_all(&folder).unwrap();
            let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\"><dict>\
                       <key>PackageFileName</key><string>Tool-3.1.pkg</string></dict></plist>\n";
            fs::write(folder.join("b.plist"), xml).unwrap();
            fs::write(folder.join("a.plist"), dict_of_strings(&[("PackageFileName", "Other.pkg")])).unwrap();
            set_time(&folder.join("a.plist"), 1_000);
            set_time(&folder.join("b.plist"), 2_000);

            let receipts = MacInstallRecords::new(root.path()).receipts().unwrap();

            assert_eq!(
                receipts,
                [
                    Receipt { package_file: "Other.pkg".into(), installed: Timestamp::from_secs(1_000) },
                    Receipt { package_file: "Tool-3.1.pkg".into(), installed: Timestamp::from_secs(2_000) },
                ]
            );
        }

        #[test]
        fn the_system_version_is_read() {
            let root = tempfile::tempdir().unwrap();
            system_version(root.path(), "15.4.1");

            assert_eq!(MacInstallRecords::new(root.path()).system_version(), Ok(SystemVersion { major: 15, minor: 4 }));
        }
    }
}
