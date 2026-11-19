//! The macOS installers by bundle name, as data: `macos.toml`, parsed and checked when the module is built.

use fresher_core::SystemVersion;
use serde::Deserialize;

/// The table shipped with the module.
const SHIPPED: &str = include_str!("../macos.toml");

/// Why the macOS installers table was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum MacosError {
    #[error("the macOS installers table is not valid TOML: {0}")]
    Format(String),
    #[error("the macOS installers table lists nothing")]
    Empty,
    #[error("`{name}` is not `Install macOS ….app` or `Install OS X ….app`")]
    BadName { name: String },
    #[error("`{name}` is listed twice")]
    Duplicate { name: String },
    #[error("`{version}` of `{name}` is not `<major>.<minor>`")]
    BadVersion { name: String, version: String },
}

/// The macOS installers and the versions they install, checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MacosInstallers {
    /// Names lowercased.
    installers: Vec<(String, SystemVersion)>,
}

impl MacosInstallers {
    pub(crate) fn shipped() -> Result<Self, MacosError> {
        Self::parse(SHIPPED)
    }

    /// Installers read from `text`: at least one; each name `Install macOS ….app` or `Install OS X ….app` with
    /// something between, none twice (case ignored); each version `<major>.<minor>`.
    pub(crate) fn parse(text: &str) -> Result<Self, MacosError> {
        let table: RawTable = toml::from_str(text).map_err(|error| MacosError::Format(error.to_string()))?;
        if table.installer.is_empty() {
            return Err(MacosError::Empty);
        }
        let mut installers: Vec<(String, SystemVersion)> = Vec::with_capacity(table.installer.len());
        for raw in table.installer {
            if !is_installer_name(&raw.name) {
                return Err(MacosError::BadName { name: raw.name });
            }
            let folded = raw.name.to_ascii_lowercase();
            if installers.iter().any(|(known, _)| *known == folded) {
                return Err(MacosError::Duplicate { name: raw.name });
            }
            let version = strict_version(&raw.version)
                .ok_or_else(|| MacosError::BadVersion { name: raw.name.clone(), version: raw.version.clone() })?;
            installers.push((folded, version));
        }
        Ok(Self { installers })
    }

    /// The version the installer named `name` installs; the case is ignored.
    pub(crate) fn version_of(&self, name: &str) -> Option<SystemVersion> {
        let folded = name.to_ascii_lowercase();
        self.installers.iter().find(|(known, _)| *known == folded).map(|(_, version)| *version)
    }
}

/// `Install macOS ….app` or `Install OS X ….app`, with a name between.
fn is_installer_name(name: &str) -> bool {
    let folded = name.to_ascii_lowercase();
    let Some(stem) = folded.strip_suffix(".app") else { return false };
    ["install macos ", "install os x "]
        .iter()
        .any(|head| stem.strip_prefix(head).is_some_and(|rest| !rest.trim().is_empty()))
}

/// `<major>.<minor>`, two plain numbers: `SystemVersion::parse` alone would take `14` and `14.0.1` too.
fn strict_version(text: &str) -> Option<SystemVersion> {
    let (major, minor) = text.split_once('.')?;
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    if !digits(major) || !digits(minor) {
        return None;
    }
    SystemVersion::parse(text)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTable {
    #[serde(default)]
    installer: Vec<RawInstaller>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawInstaller {
    name: String,
    version: String,
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fmt::Write as _;

        use crate::macos::{MacosError, MacosInstallers};

        fn table(rows: &[(&str, &str)]) -> String {
            let mut text = String::new();
            for (name, version) in rows {
                writeln!(
                    text,
                    "[[installer]]
name = \"{name}\"
version = \"{version}\"
"
                )
                .expect("a string takes text");
            }
            text
        }

        #[test]
        fn text_that_is_not_toml_is_refused() {
            assert!(matches!(MacosInstallers::parse("[[installer"), Err(MacosError::Format(_))));
        }

        #[test]
        fn an_unknown_field_is_refused() {
            let text = "[[installer]]\nname = \"Install macOS Sonoma.app\"\nversion = \"14.0\"\nwhy = 1\n";
            assert!(matches!(MacosInstallers::parse(text), Err(MacosError::Format(_))));
        }

        #[test]
        fn a_table_with_no_installer_is_refused() {
            assert_eq!(MacosInstallers::parse(""), Err(MacosError::Empty));
        }

        #[test]
        fn a_name_that_is_not_an_installer_bundle_is_refused() {
            for name in ["Install Sonoma.app", "Install macOS Sonoma", "Install macOS .app", "Sonoma.app"] {
                let text = table(&[(name, "14.0")]);
                assert_eq!(MacosInstallers::parse(&text), Err(MacosError::BadName { name: name.into() }), "{name:?}");
            }
        }

        #[test]
        fn a_name_listed_twice_in_another_case_is_refused() {
            let text = table(&[("Install macOS Sonoma.app", "14.0"), ("install macos sonoma.APP", "14.0")]);
            assert_eq!(
                MacosInstallers::parse(&text),
                Err(MacosError::Duplicate { name: "install macos sonoma.APP".into() })
            );
        }

        #[test]
        fn a_version_that_is_not_major_dot_minor_is_refused() {
            for version in ["14", "fourteen", "14.0.1", "14.", ".0", "14.x", ""] {
                let text = table(&[("Install macOS Sonoma.app", version)]);
                let expected =
                    MacosError::BadVersion { name: "Install macOS Sonoma.app".into(), version: version.into() };
                assert_eq!(MacosInstallers::parse(&text), Err(expected), "{version:?}");
            }
        }
    }

    mod positive {
        use fresher_core::SystemVersion;

        use crate::macos::MacosInstallers;

        #[test]
        fn install_macos_sonoma_is_version_14_in_any_case() {
            let installers = MacosInstallers::shipped().expect("the shipped table is valid");

            let expected = Some(SystemVersion { major: 14, minor: 0 });
            assert_eq!(installers.version_of("Install macOS Sonoma.app"), expected);
            assert_eq!(installers.version_of("INSTALL MACOS SONOMA.APP"), expected);
            assert_eq!(installers.version_of("Install macOS Nowhere.app"), None);
        }

        #[test]
        fn the_shipped_table_reaches_from_high_sierra_to_tahoe() {
            let installers = MacosInstallers::shipped().expect("the shipped table is valid");

            assert_eq!(
                installers.version_of("Install macOS High Sierra.app"),
                Some(SystemVersion { major: 10, minor: 13 })
            );
            assert_eq!(installers.version_of("Install macOS Big Sur.app"), Some(SystemVersion { major: 11, minor: 0 }));
            assert_eq!(installers.version_of("Install macOS Tahoe.app"), Some(SystemVersion { major: 26, minor: 0 }));
        }
    }
}
