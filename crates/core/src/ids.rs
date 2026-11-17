//! Identities: of a module, a category, a finding, a scan, an external tool, and a point in time.

use std::fmt;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The namespace of the core's own text and setting keys; no module may take it.
pub(crate) const CORE_NAMESPACE: &str = "core";

/// Why an identity was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum IdError {
    #[error("`{id}` is not a kebab-case name")]
    NotKebabCase { id: String },
    #[error("`{id}` is reserved for the core")]
    Reserved { id: String },
    #[error("category `{id}` is not inside the namespace of module `{module}`")]
    OutsideModule { id: String, module: String },
}

/// `true` for a lowercase kebab-case name: `demo`, `node-modules`, `jetbrains-2024`.
pub(crate) fn is_kebab_case(name: &str) -> bool {
    name.starts_with(|first: char| first.is_ascii_lowercase())
        && !name.ends_with('-')
        && !name.contains("--")
        && name.chars().all(|char| char.is_ascii_lowercase() || char.is_ascii_digit() || char == '-')
}

/// The id of a module: a kebab-case name, unique in the app (`demo`, `node-modules`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, TS)]
pub struct ModuleId(&'static str);

impl ModuleId {
    pub fn new(id: &'static str) -> Result<Self, IdError> {
        Self::check(id)?;
        Ok(Self(id))
    }

    /// Checks `id` as [`Self::new`] does, without making an id: for an id read at run time, before it is kept.
    pub fn check(id: &str) -> Result<(), IdError> {
        if !is_kebab_case(id) {
            return Err(IdError::NotKebabCase { id: id.into() });
        }
        if id == CORE_NAMESPACE {
            return Err(IdError::Reserved { id: id.into() });
        }
        Ok(())
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }

    /// The owner of the interests the engine adds to the walk for the system's facts (the projects): the core's own
    /// namespace, which no module can take.
    pub const fn system() -> Self {
        Self(CORE_NAMESPACE)
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

/// The id of a category: `<module>.<name>`, both kebab-case (`demo.caches`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, TS)]
pub struct CategoryId(&'static str);

impl CategoryId {
    /// A category of `module`; `id` must be `<module>.<name>`.
    pub fn new(module: ModuleId, id: &'static str) -> Result<Self, IdError> {
        Self::check(module.as_str(), id)?;
        Ok(Self(id))
    }

    /// Checks `id` as a category of the module named `module`, as [`Self::new`] does, without making an id: for an
    /// id read at run time, before it is kept. The module's own name is checked by [`ModuleId::check`].
    pub fn check(module: &str, id: &str) -> Result<(), IdError> {
        let outside = || IdError::OutsideModule { id: id.into(), module: module.into() };
        let (namespace, name) = id.split_once('.').ok_or_else(outside)?;
        if namespace != module {
            return Err(outside());
        }
        if !is_kebab_case(name) {
            return Err(IdError::NotKebabCase { id: id.into() });
        }
        Ok(())
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }

    /// The module whose namespace the category is in.
    pub fn module(self) -> ModuleId {
        ModuleId(self.0.split_once('.').map_or(self.0, |(module, _)| module))
    }
}

impl fmt::Display for CategoryId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

/// The id of a finding in one session, assigned by the engine: the UI's only handle on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
pub struct FindingId(u32);

impl FindingId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for FindingId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// What a finding is across scans (the ignore list, the history): its module and an identity the module chose.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, TS)]
pub struct FindingKey {
    module: ModuleId,
    identity: String,
}

impl FindingKey {
    pub fn new(module: ModuleId, identity: impl Into<String>) -> Self {
        Self { module, identity: identity.into() }
    }

    pub const fn module(&self) -> ModuleId {
        self.module
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }
}

/// The id of one scan in a session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
pub struct ScanId(u32);

impl ScanId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

/// An external tool a module talks to (`docker`): a kebab-case name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, TS)]
pub struct ToolId(&'static str);

impl ToolId {
    pub fn new(id: &'static str) -> Result<Self, IdError> {
        if is_kebab_case(id) { Ok(Self(id)) } else { Err(IdError::NotKebabCase { id: id.into() }) }
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// Seconds since the Unix epoch, read from the `Clock` port.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
pub struct Timestamp(#[ts(type = "number")] u64);

impl Timestamp {
    pub const fn from_secs(secs: u64) -> Self {
        Self(secs)
    }

    /// The whole seconds of `time`; a time before the epoch is the epoch.
    pub fn from_system_time(time: SystemTime) -> Self {
        Self(time.duration_since(SystemTime::UNIX_EPOCH).map_or(0, |since| since.as_secs()))
    }

    pub const fn secs(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{CategoryId, IdError, ModuleId, ToolId};

        #[test]
        fn rejects_a_module_id_that_is_not_kebab_case() {
            for id in ["", "Demo", "node_modules", "-demo", "demo-", "node--modules", "1demo", "de mo"] {
                assert_eq!(ModuleId::new(id), Err(IdError::NotKebabCase { id: id.into() }), "{id:?}");
            }
        }

        #[test]
        fn rejects_the_core_namespace_as_a_module_id() {
            assert_eq!(ModuleId::new("core"), Err(IdError::Reserved { id: "core".into() }));
        }

        #[test]
        fn rejects_a_category_id_outside_its_module_namespace() {
            let demo = ModuleId::new("demo").unwrap();

            for id in ["docker.images", "caches", "demos.caches", ".caches"] {
                assert!(matches!(CategoryId::new(demo, id), Err(IdError::OutsideModule { .. })), "{id:?}");
            }
        }

        #[test]
        fn rejects_a_category_name_that_is_not_kebab_case() {
            let demo = ModuleId::new("demo").unwrap();

            for id in ["demo.", "demo.Caches", "demo.old.caches"] {
                assert!(matches!(CategoryId::new(demo, id), Err(IdError::NotKebabCase { .. })), "{id:?}");
            }
        }

        #[test]
        fn check_refuses_what_new_refuses() {
            assert_eq!(ModuleId::check("Demo"), Err(IdError::NotKebabCase { id: "Demo".into() }));
            assert_eq!(ModuleId::check("core"), Err(IdError::Reserved { id: "core".into() }));
            assert!(matches!(CategoryId::check("demo", "docker.images"), Err(IdError::OutsideModule { .. })));
            assert!(matches!(CategoryId::check("demo", "demo.Caches"), Err(IdError::NotKebabCase { .. })));
        }

        #[test]
        fn rejects_a_tool_id_that_is_not_kebab_case() {
            assert!(ToolId::new("Docker").is_err());
        }

        #[test]
        fn a_time_before_the_epoch_is_the_epoch() {
            let before = std::time::SystemTime::UNIX_EPOCH - std::time::Duration::from_secs(5);

            assert_eq!(crate::Timestamp::from_system_time(before).secs(), 0);
        }
    }

    mod positive {
        use std::time::{Duration, SystemTime};

        use crate::{CategoryId, FindingId, FindingKey, ModuleId, ScanId, Timestamp, ToolId};

        #[test]
        fn accepts_kebab_case_ids() {
            let module = ModuleId::new("node-modules").unwrap();
            let category = CategoryId::new(module, "node-modules.stale-2").unwrap();

            assert_eq!(module.to_string(), "node-modules");
            assert_eq!(category.to_string(), "node-modules.stale-2");
            assert_eq!(category.as_str(), "node-modules.stale-2");
            assert_eq!(category.module(), module);
            assert_eq!(ToolId::new("docker").unwrap().as_str(), "docker");
        }

        #[test]
        fn check_accepts_what_new_accepts() {
            assert_eq!(ModuleId::check("node-modules"), Ok(()));
            assert_eq!(CategoryId::check("node-modules", "node-modules.stale-2"), Ok(()));
        }

        #[test]
        fn numeric_ids_keep_their_value() {
            assert_eq!(FindingId::new(7).get(), 7);
            assert_eq!(FindingId::new(7).to_string(), "7");
            assert_eq!(ScanId::new(3).get(), 3);
            assert_eq!(Timestamp::from_secs(9).secs(), 9);
            assert_eq!(Timestamp::from_system_time(SystemTime::UNIX_EPOCH + Duration::from_millis(1500)).secs(), 1);
        }

        #[test]
        fn a_finding_key_names_its_module_and_identity() {
            let key = FindingKey::new(ModuleId::new("demo").unwrap(), "a");

            assert_eq!(key.module().as_str(), "demo");
            assert_eq!(key.identity(), "a");
            assert_eq!(serde_json::to_value(&key).unwrap(), serde_json::json!({ "module": "demo", "identity": "a" }));
        }
    }
}
