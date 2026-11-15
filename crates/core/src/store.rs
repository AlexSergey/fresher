//! The settings file as the core sees it: raw stored values, the file's version and the store port.

use std::collections::BTreeMap;
use std::fmt;
use std::io;

use serde::{Deserialize, Serialize};

/// Reads and writes the settings file. Values come back raw: the `SettingsSchema` checks them.
pub trait SettingsStore: Send + Sync {
    fn load(&self) -> Result<StoredSettings, StoreError>;
    fn save(&self, settings: &StoredSettings) -> Result<(), StoreError>;
}

/// The stored values by key, as plain JSON, not yet checked.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StoredSettings {
    pub values: BTreeMap<String, serde_json::Value>,
}

impl StoredSettings {
    /// The `format` field of the settings file.
    pub const FORMAT: &str = "settings";
    /// The version of the settings file this build reads and writes.
    pub const VERSION: FormatVersion = FormatVersion::new(1);
    /// The settings hold the user's decisions: an older file is migrated, never thrown away.
    pub const POLICY: VersionPolicy = VersionPolicy::Migrate;
}

/// The version of a file the app writes; every file carries one next to its `format`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormatVersion(u32);

impl FormatVersion {
    pub const fn new(version: u32) -> Self {
        Self(version)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for FormatVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// What happens to a file of an older version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VersionPolicy {
    /// It holds the user's decisions or history: migrated forward, one version at a time.
    Migrate,
    /// It can be rebuilt (the scan cache): discarded.
    Discard,
}

/// Why the settings could not be read or written. No path is held: paths are not logged.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum StoreError {
    /// The file is from a newer version of the app: it is never overwritten.
    #[error("the settings file has version {found}, this app supports up to {supported}")]
    NewerVersion { found: FormatVersion, supported: FormatVersion },
    /// The file is not a settings file the app can read.
    #[error("the settings file is broken")]
    Broken,
    #[error("the settings file could not be read or written: {kind}")]
    Io { kind: io::ErrorKind },
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{FormatVersion, StoreError};

        #[test]
        fn a_newer_file_names_both_versions() {
            let error = StoreError::NewerVersion { found: FormatVersion::new(3), supported: FormatVersion::new(1) };

            assert_eq!(error.to_string(), "the settings file has version 3, this app supports up to 1");
        }
    }

    mod positive {
        use serde_json::json;

        use crate::{StoredSettings, VersionPolicy};

        #[test]
        fn the_settings_file_is_version_one_and_migrated() {
            assert_eq!(StoredSettings::FORMAT, "settings");
            assert_eq!(StoredSettings::VERSION.get(), 1);
            assert_eq!(StoredSettings::POLICY, VersionPolicy::Migrate);
        }

        #[test]
        fn stored_values_are_a_plain_object() {
            let stored: StoredSettings = serde_json::from_value(json!({ "demo.enabled": true })).unwrap();

            assert_eq!(stored.values["demo.enabled"], json!(true));
            assert_eq!(serde_json::to_value(&stored).unwrap(), json!({ "demo.enabled": true }));
        }
    }
}
