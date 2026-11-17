//! The settings in memory: loaded once through the `SettingsStore` port, resolved against the schema, changed one
//! value at a time and saved through the port before the change counts.

use std::path::{Path, PathBuf};

use fresher_core::{
    CategoryId, CommandError, Event, Finding, IGNORE_CATEGORIES, IGNORE_GROUPS, IGNORE_MODULES, IGNORE_PATHS,
    IgnoredEntry, ModuleGroup, ModuleId, ModuleSettings, ResolvedSettings, SCAN_ROOTS, SettingDescriptor,
    SettingEffect, SettingKey, SettingUpdate, SettingValue, SettingsFile, SettingsSchema, SettingsStore, SettingsView,
    StoreError, StoredSettings, Subject,
};

use crate::Registry;

/// The switch the schema adds for every module: `<module>.enabled`.
const ENABLED: &str = "enabled";

/// The ignore lists: a reset keeps them, and a change of one kills the pending cleanup plan.
const IGNORE_LISTS: [&str; 4] = [IGNORE_MODULES, IGNORE_GROUPS, IGNORE_CATEGORIES, IGNORE_PATHS];

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

    /// How the settings file was read: only a `Loaded` file tells what the user ignores.
    pub(crate) const fn file(&self) -> SettingsFile {
        self.file
    }

    /// Checks `update`, saves it through `store`, then takes it; the event announces it. Nothing changes on a refusal.
    pub(crate) fn update(&mut self, update: SettingUpdate, store: &dyn SettingsStore) -> Result<Event, CommandError> {
        let SettingUpdate { key, value } = update;
        let descriptor = self.schema.get(&key).ok_or(CommandError::UnknownSetting { key })?;
        let key = descriptor.key().clone();
        descriptor.check(&value).map_err(|_| CommandError::InvalidSettingValue { key: key.clone() })?;
        let mut stored = self.stored.clone();
        stored.values.insert(key.as_str().into(), value.to_json());
        self.save(stored, store)?;
        self.take(&key, &value);
        Ok(Event::SettingChanged { key, value })
    }

    /// Puts every setting but the ignore lists back to its default and drops every other stored key, saved once;
    /// answers one `SettingChanged` per value that changed. Nothing is saved when nothing would change; refused on a
    /// read-only file even then, so the screen never says the settings were reset when they could not be.
    pub(crate) fn reset(&mut self, store: &dyn SettingsStore) -> Result<Vec<Event>, CommandError> {
        if self.file.is_read_only() {
            return Err(CommandError::SettingsReadOnly);
        }
        let kept = |key: &str| IGNORE_LISTS.contains(&key);
        let changed: Vec<(SettingKey, SettingValue)> = self
            .schema
            .descriptors()
            .iter()
            .filter(|setting| !kept(setting.key().as_str()))
            .filter(|setting| self.resolved.get(setting.key().as_str()) != Some(setting.default()))
            .map(|setting| (setting.key().clone(), setting.default().clone()))
            .collect();
        let mut stored = self.stored.clone();
        stored.values.retain(|key, _| kept(key));
        if changed.is_empty() && stored == self.stored {
            return Ok(Vec::new());
        }
        self.save(stored, store)?;
        self.resolved.problems.retain(|problem| kept(problem.key()));
        Ok(changed
            .into_iter()
            .map(|(key, value)| {
                self.take(&key, &value);
                Event::SettingChanged { key, value }
            })
            .collect())
    }

    /// The ignore lists in use; the modules they hide are resolved through `registry`: the ignored modules it has,
    /// and every module of an ignored group.
    pub(crate) fn ignored(&self, registry: &Registry) -> Ignored {
        let modules = self.ids(IGNORE_MODULES);
        let groups: Vec<ModuleGroup> =
            self.ids(IGNORE_GROUPS).iter().filter_map(|name| ModuleGroup::parse(name)).collect();
        let named = modules.iter().filter_map(|module| registry.module_id(module));
        let grouped = groups.iter().flat_map(|&group| registry.modules_of(group));
        let mut hidden_modules: Vec<ModuleId> = named.chain(grouped).collect();
        hidden_modules.sort_unstable();
        hidden_modules.dedup();
        Ignored {
            categories: self.ids(IGNORE_CATEGORIES),
            paths: self.ignored_paths(),
            modules,
            groups,
            hidden_modules,
        }
    }

    /// The ignored paths in use.
    pub(crate) fn ignored_paths(&self) -> Vec<PathBuf> {
        match self.resolved.get(IGNORE_PATHS) {
            Some(SettingValue::Paths(paths)) => paths.clone(),
            _ => Vec::new(),
        }
    }

    /// The ids of the list `key` in use.
    fn ids(&self, key: &str) -> Vec<String> {
        match self.resolved.get(key) {
            Some(SettingValue::Ids(ids)) => ids.clone(),
            _ => Vec::new(),
        }
    }

    /// Adds `entry` to its ignore list (`true`) or takes it out (`false`), through `update`; `None` when the list
    /// already is that way.
    pub(crate) fn set_ignored(
        &mut self,
        entry: &IgnoredEntry,
        ignored: bool,
        store: &dyn SettingsStore,
    ) -> Result<Option<Event>, CommandError> {
        let (key, id) = match entry {
            IgnoredEntry::Path { path } => {
                let mut paths = self.ignored_paths();
                if !change_list(&mut paths, path, ignored) {
                    return Ok(None);
                }
                let update = SettingUpdate { key: IGNORE_PATHS.into(), value: SettingValue::Paths(paths) };
                return self.update(update, store).map(Some);
            }
            IgnoredEntry::Category { category } => (IGNORE_CATEGORIES, category.clone()),
            IgnoredEntry::Module { module } => (IGNORE_MODULES, module.clone()),
            IgnoredEntry::Group { group } => (IGNORE_GROUPS, group.as_str().to_owned()),
        };
        let mut ids = self.ids(key);
        if !change_list(&mut ids, &id, ignored) {
            return Ok(None);
        }
        let value = SettingValue::Ids(ids);
        self.update(SettingUpdate { key: key.into(), value }, store).map(Some)
    }

    /// When a change of `key` shows in the results; `None` for a key the schema does not have.
    pub(crate) fn effect(&self, key: &str) -> Option<SettingEffect> {
        self.schema.get(key).map(SettingDescriptor::effect)
    }

    /// Saves `stored` through `store` and keeps it; refused while the file is read-only.
    fn save(&mut self, stored: StoredSettings, store: &dyn SettingsStore) -> Result<(), CommandError> {
        if self.file.is_read_only() {
            return Err(CommandError::SettingsReadOnly);
        }
        store.save(&stored).map_err(|error| {
            tracing::warn!(%error, "the settings could not be saved");
            CommandError::Storage
        })?;
        self.stored = stored;
        self.file = SettingsFile::Loaded;
        Ok(())
    }

    /// Takes a saved value into use; its stored value's problem is gone.
    fn take(&mut self, key: &SettingKey, value: &SettingValue) {
        self.resolved.problems.retain(|problem| problem.key() != key.as_str());
        self.resolved.values.insert(key.clone(), value.clone());
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

    /// Whether the core's switch `key` is on; a key that is not a switch is off.
    pub(crate) fn core_switch(&self, key: &str) -> bool {
        matches!(self.resolved.get(key), Some(SettingValue::Switch(true)))
    }

    pub(crate) fn is_enabled(&self, module: ModuleId) -> bool {
        self.module(module).switch(ENABLED).unwrap_or(true)
    }
}

