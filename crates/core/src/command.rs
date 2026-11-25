//! What the commands take and return: the IPC surface between the UI and the engine.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::settings::schema::ENABLED;
use crate::{
    AccessStatus, Availability, ByteSize, CategoryDescriptor, CategoryId, CleanupId, CommandError, Coverage,
    DangerLevel, Finding, FindingId, ModuleDescriptor, ModuleGroup, ModuleId, ResolvedSettings, ScanId, ScanKind,
    ScanOutcome, SettingDescriptor, SettingGroup, SettingKey, SettingProblem, SettingValue, SettingsSchema, Sizes,
    Timestamp,
};

/// A long operation of the engine; only one runs at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Operation {
    Scan,
    Clean,
}

impl Operation {
    /// The operation's name as it is serialized.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Scan => "scan",
            Self::Clean => "clean",
        }
    }
}

/// What the engine is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum EngineStatus {
    Idle,
    Scanning { scan: ScanId },
    Cleaning { cleanup: CleanupId },
}

/// The answer of `get_overview`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub status: EngineStatus,
    pub last_scan: Option<LastScan>,
    pub totals: Vec<CategoryTotal>,
    /// Every finding of the current scan summed, a path counted once; the weakest completeness wins. Ignored findings
    /// are left out of it and of the totals.
    pub found: Sizes,
    /// How many entries the ignore lists hold.
    pub ignored: IgnoredCount,
    /// The registered modules the ignore lists hide, each with the reason, in registration order.
    pub hidden_modules: Vec<HiddenModule>,
}

/// A module whose findings the ignore lists hide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
pub struct HiddenModule {
    pub module: ModuleId,
    pub by: HiddenBy,
}

/// Why a module is hidden: its own entry, or its group's (which wins when both hold).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum HiddenBy {
    Module,
    Group,
}

/// The sizes of the ignore lists, for the overview's quiet note.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, TS)]
pub struct IgnoredCount {
    pub categories: u32,
    pub paths: u32,
    pub modules: u32,
    pub groups: u32,
}

/// The answer of `get_own_data`: what the app's own data folder takes on disk, against its budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
pub struct OwnData {
    pub total: ByteSize,
    pub budget: ByteSize,
}

/// What the user asks to ignore: a category, one finding by its path, a module, or a group of modules.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum IgnoreTarget {
    Category { category: String },
    Finding { finding: FindingId },
    Module { module: String },
    Group { group: ModuleGroup },
}

/// An entry of the ignore lists, as the settings hold it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum IgnoredEntry {
    Category { category: String },
    Path { path: PathBuf },
    Module { module: String },
    Group { group: ModuleGroup },
}

/// The scan whose findings the engine holds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct LastScan {
    pub scan: ScanId,
    pub finished: Timestamp,
    pub outcome: ScanOutcome,
    pub kind: ScanKind,
    pub coverage: Coverage,
    /// The cleanup that changed the disk since this scan, if one did: the scan's tree is out of date.
    #[serde(rename = "changedSince", skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub changed_since: Option<CleanupId>,
    /// A setting that applies at the next scan changed since this scan: "Rescan to apply".
    #[serde(rename = "settingsChanged")]
    pub settings_changed: bool,
}

/// The findings of one category in the last scan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CategoryTotal {
    pub category: CategoryId,
    pub count: u32,
    pub sizes: Sizes,
    pub highest_danger: DangerLevel,
}

/// A module as the UI sees it: what the generic module page renders.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct ModuleInfo {
    pub id: ModuleId,
    pub group: ModuleGroup,
    pub categories: Vec<CategoryInfo>,
    /// Its settings' keys, `<module>.enabled` first.
    pub settings: Vec<SettingKey>,
    /// Its choices of categories, in the order its page shows them; empty for a module without.
    pub choices: Vec<ChoiceInfo>,
}

/// A module's choice as the UI sees it: its name (the texts are `<module>.choice.<name>.label` and `.description`)
/// and the categories it selects.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct ChoiceInfo {
    pub name: String,
    pub categories: Vec<CategoryId>,
}

/// A category and what it can do with the current access.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct CategoryInfo {
    pub descriptor: CategoryDescriptor,
    pub availability: Availability,
}

