//! The version of macOS, read from the system's own version file.

use std::path::Path;

use crate::bundle::read_capped;
use crate::plist::read_strings;

/// A macOS version: `14.5` is major 14, minor 5.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MacosVersion {
    pub major: u16,
    pub minor: u16,
}

/// The version in `ProductVersion` of `<root>/System/Library/CoreServices/SystemVersion.plist` (the app passes `/`);
/// `None` when the file cannot be read or the value is not `<major>[.<minor>[.…]]`. A link is not followed.
pub fn macos_version(root: &Path) -> Option<MacosVersion> {
    let file = root.join("System").join("Library").join("CoreServices").join("SystemVersion.plist");
    let strings = read_strings(&read_capped(&file)?).ok()?;
    parse(strings.get("ProductVersion")?)
}

fn parse(text: &str) -> Option<MacosVersion> {
    let mut parts = text.trim().split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = match parts.next() {
        Some(minor) => minor.parse().ok()?,
        None => 0,
    };
    Some(MacosVersion { major, minor })
}

#[cfg(test)]
pub(crate) mod fixture {
    use std::fs;
    use std::path::Path;

    /// Writes `<root>/System/Library/CoreServices/SystemVersion.plist` with `ProductVersion` = `version`.
    pub(crate) fn system_version(root: &Path, version: &str) {
        let folder = root.join("System/Library/CoreServices");
        fs::create_dir_all(&folder).unwrap();
        let plist = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\"><dict>\
             <key>ProductName</key><string>macOS</string>\
             <key>ProductVersion</key><string>{version}</string></dict></plist>\n"
        );
        fs::write(folder.join("SystemVersion.plist"), plist).unwrap();
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fs;

        use crate::macos_version::fixture::system_version;
        use crate::macos_version::{macos_version, parse};

        #[test]
        fn a_root_without_the_file_has_no_version() {
            let root = tempfile::tempdir().unwrap();

            assert_eq!(macos_version(root.path()), None);
        }

        #[test]
        fn a_broken_file_or_value_has_no_version() {
            let root = tempfile::tempdir().unwrap();
            system_version(root.path(), "fourteen");
            assert_eq!(macos_version(root.path()), None);

            fs::write(root.path().join("System/Library/CoreServices/SystemVersion.plist"), "not a plist").unwrap();
            assert_eq!(macos_version(root.path()), None);
        }

        #[test]
        fn a_value_that_is_not_a_version_is_refused() {
            for text in ["", ".", "14.x", "-1", "x.1", "70000"] {
                assert_eq!(parse(text), None, "{text}");
            }
        }
    }

    mod positive {
        use crate::macos_version::fixture::system_version;
        use crate::macos_version::{MacosVersion, macos_version, parse};

        #[test]
        fn the_version_is_read_from_the_plist() {
            let root = tempfile::tempdir().unwrap();
            system_version(root.path(), "14.5");

            assert_eq!(macos_version(root.path()), Some(MacosVersion { major: 14, minor: 5 }));
        }

        #[test]
        fn a_major_alone_or_a_patch_level_is_read() {
            assert_eq!(parse("13"), Some(MacosVersion { major: 13, minor: 0 }));
            assert_eq!(parse("27.2.1"), Some(MacosVersion { major: 27, minor: 2 }));
        }
    }
}
