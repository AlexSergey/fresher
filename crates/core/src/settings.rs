//! Settings: their keys, kinds and values, and the check of a value against its descriptor. Pure: the file is the
//! `SettingsStore` port's.

mod core_schema;
mod folders;
pub(crate) mod schema;

use std::borrow::Borrow;
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use core_schema::{IGNORE_CATEGORIES, IGNORE_GROUPS, IGNORE_MODULES, IGNORE_PATHS, SCAN_ROOTS};
pub use schema::{ModuleSettings, ResolvedSettings, SchemaError, SettingsSchema};

use crate::{ModuleGroup, ModuleId, TextKey};

/// The key of a setting: `core.<name>` or `<module>.<name>`, dot-separated kebab-case segments (`core.scan.roots`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, TS)]
pub struct SettingKey(String);

impl SettingKey {
    pub fn new(key: impl Into<String>) -> Result<Self, SettingError> {
        let key = key.into();
        match TextKey::new(key.as_str()) {
            Ok(_) => Ok(Self(key)),
            Err(_) => Err(SettingError::InvalidKey { key }),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The first segment: `core` or a module id.
    pub fn namespace(&self) -> &str {
        self.0.split('.').next().unwrap_or_default()
    }

    /// The key of the setting's label: `<key>.label`.
    pub fn label_key(&self) -> TextKey {
        TextKey::from_valid(format!("{}.label", self.0))
    }

    /// The key of the setting's description: `<key>.description`.
    pub fn description_key(&self) -> TextKey {
        TextKey::from_valid(format!("{}.description", self.0))
    }
}

impl Borrow<str> for SettingKey {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SettingKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// What kind of value a setting holds, and its limits.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SettingKind {
    Switch,
    Number {
        #[ts(type = "number")]
        min: i64,
        #[ts(type = "number")]
        max: i64,
        unit: Option<Unit>,
    },
    Choice {
        options: Vec<&'static str>,
    },
    Folders,
    /// Ids of one kind, each checked as an id of that kind.
    Ids {
        of: IdOf,
    },
    /// Absolute paths of plain names, anywhere.
    Paths,
}

/// What the ids of an `Ids` setting name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum IdOf {
    /// Category ids: two or more dot-separated kebab-case segments each.
    Category,
    /// Module ids: one kebab-case segment each.
    Module,
    /// Group names, as a `ModuleGroup` is serialized.
    Group,
}

impl IdOf {
    /// Whether `id` is an id of this kind. Only its form is checked: whether a module registered it is the engine's.
    pub fn accepts(self, id: &str) -> bool {
        match self {
            Self::Category => TextKey::new(id).is_ok(),
            Self::Module => ModuleId::check(id).is_ok(),
            Self::Group => ModuleGroup::parse(id).is_some(),
        }
    }
}

/// The unit a number is shown in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Unit {
    Bytes,
    Days,
}

/// The value of a setting. Folders are kept as written: `~` stands for the home folder, which the app fills in.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum SettingValue {
    Switch(bool),
    Number(#[ts(type = "number")] i64),
    Choice(String),
    Folders(Vec<PathBuf>),
    Ids(Vec<String>),
    Paths(Vec<PathBuf>),
}

impl SettingValue {
    /// The value as it is stored in the settings file: a plain JSON value, without a tag.
    pub fn to_json(&self) -> serde_json::Value {
        let paths = |paths: &[PathBuf]| paths.iter().map(|path| path.to_string_lossy().into_owned()).collect();
        match self {
            Self::Switch(on) => (*on).into(),
            Self::Number(number) => (*number).into(),
            Self::Choice(choice) => choice.as_str().into(),
            Self::Folders(folders) => paths(folders),
            Self::Ids(ids) => ids.iter().map(String::as_str).collect(),
            Self::Paths(list) => paths(list),
        }
    }
}

/// Where the settings screen shows a setting. Only the core's settings are in the core's groups; a module's are in
/// `Module`, under the module's name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum SettingGroup {
    General,
    Scan,
    Ignored,
    Permissions,
    Module,
}

/// When a change of a setting shows in the results.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum SettingEffect {
    /// At once.
    Now,
    /// At the next scan: until then the last scan is marked as out of date.
    NextScan,
}

/// A setting a module (or the core) offers; its label and description are found by convention.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct SettingDescriptor {
    key: SettingKey,
    kind: SettingKind,
    default: SettingValue,
    group: SettingGroup,
    effect: SettingEffect,
}