impl ModuleInfo {
    /// The module of `descriptor`, each category's availability worked out from `access`.
    pub fn new(descriptor: &ModuleDescriptor, access: &AccessStatus) -> Self {
        let enabled = SettingKey::new(format!("{}.{ENABLED}", descriptor.id()));
        let own = descriptor.settings().iter().map(|setting| setting.key().clone());
        let category = |category: &CategoryDescriptor| CategoryInfo {
            descriptor: category.clone(),
            availability: Availability::of(category.needs(), access),
        };
        Self {
            id: descriptor.id(),
            group: descriptor.group(),
            categories: descriptor.categories().iter().map(category).collect(),
            settings: enabled.into_iter().chain(own).collect(),
            choices: descriptor
                .choices()
                .iter()
                .map(|choice| ChoiceInfo { name: choice.name().into(), categories: choice.categories().to_vec() })
                .collect(),
        }
    }
}

/// The order of a page of findings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum FindingOrder {
    /// The order the scan found them.
    Found,
    /// Allocated size, largest first; ties by id.
    Largest,
}

/// The input of `get_findings`: a page of the last scan's findings, optionally of one module or category.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, TS)]
pub struct FindingsQuery {
    module: Option<String>,
    category: Option<String>,
    order: FindingOrder,
    offset: u32,
    limit: u32,
}

impl FindingsQuery {
    /// The largest page the engine returns.
    pub const MAX_LIMIT: u32 = 500;

    pub const fn new(
        module: Option<String>,
        category: Option<String>,
        order: FindingOrder,
        offset: u32,
        limit: u32,
    ) -> Self {
        Self { module, category, order, offset, limit }
    }

    pub fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    pub fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }

    pub const fn order(&self) -> FindingOrder {
        self.order
    }

    pub const fn offset(&self) -> u32 {
        self.offset
    }

    /// The page size asked for, capped at `MAX_LIMIT`.
    pub const fn limit(&self) -> u32 {
        if self.limit > Self::MAX_LIMIT { Self::MAX_LIMIT } else { self.limit }
    }
}

/// The answer of `get_findings`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct FindingsPage {
    pub findings: Vec<Finding>,
    pub offset: u32,
    /// How many findings match the query in all.
    pub total: u32,
}

/// The answer of `get_settings`: every setting, its value, what was wrong with the stored ones, and what became of
/// the settings file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct SettingsView {
    pub settings: Vec<SettingDescriptor>,
    pub values: BTreeMap<SettingKey, SettingValue>,
    pub problems: Vec<SettingProblem>,
    /// Whether the settings can be changed, and why not.
    pub file: SettingsFile,
}

impl SettingsView {
    /// Every setting the screen shows: the settings of `SettingGroup::Agreement` are left out, descriptors, values and
    /// problems (the screen never shows them; the interface reads `get_agreement`).
    pub fn new(schema: &SettingsSchema, resolved: &ResolvedSettings, file: SettingsFile) -> Self {
        let (hidden, shown): (Vec<SettingDescriptor>, Vec<SettingDescriptor>) =
            schema.descriptors().iter().cloned().partition(|setting| setting.group() == SettingGroup::Agreement);
        let values = resolved
            .values
            .iter()
            .filter(|(key, _)| shown.iter().any(|setting| setting.key() == *key))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        let problems = resolved
            .problems
            .iter()
            .filter(|problem| hidden.iter().all(|setting| setting.key().as_str() != problem.key()))
            .cloned()
            .collect();
        Self { settings: shown, values, problems, file }
    }
}

/// What became of the settings file when it was loaded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum SettingsFile {
    /// Read (or absent): its values are in use.
    Loaded,
    /// Written by a newer version of the app: the defaults are in use and the file is never overwritten.
    NewerVersion,
    /// Not a settings file: the defaults are in use; the first save replaces it (the store moves it aside).
    Broken,
    /// Could not be read: the defaults are in use and the file is never overwritten, so a passing read error cannot
    /// replace the user's values with the defaults.
    Unreadable,
}

impl SettingsFile {
    /// Whether a change is refused with `SettingsReadOnly`.
    pub const fn is_read_only(self) -> bool {
        matches!(self, Self::NewerVersion | Self::Unreadable)
    }
}

/// The input of `update_setting`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, TS)]
pub struct SettingUpdate {
    pub key: String,
    pub value: SettingValue,
}

/// A language the app ships texts for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Locale {
    En,
}

