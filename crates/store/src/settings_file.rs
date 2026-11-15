//! `settings.json`: the core's `SettingsStore` on a file in the data folder.

use std::fs;

use fresher_core::{FormatVersion, SettingsStore, StoreError, StoredSettings};
use serde::{Deserialize, Serialize};

use crate::DataFolder;
use crate::atomic::{file_path, io_error, read_file, write_atomic};
use crate::migrate::{Migration, migrate};
use crate::versioned::{Versioned, read_versioned, write_versioned};

const FILE_NAME: &str = "settings.json";
/// Where a broken settings file is kept, once, before the first save replaces it.
const BROKEN_FILE_NAME: &str = "settings.broken.json";
/// The migrations of the settings file, one per version step; version 1 has none.
const MIGRATIONS: &[Migration] = &[];

/// The body of the settings file, next to its `format` and `version`.
#[derive(Deserialize)]
struct SettingsBody {
    values: StoredSettings,
}

/// The body as it is written, borrowing the values.
#[derive(Serialize)]
struct SettingsBodyOut<'a> {
    values: &'a StoredSettings,
}

/// The settings file in the data folder. Values are read and written raw; the core's schema checks them.
///
/// A missing file is empty settings. A broken file is an error and is left as it is until the first save, which
/// keeps it aside as `settings.broken.json`. A file from a newer version of the app is never written over.
#[derive(Debug)]
pub struct FileSettingsStore {
    folder: DataFolder,
    version: FormatVersion,
    migrations: &'static [Migration],
}

impl FileSettingsStore {
    /// The settings file `settings.json` in `folder`, at the version this build writes.
    pub fn new(folder: DataFolder) -> Self {
        Self { folder, version: StoredSettings::VERSION, migrations: MIGRATIONS }
    }

    /// The settings in the file, migrated to the current version; `None` when there is no file.
    fn read(&self) -> Result<Option<StoredSettings>, StoreError> {
        let Some(bytes) = read_file(&self.folder, FILE_NAME)? else {
            return Ok(None);
        };
        let body = match read_versioned(&bytes, StoredSettings::FORMAT, self.version, StoredSettings::POLICY)? {
            Versioned::Current(body) => body,
            Versioned::Older { version, body } => migrate(body, version, self.version, self.migrations)?,
            // The settings are migrated, never discarded: an older file that is not kept is not read.
            Versioned::Discarded { .. } => return Err(StoreError::Broken),
        };
        let body: SettingsBody = serde_json::from_value(body).map_err(|_| StoreError::Broken)?;
        Ok(Some(body.values))
    }

    fn keep_broken_file(&self) -> Result<(), StoreError> {
        let aside = file_path(&self.folder, BROKEN_FILE_NAME)?;
        fs::rename(file_path(&self.folder, FILE_NAME)?, aside).map_err(|error| io_error(&error))
    }
}

impl SettingsStore for FileSettingsStore {
    fn load(&self) -> Result<StoredSettings, StoreError> {
        Ok(self.read()?.unwrap_or_default())
    }