/// Why a setting descriptor was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum SettingError {
    #[error("`{key}` is not a setting key: two or more dot-separated kebab-case segments")]
    InvalidKey { key: String },
    #[error("the default of `{key}` does not fit its kind")]
    BadDefault { key: SettingKey },
}

/// What is wrong with a value; the setting keeps its default. Reported to the user, never a crash.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SettingProblem {
    UnknownKey {
        key: String,
    },
    WrongType {
        key: SettingKey,
    },
    OutOfRange {
        key: SettingKey,
        #[ts(type = "number")]
        min: i64,
        #[ts(type = "number")]
        max: i64,
    },
    UnknownChoice {
        key: SettingKey,
    },
    /// A folder that is not `~`, `~/…` or a plain absolute path, or that is a protected system folder.
    RefusedFolder {
        key: SettingKey,
        folder: PathBuf,
    },
    /// An id that is not of the kind the list holds (`IdOf`).
    RefusedId {
        key: SettingKey,
        id: String,
    },
}

impl SettingProblem {
    /// The key of the setting the problem is about, as stored.
    pub fn key(&self) -> &str {
        match self {
            Self::UnknownKey { key } => key,
            Self::WrongType { key }
            | Self::OutOfRange { key, .. }
            | Self::UnknownChoice { key }
            | Self::RefusedFolder { key, .. }
            | Self::RefusedId { key, .. } => key.as_str(),
        }
    }
}

impl SettingDescriptor {
    /// A module's setting whose default fits its kind (a number in its range, a choice from its list). It is shown
    /// under its module and applies at the next scan.
    pub fn new(key: SettingKey, kind: SettingKind, default: SettingValue) -> Result<Self, SettingError> {
        let descriptor = Self::unchecked(key, kind, default, SettingGroup::Module, SettingEffect::NextScan);
        match descriptor.check(&descriptor.default) {
            Ok(()) => Ok(descriptor),
            Err(_) => Err(SettingError::BadDefault { key: descriptor.key }),
        }
    }

    /// A setting of the core's own making; its default is checked by the core's tests.
    pub(crate) const fn unchecked(
        key: SettingKey,
        kind: SettingKind,
        default: SettingValue,
        group: SettingGroup,
        effect: SettingEffect,
    ) -> Self {
        Self { key, kind, default, group, effect }
    }

    pub const fn key(&self) -> &SettingKey {
        &self.key
    }

    pub const fn kind(&self) -> &SettingKind {
        &self.kind
    }

    pub const fn default(&self) -> &SettingValue {
        &self.default
    }

    pub const fn group(&self) -> SettingGroup {
        self.group
    }

    pub const fn effect(&self) -> SettingEffect {
        self.effect
    }

    /// Checks `value` against the setting's kind and limits.
    pub fn check(&self, value: &SettingValue) -> Result<(), SettingProblem> {
        let refused_path =
            |allowed: fn(&std::path::Path) -> bool, paths: &[PathBuf]| match paths.iter().find(|path| !allowed(path)) {
                None => Ok(()),
                Some(folder) => Err(SettingProblem::RefusedFolder { key: self.key.clone(), folder: folder.clone() }),
            };
        match (&self.kind, value) {
            (SettingKind::Switch, SettingValue::Switch(_)) => Ok(()),
            (SettingKind::Folders, SettingValue::Folders(folders)) => refused_path(folders::is_allowed, folders),
            (SettingKind::Paths, SettingValue::Paths(paths)) => refused_path(folders::is_plain_absolute, paths),
            (&SettingKind::Ids { of }, SettingValue::Ids(ids)) => match ids.iter().find(|id| !of.accepts(id)) {
                None => Ok(()),
                Some(id) => Err(SettingProblem::RefusedId { key: self.key.clone(), id: id.clone() }),
            },
            (&SettingKind::Number { min, max, .. }, &SettingValue::Number(number)) => {
                if (min..=max).contains(&number) {
                    Ok(())
                } else {
                    Err(SettingProblem::OutOfRange { key: self.key.clone(), min, max })
                }
            }
            (SettingKind::Choice { options }, SettingValue::Choice(choice)) => {
                if options.contains(&choice.as_str()) {
                    Ok(())
                } else {
                    Err(SettingProblem::UnknownChoice { key: self.key.clone() })
                }
            }
            _ => Err(SettingProblem::WrongType { key: self.key.clone() }),
        }
    }

