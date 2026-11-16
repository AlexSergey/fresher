//! The core's own settings.

use std::path::PathBuf;

use super::{SettingDescriptor, SettingEffect, SettingGroup, SettingKey, SettingKind, SettingValue};

/// How much the interface moves: `system` follows macOS's "Reduce motion", `reduced` keeps only fades.
pub(crate) const MOTION: &str = "core.motion";

/// The folders a scan walks; `~` is the home folder.
pub const SCAN_ROOTS: &str = "core.scan.roots";

/// The opt-ins into the places the system guards: on, the next scan tries the place, and the system may ask the user
/// once. Off by default.
pub(crate) const ACCESS_OPT_INS: [&str; 4] =
    ["core.access.desktop", "core.access.documents", "core.access.downloads", "core.access.other-apps-data"];

/// The categories the user ignores, by id: their findings are left out of the results.
pub const IGNORE_CATEGORIES: &str = "core.ignore.categories";

/// The paths the user ignores: a finding at or below one is left out, and nothing that holds one is removed.
pub const IGNORE_PATHS: &str = "core.ignore.paths";

/// The settings of the core, before every module's, in the order the screen shows them. None of them can switch off
/// the confirmation before a removal, lower a danger level or choose how items are removed: such settings do not
/// exist.
pub(super) fn core_settings() -> Vec<SettingDescriptor> {
    let setting = |key: &str, kind, default, group, effect| {
        SettingDescriptor::unchecked(SettingKey(key.into()), kind, default, group, effect)
    };
    let motion = setting(
        MOTION,
        SettingKind::Choice { options: vec!["system", "reduced"] },
        SettingValue::Choice("system".into()),
        SettingGroup::General,
        SettingEffect::Now,
    );
    let roots = setting(
        SCAN_ROOTS,
        SettingKind::Folders,
        SettingValue::Folders(vec![PathBuf::from("~")]),
        SettingGroup::Scan,
        SettingEffect::NextScan,
    );
    let opt_in = |key: &str| {
        setting(
            key,
            SettingKind::Switch,
            SettingValue::Switch(false),
            SettingGroup::Permissions,
            SettingEffect::NextScan,
        )
    };
    let categories = setting(
        IGNORE_CATEGORIES,
        SettingKind::Ids,
        SettingValue::Ids(vec![]),
        SettingGroup::Ignored,
        SettingEffect::Now,
    );
    let paths = setting(
        IGNORE_PATHS,
        SettingKind::Paths,
        SettingValue::Paths(vec![]),
        SettingGroup::Ignored,
        SettingEffect::Now,
    );
    [motion, roots].into_iter().chain(ACCESS_OPT_INS.map(opt_in)).chain([categories, paths]).collect()
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
        fn has_no_key_that_turns_off_the_confirmation_lowers_a_danger_level_or_chooses_the_removal() {
            for setting in core_settings() {
                let key = setting.key().as_str();
                for word in ["confirm", "danger", "level", "removal", "remove", "delete", "permanent", "trash"] {
                    assert!(!key.contains(word), "{key}");
                }
            }
        }

        #[test]
        fn no_core_setting_is_in_the_modules_group() {
            for setting in core_settings() {
                assert_ne!(setting.group(), crate::SettingGroup::Module, "{}", setting.key());
            }
        }

        #[test]
        fn a_module_setting_is_never_in_a_core_group() {
            let kind = crate::SettingKind::Switch;
            let module = crate::SettingDescriptor::new(
                crate::SettingKey::new("demo.ignore.paths").unwrap(),
                kind,
                crate::SettingValue::Switch(true),
            )
            .unwrap();

            assert_eq!(module.group(), crate::SettingGroup::Module);
            assert_eq!(module.effect(), crate::SettingEffect::NextScan);
        }
    }

    mod positive {
        use super::super::core_settings;
        use crate::{SettingDescriptor, SettingEffect, SettingGroup, SettingKey, SettingValue};

        #[test]
        fn every_core_setting_is_valid_and_in_the_core_namespace() {
            for setting in core_settings() {
                let key = SettingKey::new(setting.key().as_str()).unwrap();
                let rebuilt = SettingDescriptor::new(key, setting.kind().clone(), setting.default().clone()).unwrap();
                assert_eq!(rebuilt.default(), setting.default());
                assert_eq!(setting.key().namespace(), "core");
            }
        }

        #[test]
        fn every_core_setting_has_its_group_and_effect() {
            let placed: Vec<_> = core_settings()
                .iter()
                .map(|setting| (setting.key().as_str().to_owned(), setting.group(), setting.effect()))
                .collect();
            let at = |key: &str, group, effect| (key.to_owned(), group, effect);

            assert_eq!(
                placed,
                [
                    at("core.motion", SettingGroup::General, SettingEffect::Now),
                    at("core.scan.roots", SettingGroup::Scan, SettingEffect::NextScan),
                    at("core.access.desktop", SettingGroup::Permissions, SettingEffect::NextScan),
                    at("core.access.documents", SettingGroup::Permissions, SettingEffect::NextScan),
                    at("core.access.downloads", SettingGroup::Permissions, SettingEffect::NextScan),
                    at("core.access.other-apps-data", SettingGroup::Permissions, SettingEffect::NextScan),
                    at("core.ignore.categories", SettingGroup::Ignored, SettingEffect::Now),
                    at("core.ignore.paths", SettingGroup::Ignored, SettingEffect::Now),
                ]
            );
        }

        #[test]
        fn motion_follows_the_system_and_nothing_is_ignored_by_default() {
            let settings = core_settings();
            let default = |key: &str| {
                settings.iter().find(|setting| setting.key().as_str() == key).map(SettingDescriptor::default)
            };

            assert_eq!(default("core.motion"), Some(&SettingValue::Choice("system".into())));
            assert_eq!(default("core.ignore.categories"), Some(&SettingValue::Ids(vec![])));
            assert_eq!(default("core.ignore.paths"), Some(&SettingValue::Paths(vec![])));
        }
    }
}