    fn save(&self, settings: &StoredSettings) -> Result<(), StoreError> {
        match self.read() {
            Ok(_) => {}
            Err(StoreError::Broken) => self.keep_broken_file()?,
            Err(error) => return Err(error),
        }
        let bytes = write_versioned(StoredSettings::FORMAT, self.version, &SettingsBodyOut { values: settings })?;
        write_atomic(&self.folder, FILE_NAME, &bytes)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use fresher_core::FormatVersion;
    use serde_json::Value;
    use tempfile::TempDir;

    use super::{FileSettingsStore, Migration};
    use crate::DataFolder;

    struct Folder {
        root: TempDir,
    }

    impl Folder {
        fn new() -> Self {
            Self { root: TempDir::new().unwrap() }
        }

        fn store(&self) -> FileSettingsStore {
            FileSettingsStore::new(DataFolder::open(self.root.path()).unwrap())
        }

        /// A store whose current version is `version`, with its own chain of migrations.
        fn store_at(&self, version: u32, migrations: &'static [Migration]) -> FileSettingsStore {
            FileSettingsStore { version: FormatVersion::new(version), migrations, ..self.store() }
        }

        fn write(&self, name: &str, text: &str) {
            fs::write(self.root.path().join(name), text).unwrap();
        }

        fn read(&self, name: &str) -> String {
            fs::read_to_string(self.root.path().join(name)).unwrap()
        }

        fn names(&self) -> Vec<String> {
            let entries = fs::read_dir(self.root.path()).unwrap();
            let mut names: Vec<String> =
                entries.map(|entry| entry.unwrap().file_name().into_string().unwrap()).collect();
            names.sort();
            names
        }
    }

    fn settings(values: Value) -> fresher_core::StoredSettings {
        serde_json::from_value(values).unwrap()
    }

    mod negative {
        use fresher_core::{FormatVersion, SettingsStore, StoreError};
        use serde_json::json;

        use super::{Folder, settings};
        use crate::migrate::fixtures::FAILING;

        const NEWER: &str = r#"{ "format": "settings", "version": 2, "values": { "demo.new": 1 } }"#;

        #[test]
        fn never_overwrites_a_file_from_a_newer_version() {
            let folder = Folder::new();
            folder.write("settings.json", NEWER);
            let error = StoreError::NewerVersion { found: FormatVersion::new(2), supported: FormatVersion::new(1) };

            assert_eq!(folder.store().load(), Err(error.clone()));
            assert_eq!(folder.store().save(&settings(json!({ "demo.enabled": true }))), Err(error));
            assert_eq!(folder.read("settings.json"), NEWER);
            assert_eq!(folder.names(), ["settings.json"]);
        }

        #[test]
        fn a_broken_file_is_an_error_and_stays_untouched() {
            for broken in ["{ not json", "[]", r#"{ "format": "journal", "version": 1, "values": {} }"#] {
                let folder = Folder::new();
                folder.write("settings.json", broken);

                assert_eq!(folder.store().load(), Err(StoreError::Broken), "{broken}");
                assert_eq!(folder.read("settings.json"), broken);
            }
        }

        #[test]
        fn a_body_without_values_is_broken() {
            for body in
                [r#"{ "format": "settings", "version": 1 }"#, r#"{ "format": "settings", "version": 1, "values": [] }"#]
            {
                let folder = Folder::new();
                folder.write("settings.json", body);

                assert_eq!(folder.store().load(), Err(StoreError::Broken), "{body}");
            }
        }

        #[test]
        fn keeps_the_broken_file_aside_before_the_first_save() {
            let folder = Folder::new();
            folder.write("settings.broken.json", "an older broken file");
            folder.write("settings.json", "{ not json");
            let store = folder.store();
            let saved = settings(json!({ "demo.enabled": false }));

            store.save(&saved).unwrap();
            store.save(&saved).unwrap();

            assert_eq!(folder.read("settings.broken.json"), "{ not json");
            assert_eq!(store.load(), Ok(saved));
            assert_eq!(folder.names(), ["settings.broken.json", "settings.json"]);
        }

        #[test]
        fn a_failing_migration_leaves_the_old_file() {
            let folder = Folder::new();
            let old = r#"{ "format": "settings", "version": 1, "values": { "enabled": true } }"#;
            folder.write("settings.json", old);

            assert_eq!(folder.store_at(2, FAILING).load(), Err(StoreError::Broken));
            assert_eq!(folder.read("settings.json"), old);
        }

        #[test]
        fn reports_a_file_it_cannot_read() {
            let folder = Folder::new();
            std::fs::create_dir(folder.root.path().join("settings.json")).unwrap();

            assert!(matches!(folder.store().load(), Err(StoreError::Io { .. })));
            assert!(matches!(folder.store().save(&settings(json!({}))), Err(StoreError::Io { .. })));
        }
    }

    mod positive {
        use fresher_core::{SettingsStore, StoredSettings};
        use serde_json::json;

        use super::{Folder, settings};
        use crate::migrate::fixtures::TWO_STEPS;
        use crate::settings_file::MIGRATIONS;

        #[test]
        fn a_missing_file_gives_empty_settings() {
            let folder = Folder::new();

            assert_eq!(folder.store().load(), Ok(StoredSettings::default()));
            assert_eq!(folder.names(), Vec::<String>::new());
        }

        #[test]
        fn round_trips_settings() {
            let folder = Folder::new();
            let saved =
                settings(json!({ "core.scan.roots": ["~", "/Volumes/Data"], "demo.enabled": true, "x.y": null }));

            folder.store().save(&saved).unwrap();

            assert_eq!(folder.store().load(), Ok(saved));
        }

        #[test]
        fn migrates_an_old_version_step_by_step() {
            let folder = Folder::new();
            folder.write("settings.json", r#"{ "format": "settings", "version": 1, "values": { "enabled": false } }"#);
            let store = folder.store_at(3, TWO_STEPS);

            let loaded = store.load().unwrap();
            store.save(&loaded).unwrap();

            assert_eq!(loaded, settings(json!({ "demo.enabled": false, "demo.min-age": 30 })));
            assert!(folder.read("settings.json").contains("\"version\": 3"));
            assert_eq!(store.load(), Ok(loaded));
        }

        #[test]
        fn writes_keys_in_sorted_order() {
            let folder = Folder::new();

            folder.store().save(&settings(json!({ "demo.enabled": true, "core.scan.roots": ["~"] }))).unwrap();

            assert_eq!(
                folder.read("settings.json"),
                "{\n  \"format\": \"settings\",\n  \"version\": 1,\n  \"values\": {\n    \"core.scan.roots\": [\n      \
                 \"~\"\n    ],\n    \"demo.enabled\": true\n  }\n}\n"
            );
        }

        #[test]
        fn the_settings_chain_reaches_the_current_version() {
            assert_eq!(MIGRATIONS.len() + 1, usize::try_from(StoredSettings::VERSION.get()).unwrap());
        }
    }
}