    /// Reads a stored JSON value as this setting's value, and checks it.
    pub fn read(&self, stored: &serde_json::Value) -> Result<SettingValue, SettingProblem> {
        let value = match (&self.kind, stored) {
            (SettingKind::Switch, serde_json::Value::Bool(on)) => Some(SettingValue::Switch(*on)),
            (SettingKind::Number { .. }, serde_json::Value::Number(number)) => {
                number.as_i64().map(SettingValue::Number)
            }
            (SettingKind::Choice { .. }, serde_json::Value::String(choice)) => {
                Some(SettingValue::Choice(choice.clone()))
            }
            (SettingKind::Folders, serde_json::Value::Array(items)) => strings(items).map(SettingValue::Folders),
            (SettingKind::Paths, serde_json::Value::Array(items)) => strings(items).map(SettingValue::Paths),
            (SettingKind::Ids { .. }, serde_json::Value::Array(items)) => strings(items).map(SettingValue::Ids),
            _ => None,
        };
        let value = value.ok_or_else(|| SettingProblem::WrongType { key: self.key.clone() })?;
        self.check(&value)?;
        Ok(value)
    }
}

/// Every item of a stored list as a string, then as a `T`; `None` when one is not a string.
fn strings<T: From<String>>(items: &[serde_json::Value]) -> Option<Vec<T>> {
    items.iter().map(|item| item.as_str().map(|text| T::from(text.to_owned()))).collect()
}

#[cfg(test)]
mod tests {
    mod negative {
        use serde_json::json;

        use crate::{SettingDescriptor, SettingError, SettingKey, SettingKind, SettingProblem, SettingValue};

        fn days() -> SettingDescriptor {
            let kind = SettingKind::Number { min: 1, max: 365, unit: Some(crate::Unit::Days) };
            SettingDescriptor::new(SettingKey::new("demo.min-age").unwrap(), kind, SettingValue::Number(30)).unwrap()
        }

        #[test]
        fn rejects_an_invalid_setting_key() {
            for key in ["", "demo", "demo.", "Demo.age", "demo..age"] {
                assert_eq!(SettingKey::new(key), Err(SettingError::InvalidKey { key: key.into() }), "{key:?}");
            }
        }

        #[test]
        fn rejects_a_default_that_does_not_fit() {
            let key = SettingKey::new("demo.mode").unwrap();
            let choice = SettingKind::Choice { options: vec!["fast", "full"] };

            let error = SettingDescriptor::new(key.clone(), choice, SettingValue::Choice("slow".into()));

            assert_eq!(error, Err(SettingError::BadDefault { key }));
        }

        #[test]
        fn rejects_a_number_outside_its_range() {
            let key = SettingKey::new("demo.min-age").unwrap();
            let problem = SettingProblem::OutOfRange { key, min: 1, max: 365 };

            assert_eq!(days().check(&SettingValue::Number(0)), Err(problem.clone()));
            assert_eq!(days().read(&json!(366)), Err(problem));
        }

        #[test]
        fn rejects_a_choice_not_in_the_list() {
            let key = SettingKey::new("demo.mode").unwrap();
            let kind = SettingKind::Choice { options: vec!["fast", "full"] };
            let mode = SettingDescriptor::new(key.clone(), kind, SettingValue::Choice("fast".into())).unwrap();

            assert_eq!(mode.read(&json!("slow")), Err(SettingProblem::UnknownChoice { key }));
        }

        #[test]
        fn rejects_a_stored_value_of_the_wrong_type() {
            let key = SettingKey::new("demo.min-age").unwrap();

            for stored in [json!("30"), json!(1.5), json!(true), json!(null), json!([30])] {
                assert_eq!(days().read(&stored), Err(SettingProblem::WrongType { key: key.clone() }), "{stored}");
            }
            assert_eq!(days().check(&SettingValue::Switch(true)), Err(SettingProblem::WrongType { key }));
        }

