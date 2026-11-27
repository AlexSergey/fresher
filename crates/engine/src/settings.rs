//! The settings in memory: loaded once through the `SettingsStore` port, resolved against the schema, changed one
//! value at a time and saved through the port before the change counts.

use std::path::{Path, PathBuf};

use fresher_core::{
    AGREEMENT_ACCEPTED_AT, AGREEMENT_VERSION_KEY, Acceptance, BlockedReason, ByteSize, CategoryId, CommandError, Event,
    Finding, IGNORE_CATEGORIES, IGNORE_GROUPS, IGNORE_MODULES, IGNORE_PATHS, IgnoredEntry, LARGEST_FOLDER_MIN_SIZE,
    LARGEST_MIN_SIZE, ModuleGroup, ModuleId, ModuleSettings, ResolvedSettings, SCAN_ROOTS, STALE_PROJECTS_MODULE,
    STALE_PROJECTS_UNUSED_FOR, SettingDescriptor, SettingEffect, SettingKey, SettingUpdate, SettingValue, SettingsFile,
    SettingsSchema, SettingsStore, SettingsView, StoreError, StoredSettings, Subject, Timestamp, folder_min_size_of,
    min_size_of,
};

use crate::Registry;

/// The switch the schema adds for every module: `<module>.enabled`.
const ENABLED: &str = "enabled";

/// The ignore lists: a reset keeps them, and a change of one kills the pending cleanup plan.
const IGNORE_LISTS: [&str; 4] = [IGNORE_MODULES, IGNORE_GROUPS, IGNORE_CATEGORIES, IGNORE_PATHS];

/// The acceptance of the agreement: written only by `accept_agreement`, never by `update`.
const AGREEMENT: [&str; 2] = [AGREEMENT_VERSION_KEY, AGREEMENT_ACCEPTED_AT];

