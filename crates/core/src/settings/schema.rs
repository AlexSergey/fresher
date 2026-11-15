//! Every setting of the app, and the stored values resolved against them.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use super::core_schema::core_settings;
use super::{SettingDescriptor, SettingKey, SettingKind, SettingProblem, SettingValue};
use crate::{ModuleDescriptor, ModuleId, StoredSettings};

/// The last segment of the switch the schema adds for every module: `<module>.enabled`.
pub(crate) const ENABLED: &str = "enabled";

/// The settings of the core and of every module, in that order; built by the engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsSchema {
    settings: Vec<SettingDescriptor>,
}

/// Why a schema could not be built.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum SchemaError {
    #[error("setting `{key}` is declared twice (two modules with one id?)")]
    DuplicateKey { key: SettingKey },
}

/// The values of every setting, and what was wrong with the stored ones.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResolvedSettings {
    /// One value per setting of the schema: the stored one, or the default.
    pub values: BTreeMap<SettingKey, SettingValue>,
    pub problems: Vec<SettingProblem>,
}

impl SettingsSchema {
    /// The core's settings, then each module's: first its `<module>.enabled` switch (on by default), then its own.
    pub fn new<'a>(modules: impl IntoIterator<Item = &'a ModuleDescriptor>) -> Result<Self, SchemaError> {
        let mut settings = core_settings();
        for module in modules {
            settings.push(SettingDescriptor {
                key: SettingKey(format!("{}.{ENABLED}", module.id())),
                kind: SettingKind::Switch,
                default: SettingValue::Switch(true),
            });
            settings.extend(module.settings().iter().cloned());
        }
        let mut seen = BTreeSet::new();
        for setting in &settings {
            if !seen.insert(setting.key()) {
                return Err(SchemaError::DuplicateKey { key: setting.key().clone() });
            }
        }
        Ok(Self { settings })
    }

    pub fn descriptors(&self) -> &[SettingDescriptor] {
        &self.settings
    }

    pub fn get(&self, key: &str) -> Option<&SettingDescriptor> {
        self.settings.iter().find(|setting| setting.key().as_str() == key)
    }

    /// Checks a new value for `key`.
    pub fn check(&self, key: &str, value: &SettingValue) -> Result<(), SettingProblem> {
        self.get(key).ok_or_else(|| SettingProblem::UnknownKey { key: key.into() })?.check(value)
    }

    /// Every setting's value: the stored one when it is valid, otherwise the default and a problem. An unknown stored
    /// key is a problem and stays out of the values. Never fails.
    pub fn resolve(&self, stored: &StoredSettings) -> ResolvedSettings {
        let mut resolved = ResolvedSettings::default();
        for key in stored.values.keys().filter(|key| self.get(key).is_none()) {
            resolved.problems.push(SettingProblem::UnknownKey { key: key.clone() });
        }
        for setting in &self.settings {
            let value = match stored.values.get(setting.key().as_str()).map(|json| setting.read(json)) {
                Some(Ok(value)) => value,
                Some(Err(problem)) => {
                    resolved.problems.push(problem);
                    setting.default().clone()
                }
                None => setting.default().clone(),
            };
            resolved.values.insert(setting.key().clone(), value);
        }
        resolved
    }
}

impl ResolvedSettings {
    pub fn get(&self, key: &str) -> Option<&SettingValue> {
        self.values.get(key)
    }

    /// The values in the namespace of `module`, for its `ModuleContext`.
    pub fn module(&self, module: ModuleId) -> ModuleSettings {
        let values = self.values.iter().filter(|(key, _)| key.namespace() == module.as_str());
        ModuleSettings { module, values: values.map(|(key, value)| (key.clone(), value.clone())).collect() }
    }
}

/// A module's view of its own settings, by the name after `<module>.`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleSettings {
    module: ModuleId,
    values: BTreeMap<SettingKey, SettingValue>,
}

impl ModuleSettings {
    /// No values: what a module sees when its settings have not been resolved (tests of a module without settings).
    pub const fn empty(module: ModuleId) -> Self {
        Self { module, values: BTreeMap::new() }
    }

    pub const fn module(&self) -> ModuleId {
        self.module
    }

    /// The value of `<module>.<name>`.
    pub fn get(&self, name: &str) -> Option<&SettingValue> {
        self.values.get(format!("{}.{name}", self.module).as_str())
    }