impl Locale {
    /// The locale named `locale`; only `en` ships now.
    pub fn parse(locale: &str) -> Result<Self, CommandError> {
        match locale {
            "en" => Ok(Self::En),
            _ => Err(CommandError::UnknownLocale { locale: locale.into() }),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
        }
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{CommandError, FindingsQuery, Locale, SettingsFile};

        #[test]
        fn a_findings_query_over_the_limit_is_capped() {
            let query: FindingsQuery = serde_json::from_str(
                r#"{ "module": null, "category": null, "order": "found", "offset": 0, "limit": 100000 }"#,
            )
            .unwrap();

            assert_eq!(query.limit(), FindingsQuery::MAX_LIMIT);
        }

        #[test]
        fn a_file_that_is_newer_or_unreadable_is_read_only() {
            assert!(SettingsFile::NewerVersion.is_read_only());
            assert!(SettingsFile::Unreadable.is_read_only());
        }

        #[test]
        fn the_settings_view_leaves_the_agreement_group_out() {
            let schema = crate::SettingsSchema::new([]).unwrap();
            let resolved = schema.resolve(&crate::StoredSettings::default());

            let view = crate::SettingsView::new(&schema, &resolved, SettingsFile::Loaded);

            assert!(resolved.values.keys().any(|key| key.as_str() == "core.agreement.version"));
            assert!(view.settings.iter().all(|setting| setting.group() != crate::SettingGroup::Agreement));
            assert!(view.values.keys().all(|key| !key.as_str().starts_with("core.agreement.")));
            assert_eq!(view.settings.len(), view.values.len());
        }

        #[test]
        fn the_settings_view_leaves_the_problems_of_the_agreement_out() {
            let schema = crate::SettingsSchema::new([]).unwrap();
            let stored = [("core.agreement.version", serde_json::json!("one")), ("gone.key", serde_json::json!(1))];
            let stored = crate::StoredSettings {
                values: stored.into_iter().map(|(key, value)| (key.to_owned(), value)).collect(),
            };
            let resolved = schema.resolve(&stored);

            let view = crate::SettingsView::new(&schema, &resolved, SettingsFile::Loaded);

            assert!(resolved.problems.iter().any(|problem| problem.key() == "core.agreement.version"));
            assert_eq!(view.problems, [crate::SettingProblem::UnknownKey { key: "gone.key".into() }]);
        }

        #[test]
        fn an_unknown_locale_is_refused() {
            for locale in ["fr", "EN", ""] {
                assert_eq!(Locale::parse(locale), Err(CommandError::UnknownLocale { locale: locale.into() }));
            }
        }
    }

    mod positive {
        use serde_json::json;

        use crate::{
            Access, AccessStatus, Availability, ByteSize, Capability, CapabilityState, CategoryDescriptor, CategoryId,
            CategoryInfo, CategoryTotal, Coverage, DangerLevel, EngineStatus, FindingOrder, FindingsQuery, LastScan,
            Locale, Locales, ModuleDescriptor, ModuleGroup, ModuleId, ModuleInfo, Need, Operation, Overview, ScanId,
            ScanKind, ScanOutcome, SettingDescriptor, SettingKey, SettingKind, SettingUpdate, SettingValue,
            SettingsFile, SettingsSchema, SettingsView, Sizes, StoredSettings, Timestamp,
        };

        #[test]
        fn last_scan_serializes_its_kind() {
            let last = LastScan {
                scan: ScanId::new(3),
                finished: Timestamp::from_secs(200),
                outcome: ScanOutcome::Completed,
                kind: ScanKind::Rescan { changed_folders: 4 },
                coverage: Coverage::default(),
                changed_since: None,
                settings_changed: false,
            };
            let changed = LastScan { changed_since: Some(crate::CleanupId::new(5)), ..last.clone() };
            assert_eq!(serde_json::to_value(changed).unwrap()["changedSince"], 5);

            assert_eq!(
                serde_json::to_value(last).unwrap(),
                json!({
                    "scan": 3,
                    "finished": 200,
                    "outcome": { "kind": "completed" },
                    "kind": { "kind": "rescan", "changedFolders": 4 },
                    "coverage": { "notEntered": [], "unreadable": [] },
                    "settingsChanged": false,
                })
            );
        }

        #[test]
        fn a_findings_query_keeps_what_it_was_given() {
            let query =
                FindingsQuery::new(Some("demo".into()), Some("demo.caches".into()), FindingOrder::Found, 20, 50);

            assert_eq!(query.module(), Some("demo"));
            assert_eq!(query.category(), Some("demo.caches"));
            assert_eq!(query.order(), FindingOrder::Found);
            assert_eq!(query.offset(), 20);
            assert_eq!(query.limit(), 50);
        }

