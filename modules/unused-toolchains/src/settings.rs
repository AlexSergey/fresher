//! The module's one setting: whether Node versions are offered. Off by default (the user's decision of 2026-11-19).

use fresher_core::{ModuleId, ModuleSettings, SettingDescriptor, SettingError, SettingKey, SettingKind, SettingValue};

/// The switch's name after `unused-toolchains.`.
pub(crate) const NODE_VERSIONS: &str = "node-versions";

/// `unused-toolchains.node-versions`: a switch, off by default.
pub(crate) fn node_versions_setting(module: ModuleId) -> Result<SettingDescriptor, SettingError> {
    let key = SettingKey::new(format!("{module}.{NODE_VERSIONS}"))?;
    SettingDescriptor::new(key, SettingKind::Switch, SettingValue::Switch(false))
}

/// Whether the module's switch `switch` (the name after `unused-toolchains.`) is on; a value the engine did not
/// resolve counts as off.
pub(crate) fn is_on(settings: &ModuleSettings, switch: &str) -> bool {
    settings.switch(switch).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{ModuleId, ModuleSettings};

        use crate::settings::{NODE_VERSIONS, is_on};

        #[test]
        fn an_unresolved_switch_counts_as_off() {
            let settings = ModuleSettings::empty(ModuleId::new("unused-toolchains").unwrap());

            assert!(!is_on(&settings, NODE_VERSIONS));
        }
    }

    mod positive {
        use fresher_core::{ModuleId, SettingValue};

        use crate::settings::node_versions_setting;

        #[test]
        fn the_switch_is_off_by_default() {
            let setting = node_versions_setting(ModuleId::new("unused-toolchains").unwrap()).unwrap();

            assert_eq!(setting.key().as_str(), "unused-toolchains.node-versions");
            assert_eq!(setting.default(), &SettingValue::Switch(false));
        }
    }
}
