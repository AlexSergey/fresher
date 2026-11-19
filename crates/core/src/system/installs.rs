//! The system's installer receipts and the version of macOS: what proves an installer belongs to an installed app, or
//! that the running macOS has passed an old one.

use crate::{SystemError, Timestamp};

/// One installer receipt: the package the system installed, and when.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    /// The file name of the package that was installed (`Tool-3.1.pkg`).
    pub package_file: String,
    /// When the system recorded the install, in whole seconds.
    pub installed: Timestamp,
}

/// A macOS version: `15.4` is major 15, minor 4. Versions compare as `(major, minor)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemVersion {
    pub major: u16,
    pub minor: u16,
}

impl SystemVersion {
    /// Reads `15`, `15.4` or `10.15`; a third part is ignored. Anything else is `None`.
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.trim().split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = match parts.next() {
            Some(minor) => minor.parse().ok()?,
            None => 0,
        };
        Some(Self { major, minor })
    }
}

/// The installer receipts and the version of the running macOS.
pub trait InstallRecords: Send + Sync {
    /// The receipts of the packages the system installed. An error when they cannot be read: then nothing is known.
    fn receipts(&self) -> Result<Vec<Receipt>, SystemError>;

    /// The version of the running macOS.
    fn system_version(&self) -> Result<SystemVersion, SystemError>;
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::SystemVersion;

        #[test]
        fn a_version_that_is_not_numbers_is_refused() {
            for text in ["", "x", "15.", "15.x"] {
                assert_eq!(SystemVersion::parse(text), None, "{text:?}");
            }
        }
    }

    mod positive {
        use crate::SystemVersion;

        #[test]
        fn versions_parse_and_compare_as_major_then_minor() {
            let parse = |text| SystemVersion::parse(text).unwrap();

            assert!(parse("10.15") < parse("11.0"));
            assert!(parse("11.0") < parse("15.4"));
            assert!(parse("15.4") < parse("26.0"));
            assert_eq!(parse("15"), parse("15.0"));
            assert_eq!(parse("15.4.1"), parse("15.4"));
        }
    }
}
