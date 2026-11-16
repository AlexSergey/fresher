//! The core's own settings.

use std::path::PathBuf;

use super::{SettingDescriptor, SettingKey, SettingKind, SettingValue};

/// The folders a scan walks; `~` is the home folder.
pub(crate) const SCAN_ROOTS: &str = "core.scan.roots";

/// The opt-ins into the places the system guards: on, the next scan tries the place, and the system may ask the user
/// once. Off by default.
pub(crate) const ACCESS_OPT_INS: [&str; 4] =
    ["core.access.desktop", "core.access.documents", "core.access.downloads", "core.access.other-apps-data"];

/// The settings of the core, before every module's. None of them can switch off the confirmation before a removal or
/// lower a danger level: such settings do not exist.
pub(super) fn core_settings() -> Vec<SettingDescriptor> {
    let roots = SettingDescriptor {
        key: SettingKey(SCAN_ROOTS.into()),
        kind: SettingKind::Folders,
        default: SettingValue::Folders(vec![PathBuf::from("~")]),
    };
    let opt_in = |key: &str| SettingDescriptor {
        key: SettingKey(key.into()),
        kind: SettingKind::Switch,
        default: SettingValue::Switch(false),
    };
    std::iter::once(roots).chain(ACCESS_OPT_INS.map(opt_in)).collect()
}

#[cfg(test)]
mod tests {
    mod negative {
        use super::super::core_settings;

        #[test]
        fn the_opt_in_switches_are_off_by_default() {
            let opt_ins: Vec<_> = core_settings()
                .into_iter()
                .filter(|setting| setting.key().as_str().starts_with("core.access."))
                .collect();

            assert_eq!(opt_ins.len(), 4);
            for setting in opt_ins {
                assert_eq!(setting.default(), &crate::SettingValue::Switch(false), "{}", setting.key().as_str());
            }
        }

        #[test]
        fn has_no_key_that_turns_off_the_confirmation_or_lowers_a_danger_level() {
            for setting in core_settings() {
                let key = setting.key().as_str();
                assert!(!key.contains("confirm") && !key.contains("danger"), "{key}");
            }
        }
    }

    mod positive {
        use super::super::core_settings;
        use crate::{SettingDescriptor, SettingKey};

        #[test]
        fn every_core_setting_is_valid_and_in_the_core_namespace() {
            for setting in core_settings() {
                let key = SettingKey::new(setting.key().as_str()).unwrap();
                let rebuilt = SettingDescriptor::new(key, setting.kind().clone(), setting.default().clone()).unwrap();
                assert_eq!(rebuilt, setting);
                assert_eq!(setting.key().namespace(), "core");
            }
        }
    }
}