        #[test]
        fn rejects_a_relative_or_protected_folder() {
            let key = SettingKey::new("demo.roots").unwrap();
            let roots =
                SettingDescriptor::new(key.clone(), SettingKind::Folders, SettingValue::Folders(vec![])).unwrap();
            let refused = |folder: &str| SettingProblem::RefusedFolder { key: key.clone(), folder: folder.into() };

            assert_eq!(roots.read(&json!(["~", "/System"])), Err(refused("/System")));
            assert_eq!(roots.read(&json!(["~/../.."])), Err(refused("~/../..")));
            assert_eq!(roots.check(&SettingValue::Folders(vec!["Projects".into()])), Err(refused("Projects")));
            assert_eq!(roots.check(&SettingValue::Folders(vec!["/".into()])), Err(refused("/")));
        }

        fn list(key: &str, kind: SettingKind, default: SettingValue) -> SettingDescriptor {
            SettingDescriptor::new(SettingKey::new(key).unwrap(), kind, default).unwrap()
        }

        #[test]
        fn rejects_an_id_that_is_not_a_category_id() {
            let key = SettingKey::new("demo.ids").unwrap();
            let ids = list("demo.ids", SettingKind::Ids { of: crate::IdOf::Category }, SettingValue::Ids(vec![]));

            for id in ["", "demo", "Demo.caches", "demo..caches", "demo.caches."] {
                let problem = SettingProblem::RefusedId { key: key.clone(), id: id.into() };
                assert_eq!(ids.read(&json!(["demo.caches", id])), Err(problem.clone()), "{id:?}");
                assert_eq!(problem.key(), "demo.ids");
            }
        }

        #[test]
        fn rejects_an_id_that_is_not_a_module_id() {
            let key = SettingKey::new("demo.modules").unwrap();
            let modules = list("demo.modules", SettingKind::Ids { of: crate::IdOf::Module }, SettingValue::Ids(vec![]));

            for id in ["", "Demo", "demo.caches", "demo_x", "core"] {
                let problem = SettingProblem::RefusedId { key: key.clone(), id: id.into() };
                assert_eq!(modules.read(&json!(["demo", id])), Err(problem), "{id:?}");
            }
        }

        #[test]
        fn rejects_a_name_that_is_not_a_group() {
            let key = SettingKey::new("demo.groups").unwrap();
            let groups = list("demo.groups", SettingKind::Ids { of: crate::IdOf::Group }, SettingValue::Ids(vec![]));

            for id in ["", "apps", "Debug", "development.tools"] {
                let problem = SettingProblem::RefusedId { key: key.clone(), id: id.into() };
                assert_eq!(groups.read(&json!(["debug", id])), Err(problem), "{id:?}");
            }
        }

        #[test]
        fn rejects_a_relative_or_dotted_ignored_path() {
            let key = SettingKey::new("demo.paths").unwrap();
            let paths = list("demo.paths", SettingKind::Paths, SettingValue::Paths(vec![]));

            for path in ["~/keep", "keep", "/a/../b", "/", ""] {
                let problem = SettingProblem::RefusedFolder { key: key.clone(), folder: path.into() };
                assert_eq!(paths.read(&json!([path])), Err(problem), "{path:?}");
            }
        }

        #[test]
        fn rejects_a_stored_list_that_is_not_strings() {
            let ids = list("demo.ids", SettingKind::Ids { of: crate::IdOf::Category }, SettingValue::Ids(vec![]));
            let paths = list("demo.paths", SettingKind::Paths, SettingValue::Paths(vec![]));

            assert_eq!(
                ids.read(&json!([1])),
                Err(SettingProblem::WrongType { key: SettingKey::new("demo.ids").unwrap() })
            );
            assert_eq!(
                paths.read(&json!("/a")),
                Err(SettingProblem::WrongType { key: SettingKey::new("demo.paths").unwrap() })
            );
            assert_eq!(
                ids.check(&SettingValue::Paths(vec![])),
                Err(SettingProblem::WrongType { key: SettingKey::new("demo.ids").unwrap() })
            );
        }

        #[test]
        fn rejects_folders_that_are_not_strings() {
            let key = SettingKey::new("demo.roots").unwrap();
            let roots =
                SettingDescriptor::new(key.clone(), SettingKind::Folders, SettingValue::Folders(vec![])).unwrap();

            assert_eq!(roots.read(&json!(["~", 1])), Err(SettingProblem::WrongType { key }));
        }
    }

    mod positive {
        use serde_json::json;

        use crate::{SettingDescriptor, SettingKey, SettingKind, SettingValue};