    pub fn switch(&self, name: &str) -> Option<bool> {
        match self.get(name)? {
            SettingValue::Switch(on) => Some(*on),
            _ => None,
        }
    }

    pub fn number(&self, name: &str) -> Option<i64> {
        match self.get(name)? {
            SettingValue::Number(number) => Some(*number),
            _ => None,
        }
    }

    pub fn choice(&self, name: &str) -> Option<&str> {
        match self.get(name)? {
            SettingValue::Choice(choice) => Some(choice),
            _ => None,
        }
    }

    pub fn folders(&self, name: &str) -> Option<&[PathBuf]> {
        match self.get(name)? {
            SettingValue::Folders(folders) => Some(folders),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Locales, ModuleDescriptor, ModuleId, SettingDescriptor, SettingKey, SettingKind, SettingValue, SettingsSchema,
        StoredSettings, Unit,
    };

    fn demo() -> ModuleId {
        ModuleId::new("demo").unwrap()
    }

    fn setting(key: &str, kind: SettingKind, default: SettingValue) -> SettingDescriptor {
        SettingDescriptor::new(SettingKey::new(key).unwrap(), kind, default).unwrap()
    }

    fn module() -> ModuleDescriptor {
        let settings = vec![
            setting(
                "demo.min-age",
                SettingKind::Number { min: 1, max: 365, unit: Some(Unit::Days) },
                SettingValue::Number(30),
            ),
            setting(
                "demo.mode",
                SettingKind::Choice { options: vec!["fast", "full"] },
                SettingValue::Choice("fast".into()),
            ),
        ];
        ModuleDescriptor::new(demo(), vec![], settings, Locales { en: "{}" }).unwrap()
    }

    fn schema() -> SettingsSchema {
        SettingsSchema::new([&module()]).unwrap()
    }

    fn stored(values: serde_json::Value) -> StoredSettings {
        serde_json::from_value(values).unwrap()
    }

    mod negative {
        use serde_json::json;

        use super::{module, schema, stored};
        use crate::{SchemaError, SettingKey, SettingProblem, SettingValue, SettingsSchema};

        #[test]
        fn falls_back_to_the_default_for_a_value_of_the_wrong_type() {
            let resolved = schema().resolve(&stored(json!({ "demo.min-age": "old", "demo.enabled": 1 })));

            assert_eq!(resolved.get("demo.min-age"), Some(&SettingValue::Number(30)));
            assert_eq!(resolved.get("demo.enabled"), Some(&SettingValue::Switch(true)));
            assert_eq!(
                resolved.problems,
                [
                    SettingProblem::WrongType { key: SettingKey::new("demo.enabled").unwrap() },
                    SettingProblem::WrongType { key: SettingKey::new("demo.min-age").unwrap() },
                ]
            );
        }

        #[test]
        fn reports_an_unknown_key_and_keeps_it_out_of_the_values() {
            let resolved = schema().resolve(&stored(json!({ "demo.gone": true })));

            assert_eq!(resolved.problems, [SettingProblem::UnknownKey { key: "demo.gone".into() }]);
            assert_eq!(resolved.get("demo.gone"), None);
        }

        #[test]
        fn refuses_a_new_value_for_an_unknown_key() {
            let problem = schema().check("demo.gone", &SettingValue::Switch(true));

            assert_eq!(problem, Err(SettingProblem::UnknownKey { key: "demo.gone".into() }));
        }

        #[test]
        fn rejects_two_modules_with_one_id() {
            let error = SettingsSchema::new([&module(), &module()]);

            assert_eq!(error, Err(SchemaError::DuplicateKey { key: SettingKey::new("demo.enabled").unwrap() }));
        }

        #[test]
        fn a_module_reads_nothing_of_another_kind_or_name() {
            let settings = schema().resolve(&stored(json!({}))).module(super::demo());

            assert_eq!(settings.switch("min-age"), None);
            assert_eq!(settings.number("mode"), None);
            assert_eq!(settings.choice("enabled"), None);
            assert_eq!(settings.folders("enabled"), None);
            assert_eq!(settings.get("missing"), None);
            assert_eq!(crate::ModuleSettings::empty(super::demo()).switch("enabled"), None);
        }
    }

    mod positive {
        use std::path::PathBuf;

        use serde_json::json;

        use super::{schema, stored};
        use crate::{SettingDescriptor, SettingValue, SettingsSchema};

        #[test]
        fn resolves_a_valid_value() {
            let resolved = schema().resolve(&stored(json!({ "demo.min-age": 90, "demo.mode": "full" })));

            assert_eq!(resolved.problems, []);
            assert_eq!(resolved.get("demo.min-age"), Some(&SettingValue::Number(90)));
            assert_eq!(resolved.get("demo.mode"), Some(&SettingValue::Choice("full".into())));
            assert_eq!(schema().check("demo.min-age", &SettingValue::Number(7)), Ok(()));
        }

        #[test]
        fn adds_an_enabled_switch_for_every_module() {
            let schema = schema();
            let keys: Vec<_> = schema.descriptors().iter().map(|setting| setting.key().as_str()).collect();

            assert_eq!(keys, ["core.scan.roots", "demo.enabled", "demo.min-age", "demo.mode"]);
            assert_eq!(schema.get("demo.enabled").map(SettingDescriptor::default), Some(&SettingValue::Switch(true)));
        }

        #[test]
        fn defaults_the_scan_roots_to_home() {
            let resolved = SettingsSchema::new([]).unwrap().resolve(&stored(json!({})));

            assert_eq!(resolved.get("core.scan.roots"), Some(&SettingValue::Folders(vec![PathBuf::from("~")])));
            assert_eq!(resolved.problems, []);
        }

        #[test]
        fn a_module_reads_its_own_values_by_name() {
            let resolved = schema().resolve(&stored(json!({ "demo.enabled": false, "core.scan.roots": ["/data"] })));
            let settings = resolved.module(super::demo());

            assert_eq!(settings.module(), super::demo());
            assert_eq!(settings.switch("enabled"), Some(false));
            assert_eq!(settings.number("min-age"), Some(30));
            assert_eq!(settings.choice("mode"), Some("fast"));
            assert_eq!(settings.get("scan.roots"), None);
            let core = resolved.module(crate::ModuleId::new("other").unwrap());
            assert_eq!(core.get("enabled"), None);
            assert_eq!(resolved.get("core.scan.roots"), Some(&SettingValue::Folders(vec![PathBuf::from("/data")])));
        }

        #[test]
        fn a_module_reads_its_folders() {
            let folders = SettingDescriptor::new(
                crate::SettingKey::new("demo.extra").unwrap(),
                crate::SettingKind::Folders,
                SettingValue::Folders(vec![PathBuf::from("/x")]),
            )
            .unwrap();
            let module =
                crate::ModuleDescriptor::new(super::demo(), vec![], vec![folders], crate::Locales { en: "{}" })
                    .unwrap();
            let settings = SettingsSchema::new([&module]).unwrap().resolve(&stored(json!({}))).module(super::demo());

            assert_eq!(settings.folders("extra"), Some([PathBuf::from("/x")].as_slice()));
        }
    }

    mod property {
        use proptest::prelude::*;
        use serde_json::Value;

        use super::schema;
        use crate::StoredSettings;

        fn json() -> impl Strategy<Value = Value> {
            let leaf = prop_oneof![
                Just(Value::Null),
                any::<bool>().prop_map(Value::from),
                any::<i64>().prop_map(Value::from),
                any::<f64>().prop_map(Value::from),
                ".*".prop_map(Value::from),
                prop_oneof![Just("fast"), Just("full"), Just("~")].prop_map(Value::from),
            ];
            leaf.prop_recursive(3, 16, 4, |inner| {
                prop_oneof![
                    prop::collection::vec(inner.clone(), 0..4).prop_map(Value::from),
                    prop::collection::btree_map(".*", inner, 0..4)
                        .prop_map(|map| Value::Object(map.into_iter().collect())),
                ]
            })
        }

        fn key() -> impl Strategy<Value = String> {
            prop_oneof![
                Just("core.scan.roots".to_owned()),
                Just("demo.enabled".to_owned()),
                Just("demo.min-age".to_owned()),
                Just("demo.mode".to_owned()),
                ".*",
            ]
        }

        proptest! {
            #[test]
            fn resolve_never_fails_whatever_the_stored_json(values in prop::collection::btree_map(key(), json(), 0..6)) {
                let schema = schema();

                let resolved = schema.resolve(&StoredSettings { values });

                prop_assert_eq!(resolved.values.len(), schema.descriptors().len());
                for (key, value) in &resolved.values {
                    prop_assert_eq!(schema.check(key.as_str(), value), Ok(()));
                }
            }
        }
    }
}