/// What a reset keeps: the ignore lists and the acceptance of the agreement.
const KEPT_BY_RESET: [&[&str]; 2] = [&IGNORE_LISTS, &AGREEMENT];

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
    /// The acceptance of the agreement is not a setting to change: its keys are refused as unknown.
    pub(crate) fn update(&mut self, update: SettingUpdate, store: &dyn SettingsStore) -> Result<Event, CommandError> {
        let SettingUpdate { key, value } = update;
        if AGREEMENT.contains(&key.as_str()) {
            return Err(CommandError::UnknownSetting { key });
        }
        let descriptor = self.schema.get(&key).ok_or(CommandError::UnknownSetting { key })?;
        let key = descriptor.key().clone();
        descriptor.check(&value).map_err(|_| CommandError::InvalidSettingValue { key: key.clone() })?;
        let mut stored = self.stored.clone();
        stored.values.insert(key.as_str().into(), value.to_json());
        self.save(stored, store)?;
        self.take(&key, &value);
        Ok(Event::SettingChanged { key, value })
    }

    /// Puts every setting but the ignore lists and the acceptance of the agreement back to its default and drops every
    /// other stored key, saved once (`KEPT_BY_RESET`); answers one `SettingChanged` per value that changed. Nothing is
    /// saved when nothing would change; refused on a read-only file even then, so the screen never says the settings
    /// were reset when they could not be.
    pub(crate) fn reset(&mut self, store: &dyn SettingsStore) -> Result<Vec<Event>, CommandError> {
        if self.file.is_read_only() {
            return Err(CommandError::SettingsReadOnly);
        }
        let kept = |key: &str| KEPT_BY_RESET.iter().any(|keys| keys.contains(&key));
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

    /// The last acceptance of the agreement, of whatever version; `None` when it was never accepted.
    pub(crate) fn agreement(&self) -> Option<Acceptance> {
        let number = |key: &str| match self.resolved.get(key) {
            Some(SettingValue::Number(number)) => *number,
            _ => 0,
        };
        let version = u32::try_from(number(AGREEMENT_VERSION_KEY)).ok().filter(|version| *version > 0)?;
        let at = Timestamp::from_secs(u64::try_from(number(AGREEMENT_ACCEPTED_AT)).unwrap_or_default());
        Some(Acceptance { version, at })
    }

    /// Records the acceptance of `version` at `at`: both values in one save, then takes them. Refused with
    /// `CleaningBlocked` (`Settings`) unless the file is `Loaded`: a save would replace a broken file the user may
    /// still want, and cleaning stays blocked by that file whatever is accepted.
    pub(crate) fn accept_agreement(
        &mut self,
        version: u32,
        at: Timestamp,
        store: &dyn SettingsStore,
    ) -> Result<(), CommandError> {
        if self.file != SettingsFile::Loaded {
            return Err(CommandError::CleaningBlocked { reason: BlockedReason::Settings });
        }
        let at = i64::try_from(at.secs()).map_err(|_| CommandError::Internal)?;
        let mut values = Vec::new();
        for (key, number) in [(AGREEMENT_VERSION_KEY, i64::from(version)), (AGREEMENT_ACCEPTED_AT, at)] {
            let descriptor = self.schema.get(key).ok_or(CommandError::Internal)?;
            values.push((descriptor.key().clone(), SettingValue::Number(number)));
        }
        let mut stored = self.stored.clone();
        for (key, value) in &values {
            stored.values.insert(key.as_str().into(), value.to_json());
        }
        self.save(stored, store)?;
        for (key, value) in &values {
            self.take(key, value);
        }
        Ok(())
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

    /// The smallest size the Largest screen lists; a choice that is not one is the default's.
    pub(crate) fn largest_min_size(&self) -> ByteSize {
        match self.resolved.get(LARGEST_MIN_SIZE) {
            Some(SettingValue::Choice(choice)) => min_size_of(choice),
            _ => min_size_of(""),
        }
    }

    /// The smallest folder the Largest screen's Folders lists; a choice that is not one is the default's.
    pub(crate) fn largest_folder_min_size(&self) -> ByteSize {
        match self.resolved.get(LARGEST_FOLDER_MIN_SIZE) {
            Some(SettingValue::Choice(choice)) => folder_min_size_of(choice),
            _ => folder_min_size_of(""),
        }
    }

    /// The days without a change after which a project is stale: the stale projects module's setting, its default
    /// when the module's id does not parse.
    pub(crate) fn stale_days(&self) -> u32 {
        match ModuleId::new(STALE_PROJECTS_MODULE) {
            Ok(id) => STALE_PROJECTS_UNUSED_FOR.days(&self.module(id)),
            Err(_) => STALE_PROJECTS_UNUSED_FOR.default_days(),
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

    /// The settings of a schema that holds the stale projects' days, over `stored`.
    fn stale_settings(stored: &[(&str, serde_json::Value)]) -> super::Settings {
        use fresher_core::{
            Locales, ModuleDescriptor, ModuleGroup, ModuleId, STALE_PROJECTS_MODULE, STALE_PROJECTS_UNUSED_FOR,
            SettingsSchema,
        };

        let id = ModuleId::new(STALE_PROJECTS_MODULE).unwrap();
        let days = vec![STALE_PROJECTS_UNUSED_FOR.descriptor(id).unwrap()];
        let module = ModuleDescriptor::new(id, ModuleGroup::Development, vec![], days, Locales { en: "{}" }).unwrap();
        let schema = SettingsSchema::new([&module]).unwrap();
        super::Settings::load(schema, &MemorySettings::with(stored))
    }

    mod negative {
        use std::io;

        use fresher_core::{BlockedReason, CommandError, FormatVersion, SettingProblem, SettingValue, StoreError};
        use serde_json::json;

        use super::{engine, home, key, roots, update};
        use crate::testing::{Fixture, MemorySettings, ScriptedWalk, poison};
        use fresher_core::SettingsFile;

        #[test]
        fn refuses_an_unknown_setting() {
            let (engine, settings) = engine(MemorySettings::default());

            let error = engine.update_setting(update("alpha.missing", SettingValue::Switch(true)));

            assert_eq!(error, Err(CommandError::UnknownSetting { key: "alpha.missing".into() }));
            assert_eq!(settings.saves(), 0);
        }

        #[test]
        fn update_refuses_the_agreement_keys_and_saves_nothing() {
            let (engine, settings) = engine(MemorySettings::default());

            for name in ["core.agreement.version", "core.agreement.accepted-at"] {
                let error = engine.update_setting(update(name, SettingValue::Number(0)));
                assert_eq!(error, Err(CommandError::UnknownSetting { key: name.into() }), "{name}");
            }
            assert_eq!(settings.saves(), 0);
            assert!(engine.agreement().unwrap().is_accepted());
        }

        #[test]
        fn accepting_on_a_file_that_is_not_loaded_is_refused_and_saves_nothing() {
            let newer = StoreError::NewerVersion { found: FormatVersion::new(2), supported: FormatVersion::new(1) };
            let unreadable = StoreError::Io { kind: io::ErrorKind::PermissionDenied };
            for error in [StoreError::Broken, newer, unreadable] {
                let fixture =
                    Fixture::new(ScriptedWalk::new()).unaccepted().settings(MemorySettings::unloadable(error.clone()));
                let settings = std::sync::Arc::clone(&fixture.settings);
                let engine = fixture.engine();

                let refused = engine.accept_agreement(fresher_core::AGREEMENT_VERSION);

                let blocked = CommandError::CleaningBlocked { reason: BlockedReason::Settings };
                assert_eq!(refused, Err(blocked), "{error}");
                assert_eq!(settings.saves(), 0, "{error}");
                assert_eq!(engine.agreement().map(|view| view.accepted), Ok(None), "{error}");
                assert_ne!(engine.settings().map(|view| view.file), Ok(SettingsFile::Loaded), "{error}");
            }
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
        fn stale_days_out_of_range_take_the_nearest_limit() {
            let key = key("stale-projects.unused-for-days");
            let mut settings = super::stale_settings(&[]);

            settings.take(&key, &SettingValue::Number(1));
            assert_eq!(settings.stale_days(), 30);
            settings.take(&key, &SettingValue::Number(100_000));
            assert_eq!(settings.stale_days(), 3650);
        }

        #[test]
        fn a_stored_folder_size_outside_the_choices_is_the_default() {
            let (engine, _) = engine(MemorySettings::with(&[("core.largest.folder-min-size", json!("2-gb"))]));

            assert_eq!(engine.settings().unwrap().problems.len(), 1);
            assert_eq!(
                super::super::Settings::load(
                    fresher_core::SettingsSchema::new([]).unwrap(),
                    &MemorySettings::with(&[("core.largest.folder-min-size", json!("2-gb"))])
                )
                .largest_folder_min_size()
                .bytes(),
                1_000_000_000
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
        use crate::testing::{Fixture, MemorySettings, START, ScriptedWalk, scan};
        use fresher_core::SettingsFile;

        #[test]
        fn stale_days_default_to_a_year() {
            assert_eq!(super::stale_settings(&[]).stale_days(), 365);
        }

        #[test]
        fn stale_days_follow_the_stale_projects_setting() {
            let stored = [("stale-projects.unused-for-days", json!(90))];

            assert_eq!(super::stale_settings(&stored).stale_days(), 90);
        }

        #[test]
        fn the_folder_size_follows_its_setting() {
            let sized = |choice: &str| {
                let stored = MemorySettings::with(&[("core.largest.folder-min-size", json!(choice))]);
                let schema = fresher_core::SettingsSchema::new([]).unwrap();
                super::super::Settings::load(schema, &stored).largest_folder_min_size().bytes()
            };
            let unset = super::super::Settings::load(
                fresher_core::SettingsSchema::new([]).unwrap(),
                &MemorySettings::default(),
            );

            assert_eq!(unset.largest_folder_min_size().bytes(), 1_000_000_000);
            assert_eq!(sized("500-mb"), 500_000_000);
            assert_eq!(sized("10-gb"), 10_000_000_000);
        }

        #[test]
        fn reset_keeps_the_acceptance() {
            let (engine, settings) = engine(MemorySettings::with(&[("core.motion", json!("reduced"))]));
            let before = engine.agreement().unwrap();

            engine.reset_settings().unwrap();

            assert_eq!(settings.saves(), 1);
            assert!(before.is_accepted());
            assert_eq!(engine.agreement(), Ok(before));
            let stored = settings.stored();
            assert_eq!(stored.values.get("core.agreement.version"), Some(&json!(fresher_core::AGREEMENT_VERSION)));
            assert_eq!(stored.values.get("core.agreement.accepted-at"), Some(&json!(START)));
        }

        #[test]
        fn acceptance_saves_both_values_once() {
            let fixture = Fixture::new(ScriptedWalk::new()).unaccepted();
            let settings = std::sync::Arc::clone(&fixture.settings);
            let engine = fixture.engine();

            engine.accept_agreement(fresher_core::AGREEMENT_VERSION).unwrap();

            assert_eq!(settings.saves(), 1);
            let stored = settings.stored();
            assert_eq!(stored.values.len(), 2);
            assert_eq!(stored.values.get("core.agreement.version"), Some(&json!(fresher_core::AGREEMENT_VERSION)));
            assert_eq!(stored.values.get("core.agreement.accepted-at"), Some(&json!(START)));
        }

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
