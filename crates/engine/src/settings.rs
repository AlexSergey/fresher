//! The settings in memory: loaded once through the `SettingsStore` port, resolved against the schema, changed one
//! value at a time and saved through the port before the change counts.

use std::path::PathBuf;

use fresher_core::{
    CommandError, Event, ModuleId, ModuleSettings, ResolvedSettings, SettingProblem, SettingUpdate, SettingValue,
    SettingsFile, SettingsSchema, SettingsStore, SettingsView, StoreError, StoredSettings,
};

/// The core's setting of the folders a scan walks.
const SCAN_ROOTS: &str = "core.scan.roots";
/// The switch the schema adds for every module: `<module>.enabled`.
const ENABLED: &str = "enabled";

/// The schema, the stored values as read, and the values in use.
#[derive(Debug)]
pub(crate) struct Settings {
    schema: SettingsSchema,
    stored: StoredSettings,
    resolved: ResolvedSettings,
    file: SettingsFile,
}

impl Settings {
    /// Loads the stored values; whatever cannot be read falls back to the defaults.
    pub(crate) fn load(schema: SettingsSchema, store: &dyn SettingsStore) -> Self {
        let (stored, file) = match store.load() {
            Ok(stored) => (stored, SettingsFile::Loaded),
            Err(error) => {
                tracing::warn!(%error, "the settings file is not used; the defaults are");
                let file = match error {
                    StoreError::NewerVersion { .. } => SettingsFile::NewerVersion,
                    StoreError::Broken => SettingsFile::Broken,
                    StoreError::Io { .. } | StoreError::TooLarge { .. } => SettingsFile::Unreadable,
                };
                (StoredSettings::default(), file)
            }
        };
        let resolved = schema.resolve(&stored);
        Self { schema, stored, resolved, file }
    }

    pub(crate) fn view(&self) -> SettingsView {
        SettingsView::new(&self.schema, &self.resolved, self.file)
    }

    /// Checks `update`, saves it through `store`, then takes it; the event announces it. Nothing changes on a refusal.
    pub(crate) fn update(&mut self, update: SettingUpdate, store: &dyn SettingsStore) -> Result<Event, CommandError> {
        let SettingUpdate { key, value } = update;
        let descriptor = self.schema.get(&key).ok_or(CommandError::UnknownSetting { key })?;
        let key = descriptor.key().clone();
        descriptor.check(&value).map_err(|_| CommandError::InvalidSettingValue { key: key.clone() })?;
        if self.file.is_read_only() {
            return Err(CommandError::SettingsReadOnly);
        }
        let mut stored = self.stored.clone();
        stored.values.insert(key.as_str().into(), value.to_json());
        store.save(&stored).map_err(|error| {
            tracing::warn!(%error, key = key.as_str(), "a setting could not be saved");
            CommandError::Storage
        })?;
        self.stored = stored;
        self.file = SettingsFile::Loaded;
        self.resolved.problems.retain(|problem| problem_key(problem) != key.as_str());
        self.resolved.values.insert(key.clone(), value.clone());
        Ok(Event::SettingChanged { key, value })
    }

    /// The folders a scan walks, as written (`~` is filled in by the walk's adapter).
    pub(crate) fn scan_roots(&self) -> Vec<PathBuf> {
        match self.resolved.get(SCAN_ROOTS) {
            Some(SettingValue::Folders(roots)) => roots.clone(),
            _ => Vec::new(),
        }
    }

    pub(crate) fn module(&self, module: ModuleId) -> ModuleSettings {
        self.resolved.module(module)
    }

    pub(crate) fn is_enabled(&self, module: ModuleId) -> bool {
        self.module(module).switch(ENABLED).unwrap_or(true)
    }
}