        #[test]
        fn a_loaded_or_broken_file_can_be_written() {
            assert!(!SettingsFile::Loaded.is_read_only());
            assert!(!SettingsFile::Broken.is_read_only());
            assert_eq!(serde_json::to_value(SettingsFile::NewerVersion).unwrap(), "newer-version");
        }

        #[test]
        fn english_is_the_shipped_locale() {
            assert_eq!(Locale::parse("en"), Ok(Locale::En));
            assert_eq!(Locale::En.as_str(), "en");
            assert_eq!(serde_json::to_value(Locale::En).unwrap(), "en");
        }

        #[test]
        fn multi_word_fields_serialize_in_camel_case() {
            let overview = Overview {
                status: EngineStatus::Idle,
                last_scan: None,
                totals: Vec::new(),
                found: Sizes::ZERO,
                ignored: crate::IgnoredCount::default(),
                hidden_modules: Vec::new(),
            };
            let demo = ModuleId::new("demo").unwrap();
            let total = CategoryTotal {
                category: CategoryId::new(demo, "demo.caches").unwrap(),
                count: 1,
                sizes: Sizes::complete(ByteSize::new(1), ByteSize::new(1)),
                highest_danger: DangerLevel::Low,
            };
            let access =
                AccessStatus { capabilities: vec![CapabilityState::new(Capability::AllFiles, Access::Unknown)] };

            assert_eq!(serde_json::to_value(overview).unwrap()["lastScan"], json!(null));
            assert_eq!(
                serde_json::from_value::<crate::IgnoreTarget>(json!({ "kind": "finding", "finding": 3 })).unwrap(),
                crate::IgnoreTarget::Finding { finding: crate::FindingId::new(3) }
            );
            assert_eq!(
                serde_json::to_value(crate::IgnoredEntry::Path { path: "/a".into() }).unwrap(),
                json!({ "kind": "path", "path": "/a" })
            );
            assert_eq!(serde_json::to_value(total).unwrap()["highestDanger"], "low");
            assert_eq!(
                serde_json::to_value(access).unwrap()["capabilities"][0]["capability"],
                json!({ "kind": "all-files" })
            );
        }

        #[test]
        fn the_ignore_kinds_of_a_module_and_a_group_round_trip() {
            let module = json!({ "kind": "module", "module": "demo" });
            let group = json!({ "kind": "group", "group": "debug" });

            assert_eq!(
                serde_json::from_value::<crate::IgnoreTarget>(module.clone()).unwrap(),
                crate::IgnoreTarget::Module { module: "demo".into() }
            );
            assert_eq!(
                serde_json::from_value::<crate::IgnoreTarget>(group.clone()).unwrap(),
                crate::IgnoreTarget::Group { group: ModuleGroup::Debug }
            );
            for entry in [module, group] {
                let read = serde_json::from_value::<crate::IgnoredEntry>(entry.clone()).unwrap();
                assert_eq!(serde_json::to_value(read).unwrap(), entry);
            }
            assert!(
                serde_json::from_value::<crate::IgnoreTarget>(json!({ "kind": "group", "group": "downloads" }))
                    .is_err()
            );
        }

        #[test]
        fn cleaning_status_carries_its_cleanup() {
            let cleaning = EngineStatus::Cleaning { cleanup: crate::CleanupId::new(3) };

            assert_eq!(serde_json::to_value(cleaning).unwrap(), json!({ "state": "cleaning", "cleanup": 3 }));
        }

        #[test]
        fn the_engine_status_serializes_with_a_state_tag() {
            let scanning = EngineStatus::Scanning { scan: ScanId::new(2) };

            assert_eq!(serde_json::to_value(scanning).unwrap(), json!({ "state": "scanning", "scan": 2 }));
            assert_eq!(serde_json::to_value(Operation::Clean).unwrap(), Operation::Clean.as_str());
            assert_eq!(Operation::Scan.as_str(), "scan");
        }