/// Whether a change of `key` changes what a scan reads: the roots or a module's switch. A pending cleanup plan dies
/// with such a change.
pub(crate) fn changes_what_is_scanned(key: &SettingKey) -> bool {
    let key = key.as_str();
    key == SCAN_ROOTS || key.strip_suffix(ENABLED).is_some_and(|module| module.ends_with('.'))
}

/// Whether `key` is one of the ignore lists: a change of one kills the pending cleanup plan.
pub(crate) fn is_ignore_list(key: &SettingKey) -> bool {
    IGNORE_LISTS.contains(&key.as_str())
}

/// The ignore lists: categories by id, items by path, modules by id, groups; and the modules they hide.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Ignored {
    pub(crate) categories: Vec<String>,
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) modules: Vec<String>,
    pub(crate) groups: Vec<ModuleGroup>,
    /// The registered modules the lists hide: each ignored one, and every module of an ignored group. A module
    /// restored on its own stays here while its group is ignored.
    pub(crate) hidden_modules: Vec<ModuleId>,
}

impl Ignored {
    /// Whether `finding` is left out: its category or its module is ignored, or its path is at or below an ignored
    /// path.
    pub(crate) fn hides(&self, finding: &Finding) -> bool {
        let new = finding.finding();
        match new.subject() {
            Subject::Path(path) => self.hides_place(new.category(), path),
            Subject::External { .. } => self.hides_category(new.category()),
        }
    }

    /// Whether a claim of `path` by `category` is left out: the category or its module is ignored, or `path` is at
    /// or below an ignored path (compared by components).
    pub(crate) fn hides_place(&self, category: CategoryId, path: &Path) -> bool {
        self.hides_category(category) || self.paths.iter().any(|ignored| path.starts_with(ignored))
    }

    /// Whether `category` is ignored, on its own or with its module (or its module's group).
    pub(crate) fn hides_category(&self, category: CategoryId) -> bool {
        self.hidden_modules.contains(&category.module())
            || self.categories.iter().any(|ignored| ignored == category.as_str())
    }
}

/// Adds `item` to `list` (`add`) or removes it; `false` when the list already was that way.
fn change_list<T: PartialEq + Clone>(list: &mut Vec<T>, item: &T, add: bool) -> bool {
    let present = list.contains(item);
    if add && !present {
        list.push(item.clone());
    } else if !add && present {
        list.retain(|kept| kept != item);
    } else {
        return false;
    }
    true
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
            assert_eq!(keys.first(), Some(&"core.motion"));
            assert_eq!(keys.last(), Some(&"alpha.enabled"));
            assert_eq!(view.values.len(), keys.len());
            assert_eq!(engine.settings().map(|view| view.file), Ok(SettingsFile::Loaded));
        }
    }
}