        #[test]
        fn derives_the_label_and_description_keys() {
            let key = SettingKey::new("demo.min-age").unwrap();

            assert_eq!(key.label_key().as_str(), "demo.min-age.label");
            assert_eq!(key.description_key().as_str(), "demo.min-age.description");
            assert_eq!(key.namespace(), "demo");
            assert_eq!(key.to_string(), "demo.min-age");
        }

        #[test]
        fn reads_every_kind_from_plain_json() {
            let read = |kind: SettingKind, default: SettingValue, stored: serde_json::Value| {
                SettingDescriptor::new(SettingKey::new("demo.value").unwrap(), kind, default).unwrap().read(&stored)
            };

            assert_eq!(
                read(SettingKind::Switch, SettingValue::Switch(true), json!(false)),
                Ok(SettingValue::Switch(false))
            );
            assert_eq!(
                read(SettingKind::Number { min: 0, max: 9, unit: None }, SettingValue::Number(1), json!(9)),
                Ok(SettingValue::Number(9))
            );
            assert_eq!(
                read(SettingKind::Choice { options: vec!["a", "b"] }, SettingValue::Choice("a".into()), json!("b")),
                Ok(SettingValue::Choice("b".into()))
            );
            assert_eq!(
                read(SettingKind::Folders, SettingValue::Folders(vec![]), json!(["~", "/Volumes/Data"])),
                Ok(SettingValue::Folders(vec!["~".into(), "/Volumes/Data".into()]))
            );
        }

        #[test]
        fn reads_ids_and_paths_from_plain_json_and_stores_them_back() {
            let ids = SettingDescriptor::new(
                SettingKey::new("demo.ids").unwrap(),
                SettingKind::Ids { of: crate::IdOf::Category },
                SettingValue::Ids(vec![]),
            )
            .unwrap();
            let paths = SettingDescriptor::new(
                SettingKey::new("demo.paths").unwrap(),
                SettingKind::Paths,
                SettingValue::Paths(vec![]),
            )
            .unwrap();

            let read_ids = ids.read(&json!(["alpha.caches", "beta.deps"])).unwrap();
            let read_paths = paths.read(&json!(["/System/Library", "/Users/ann/keep"])).unwrap();

            assert_eq!(read_ids, SettingValue::Ids(vec!["alpha.caches".into(), "beta.deps".into()]));
            assert_eq!(read_paths, SettingValue::Paths(vec!["/System/Library".into(), "/Users/ann/keep".into()]));
            assert_eq!(read_ids.to_json(), json!(["alpha.caches", "beta.deps"]));
            assert_eq!(read_paths.to_json(), json!(["/System/Library", "/Users/ann/keep"]));
        }

        #[test]
        fn reads_module_ids_and_group_names_and_serializes_what_the_list_holds() {
            let list = |key: &str, of| {
                SettingDescriptor::new(
                    SettingKey::new(key).unwrap(),
                    SettingKind::Ids { of },
                    SettingValue::Ids(vec![]),
                )
                .unwrap()
            };
            let modules = list("demo.modules", crate::IdOf::Module);
            let groups = list("demo.groups", crate::IdOf::Group);

            assert_eq!(
                modules.read(&json!(["demo", "jet-brains"])),
                Ok(SettingValue::Ids(vec!["demo".into(), "jet-brains".into()]))
            );
            assert_eq!(
                groups.read(&json!(["development", "debug"])),
                Ok(SettingValue::Ids(vec!["development".into(), "debug".into()]))
            );
            assert_eq!(serde_json::to_value(groups.kind()).unwrap(), json!({ "kind": "ids", "of": "group" }));
        }

        #[test]
        fn stores_values_as_plain_json() {
            assert_eq!(SettingValue::Switch(true).to_json(), json!(true));
            assert_eq!(SettingValue::Number(-3).to_json(), json!(-3));
            assert_eq!(SettingValue::Choice("fast".into()).to_json(), json!("fast"));
            assert_eq!(SettingValue::Folders(vec!["~".into()]).to_json(), json!(["~"]));
        }

        #[test]
        fn serializes_a_value_with_a_kind_tag() {
            let value = SettingValue::Number(5);

            assert_eq!(serde_json::to_value(&value).unwrap(), json!({ "kind": "number", "value": 5 }));
            assert_eq!(serde_json::from_value::<SettingValue>(json!({ "kind": "number", "value": 5 })).unwrap(), value);
        }
    }
}