        #[test]
        fn module_info_lists_the_enabled_switch_first() {
            let demo = ModuleId::new("demo").unwrap();
            let caches = CategoryDescriptor::new(CategoryId::new(demo, "demo.caches").unwrap(), DangerLevel::Safe);
            let deep = SettingDescriptor::new(
                SettingKey::new("demo.deep").unwrap(),
                SettingKind::Switch,
                SettingValue::Switch(false),
            )
            .unwrap();
            let descriptor = ModuleDescriptor::new(
                demo,
                ModuleGroup::Development,
                vec![caches.clone()],
                vec![deep],
                Locales { en: "{}" },
            )
            .unwrap();

            let info = ModuleInfo::new(&descriptor, &AccessStatus::default());

            assert_eq!(info.id, demo);
            assert_eq!(info.group, ModuleGroup::Development);
            assert_eq!(serde_json::to_value(&info).unwrap()["group"], "development");
            assert_eq!(info.categories, [CategoryInfo { descriptor: caches, availability: Availability::Ready }]);
            assert_eq!(info.settings.iter().map(SettingKey::as_str).collect::<Vec<_>>(), ["demo.enabled", "demo.deep"]);

            let schema = SettingsSchema::new([&descriptor]).unwrap();
            let view = SettingsView::new(&schema, &schema.resolve(&StoredSettings::default()), SettingsFile::Loaded);
            // Motion, the scan roots, the smallest size on Largest, the four access opt-ins, the four ignore lists, and the module's two.
            assert_eq!(view.settings.len(), 13);
            assert_eq!(view.values.len(), 13);
            assert_eq!(view.problems, []);
        }

        #[test]
        fn module_info_carries_its_choices_in_order_and_none_without() {
            let demo = ModuleId::new("demo").unwrap();
            let id = |id| CategoryId::new(demo, id).unwrap();
            let categories =
                ["demo.caches", "demo.logs"].map(|name| CategoryDescriptor::new(id(name), DangerLevel::Low));
            let descriptor =
                ModuleDescriptor::new(demo, ModuleGroup::Debug, categories.to_vec(), vec![], Locales { en: "{}" })
                    .unwrap();
            let choices = vec![
                crate::ModuleChoice::new("everything", vec![id("demo.caches"), id("demo.logs")]),
                crate::ModuleChoice::new("keep", vec![id("demo.caches")]),
            ];

            let plain = ModuleInfo::new(&descriptor, &AccessStatus::default());
            let info = ModuleInfo::new(&descriptor.with_choices(choices).unwrap(), &AccessStatus::default());

            assert_eq!(plain.choices, []);
            let names: Vec<_> = info.choices.iter().map(|choice| choice.name.as_str()).collect();
            assert_eq!(names, ["everything", "keep"]);
            assert_eq!(info.choices[1].categories, [id("demo.caches")]);
            assert_eq!(
                serde_json::to_value(&info.choices[1]).unwrap(),
                json!({ "name": "keep", "categories": ["demo.caches"] })
            );
        }

        #[test]
        fn module_info_gives_each_category_its_availability() {
            let demo = ModuleId::new("demo").unwrap();
            let category = |id| CategoryDescriptor::new(CategoryId::new(demo, id).unwrap(), DangerLevel::Low);
            let caches = category("demo.caches").with_needs(vec![Need::sees_more(Capability::AllFiles)]);
            let logs = category("demo.logs");
            let descriptor =
                ModuleDescriptor::new(demo, ModuleGroup::Debug, vec![caches, logs], vec![], Locales { en: "{}" })
                    .unwrap();
            let denied =
                AccessStatus { capabilities: vec![CapabilityState::new(Capability::AllFiles, Access::Denied)] };

            let info = ModuleInfo::new(&descriptor, &denied);

            let availability: Vec<_> = info.categories.iter().map(|category| category.availability.clone()).collect();
            assert_eq!(
                availability,
                [Availability::Limited { missing: vec![Capability::AllFiles] }, Availability::Ready]
            );
            assert_eq!(
                serde_json::to_value(&info.categories[1]).unwrap(),
                json!({
                    "descriptor": { "id": "demo.logs", "danger": "low", "needs": [], "icon": "generic" },
                    "availability": { "kind": "ready" },
                })
            );
        }

        #[test]
        fn a_setting_update_reads_a_tagged_value() {
            let update: SettingUpdate =
                serde_json::from_value(json!({ "key": "demo.deep", "value": { "kind": "switch", "value": true } }))
                    .unwrap();

            assert_eq!(update.key, "demo.deep");
            assert_eq!(update.value, SettingValue::Switch(true));
        }
    }
}
