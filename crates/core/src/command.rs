//! What the commands take and return: the IPC surface between the UI and the engine.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::settings::schema::ENABLED;
use crate::{
    CategoryDescriptor, CategoryId, CommandError, DangerLevel, Finding, ModuleDescriptor, ModuleId, ResolvedSettings,
    ScanId, ScanKind, ScanOutcome, SettingDescriptor, SettingKey, SettingProblem, SettingValue, SettingsSchema, Sizes,
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
    Cleaning,
}

/// The answer of `get_overview`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub status: EngineStatus,
    pub last_scan: Option<LastScan>,
    pub totals: Vec<CategoryTotal>,
    /// Every finding of the current scan summed, a path counted once; the weakest completeness wins.
    pub found: Sizes,
}

/// The scan whose findings the engine holds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct LastScan {
    pub scan: ScanId,
    pub finished: Timestamp,
    pub outcome: ScanOutcome,
    pub kind: ScanKind,
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
    pub categories: Vec<CategoryDescriptor>,
    /// Its settings' keys, `<module>.enabled` first.
    pub settings: Vec<SettingKey>,
}

impl From<&ModuleDescriptor> for ModuleInfo {
    fn from(descriptor: &ModuleDescriptor) -> Self {
        let enabled = SettingKey::new(format!("{}.{ENABLED}", descriptor.id()));
        let own = descriptor.settings().iter().map(|setting| setting.key().clone());
        Self {
            id: descriptor.id(),
            categories: descriptor.categories().to_vec(),
            settings: enabled.into_iter().chain(own).collect(),
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
    pub fn new(schema: &SettingsSchema, resolved: &ResolvedSettings, file: SettingsFile) -> Self {
        Self {
            settings: schema.descriptors().to_vec(),
            values: resolved.values.clone(),
            problems: resolved.problems.clone(),
            file,
        }
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
        fn an_unknown_locale_is_refused() {
            for locale in ["fr", "EN", ""] {
                assert_eq!(Locale::parse(locale), Err(CommandError::UnknownLocale { locale: locale.into() }));
            }
        }
    }

    mod positive {
        use serde_json::json;

        use crate::{
            Access, AccessStatus, ByteSize, CategoryDescriptor, CategoryId, CategoryTotal, DangerLevel, EngineStatus,
            FindingOrder, FindingsQuery, LastScan, Locale, Locales, ModuleDescriptor, ModuleId, ModuleInfo, Operation,
            Overview, ScanId, ScanKind, ScanOutcome, SettingDescriptor, SettingKey, SettingKind, SettingUpdate,
            SettingValue, SettingsFile, SettingsSchema, SettingsView, Sizes, StoredSettings, Timestamp,
        };

        #[test]
        fn last_scan_serializes_its_kind() {
            let last = LastScan {
                scan: ScanId::new(3),
                finished: Timestamp::from_secs(200),
                outcome: ScanOutcome::Completed,
                kind: ScanKind::Restored { taken: Timestamp::from_secs(100) },
            };

            assert_eq!(
                serde_json::to_value(last).unwrap(),
                json!({
                    "scan": 3,
                    "finished": 200,
                    "outcome": { "kind": "completed" },
                    "kind": { "kind": "restored", "taken": 100 },
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
            let overview =
                Overview { status: EngineStatus::Idle, last_scan: None, totals: Vec::new(), found: Sizes::ZERO };
            let demo = ModuleId::new("demo").unwrap();
            let total = CategoryTotal {
                category: CategoryId::new(demo, "demo.caches").unwrap(),
                count: 1,
                sizes: Sizes::complete(ByteSize::new(1), ByteSize::new(1)),
                highest_danger: DangerLevel::Low,
            };
            let access = AccessStatus { full_disk_access: Access::Unknown };

            assert_eq!(serde_json::to_value(overview).unwrap()["lastScan"], json!(null));
            assert_eq!(serde_json::to_value(total).unwrap()["highestDanger"], "low");
            assert_eq!(serde_json::to_value(access).unwrap(), json!({ "fullDiskAccess": "unknown" }));
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
            let descriptor = ModuleDescriptor::new(demo, vec![caches], vec![deep], Locales { en: "{}" }).unwrap();

            let info = ModuleInfo::from(&descriptor);

            assert_eq!(info.id, demo);
            assert_eq!(info.categories, [caches]);
            assert_eq!(info.settings.iter().map(SettingKey::as_str).collect::<Vec<_>>(), ["demo.enabled", "demo.deep"]);

            let schema = SettingsSchema::new([&descriptor]).unwrap();
            let view = SettingsView::new(&schema, &schema.resolve(&StoredSettings::default()), SettingsFile::Loaded);
            assert_eq!(view.settings.len(), 3);
            assert_eq!(view.values.len(), 3);
            assert_eq!(view.problems, []);
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