fn problem_key(problem: &SettingProblem) -> &str {
    match problem {
        SettingProblem::UnknownKey { key } => key,
        SettingProblem::WrongType { key }
        | SettingProblem::OutOfRange { key, .. }
        | SettingProblem::UnknownChoice { key }
        | SettingProblem::RefusedFolder { key, .. } => key.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use fresher_core::{SettingKey, SettingUpdate, SettingValue};

    use crate::Engine;
    use crate::testing::{Fixture, MemorySettings, ScriptedWalk, TestModule};

    /// An engine with module `alpha` over `settings`, and the settings to look at afterwards.
    fn engine(settings: MemorySettings) -> (Engine, Arc<MemorySettings>) {
        let fixture = Fixture::new(ScriptedWalk::new()).module(TestModule::builder("alpha")).settings(settings);
        let settings = Arc::clone(&fixture.settings);
        (fixture.engine(), settings)
    }

    fn key(key: &str) -> SettingKey {
        SettingKey::new(key).unwrap()
    }

    fn update(key: &str, value: SettingValue) -> SettingUpdate {
        SettingUpdate { key: key.into(), value }
    }

    fn roots(engine: &Engine) -> Option<SettingValue> {
        engine.settings().unwrap().values.get(&key("core.scan.roots")).cloned()
    }

    fn home() -> SettingValue {
        SettingValue::Folders(vec![PathBuf::from("~")])
    }

    mod negative {
        use std::io;

        use fresher_core::{CommandError, FormatVersion, SettingProblem, SettingValue, StoreError};
        use serde_json::json;

        use super::{engine, home, key, roots, update};
        use crate::testing::{MemorySettings, poison};
        use fresher_core::SettingsFile;

        #[test]
        fn refuses_an_unknown_setting() {
            let (engine, settings) = engine(MemorySettings::default());

            let error = engine.update_setting(update("alpha.missing", SettingValue::Switch(true)));

            assert_eq!(error, Err(CommandError::UnknownSetting { key: "alpha.missing".into() }));
            assert_eq!(settings.saves(), 0);
        }

        #[test]
        fn refuses_a_value_of_the_wrong_type() {
            let (engine, settings) = engine(MemorySettings::default());

            let error = engine.update_setting(update("alpha.enabled", SettingValue::Number(1)));

            assert_eq!(error, Err(CommandError::InvalidSettingValue { key: key("alpha.enabled") }));
            assert_eq!(settings.saves(), 0);
        }

        #[test]
        fn refuses_a_protected_or_relative_scan_root() {
            let (engine, settings) = engine(MemorySettings::default());

            for folder in ["/System", "/", "~/../..", "Projects"] {
                let error =
                    engine.update_setting(update("core.scan.roots", SettingValue::Folders(vec![folder.into()])));
                assert_eq!(error, Err(CommandError::InvalidSettingValue { key: key("core.scan.roots") }), "{folder}");
            }
            assert_eq!(settings.saves(), 0);
            assert_eq!(roots(&engine), Some(home()));
        }

        #[test]
        fn a_stored_protected_scan_root_is_a_problem_and_the_default_is_used() {
            let (engine, _) = engine(MemorySettings::with(&[("core.scan.roots", json!(["~/Projects", "/usr/bin"]))]));

            let problem = SettingProblem::RefusedFolder { key: key("core.scan.roots"), folder: "/usr/bin".into() };
            assert_eq!(engine.settings().unwrap().problems, [problem]);
            assert_eq!(roots(&engine), Some(home()));
        }

        #[test]
        fn keeps_working_on_defaults_when_the_file_is_newer() {
            let newer = StoreError::NewerVersion { found: FormatVersion::new(2), supported: FormatVersion::new(1) };
            let (engine, _) = engine(MemorySettings::unloadable(newer));

            assert_eq!(engine.settings().map(|view| view.file), Ok(SettingsFile::NewerVersion));
            assert!(engine.settings().unwrap().file.is_read_only());
            assert_eq!(roots(&engine), Some(home()));
        }

        #[test]
        fn never_saves_when_the_file_is_newer() {
            let newer = StoreError::NewerVersion { found: FormatVersion::new(2), supported: FormatVersion::new(1) };
            let (engine, settings) = engine(MemorySettings::unloadable(newer));
            let events = engine.subscribe();

            let error = engine.update_setting(update("alpha.enabled", SettingValue::Switch(false)));

            assert_eq!(error, Err(CommandError::SettingsReadOnly));
            assert_eq!(settings.saves(), 0);
            assert_eq!(events.try_iter().count(), 0);
        }

        #[test]
        fn never_saves_when_the_file_could_not_be_read() {
            let unreadable = StoreError::Io { kind: io::ErrorKind::PermissionDenied };
            let (engine, settings) = engine(MemorySettings::unloadable(unreadable));
            let events = engine.subscribe();

            let error = engine.update_setting(update("alpha.enabled", SettingValue::Switch(false)));

            assert_eq!(error, Err(CommandError::SettingsReadOnly));
            assert!(engine.settings().unwrap().file.is_read_only());
            assert_eq!(settings.saves(), 0);
            assert_eq!(events.try_iter().count(), 0);
        }

        #[test]
        fn keeps_the_old_value_when_saving_fails() {
            let failing = MemorySettings::default().failing_saves(StoreError::Io { kind: io::ErrorKind::StorageFull });
            let (engine, settings) = engine(failing);
            let events = engine.subscribe();

            let error =
                engine.update_setting(update("core.scan.roots", SettingValue::Folders(vec!["/Volumes".into()])));

            assert_eq!(error, Err(CommandError::Storage));
            assert_eq!(settings.saves(), 1);
            assert_eq!(roots(&engine), Some(home()));
            assert_eq!(events.try_iter().count(), 0);
        }

        #[test]
        fn a_broken_or_unreadable_file_gives_the_defaults() {
            let (broken, _) = engine(MemorySettings::unloadable(StoreError::Broken));
            let unreadable = StoreError::Io { kind: io::ErrorKind::PermissionDenied };
            let (unreadable, _) = engine(MemorySettings::unloadable(unreadable));

            assert_eq!(broken.settings().map(|view| view.file), Ok(SettingsFile::Broken));
            assert_eq!(unreadable.settings().map(|view| view.file), Ok(SettingsFile::Unreadable));
            assert_eq!(roots(&broken), Some(home()));
            assert!(!broken.settings().unwrap().file.is_read_only());
        }

        #[test]
        fn a_broken_value_falls_back_to_its_default_with_a_problem() {
            let (engine, _) = engine(MemorySettings::with(&[("alpha.enabled", json!("yes")), ("gone.key", json!(1))]));

            let view = engine.settings().unwrap();

            assert_eq!(view.values.get(&key("alpha.enabled")), Some(&SettingValue::Switch(true)));
            assert_eq!(
                view.problems,
                [
                    SettingProblem::UnknownKey { key: "gone.key".into() },
                    SettingProblem::WrongType { key: key("alpha.enabled") }
                ]
            );
        }

        #[test]
        fn a_poisoned_settings_lock_is_an_internal_error() {
            let (engine, _) = engine(MemorySettings::default());
            poison(&engine.shared.settings);

            assert_eq!(engine.settings(), Err(CommandError::Internal));
            assert_eq!(
                engine.update_setting(update("alpha.enabled", SettingValue::Switch(false))),
                Err(CommandError::Internal)
            );
            assert_eq!(engine.start_scan(crate::ScanMode::Auto), Err(CommandError::Internal));
        }
    }

    mod positive {
        use fresher_core::{Event, ModuleId, ScanId, SettingProblem, SettingValue, StoreError};
        use serde_json::json;

        use super::{engine, key, update};
        use crate::testing::{MemorySettings, scan};
        use fresher_core::SettingsFile;

        #[test]
        fn persists_and_announces_a_change() {
            let (engine, settings) = engine(MemorySettings::with(&[("gone.key", json!(1))]));
            let events = engine.subscribe();

            engine.update_setting(update("alpha.enabled", SettingValue::Switch(false))).unwrap();

            let stored = settings.stored();
            assert_eq!(stored.values.get("alpha.enabled"), Some(&json!(false)));
            assert_eq!(stored.values.get("gone.key"), Some(&json!(1)));
            let changed = Event::SettingChanged { key: key("alpha.enabled"), value: SettingValue::Switch(false) };
            assert_eq!(events.try_iter().collect::<Vec<_>>(), [changed]);
            let view = engine.settings().unwrap();
            assert_eq!(view.values.get(&key("alpha.enabled")), Some(&SettingValue::Switch(false)));
        }

        #[test]
        fn a_disabled_module_is_left_out_of_the_next_scan() {
            let (engine, _) = engine(MemorySettings::default());
            let alpha = ModuleId::new("alpha").unwrap();

            let first = scan(&engine);
            engine.update_setting(update("alpha.enabled", SettingValue::Switch(false))).unwrap();
            let second = scan(&engine);

            assert_eq!(first.first(), Some(&Event::ScanStarted { scan: ScanId::new(1), modules: vec![alpha] }));
            assert_eq!(second.first(), Some(&Event::ScanStarted { scan: ScanId::new(2), modules: vec![] }));
        }

        #[test]
        fn a_change_clears_the_problem_of_its_key() {
            let (engine, _) = engine(MemorySettings::with(&[("alpha.enabled", json!("yes")), ("gone.key", json!(1))]));

            engine.update_setting(update("alpha.enabled", SettingValue::Switch(false))).unwrap();

            assert_eq!(engine.settings().unwrap().problems, [SettingProblem::UnknownKey { key: "gone.key".into() }]);
        }

        #[test]
        fn saving_over_a_broken_file_makes_it_loaded() {
            let (engine, settings) = engine(MemorySettings::unloadable(StoreError::Broken));

            engine.update_setting(update("alpha.enabled", SettingValue::Switch(false))).unwrap();

            assert_eq!(engine.settings().map(|view| view.file), Ok(SettingsFile::Loaded));
            assert_eq!(settings.stored().values.len(), 1);
        }

        #[test]
        fn lists_every_setting_of_the_schema() {
            let (engine, _) = engine(MemorySettings::default());

            let view = engine.settings().unwrap();

            let keys: Vec<_> = view.settings.iter().map(|setting| setting.key().as_str()).collect();
            assert_eq!(keys, ["core.scan.roots", "alpha.enabled"]);
            assert_eq!(view.values.len(), 2);
            assert_eq!(engine.settings().map(|view| view.file), Ok(SettingsFile::Loaded));
        }
    }
}
