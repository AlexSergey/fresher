//! The core's own settings.

use std::path::PathBuf;

use crate::ByteSize;

use super::{IdOf, SettingDescriptor, SettingEffect, SettingGroup, SettingKey, SettingKind, SettingValue};

/// How much the interface moves: `system` follows macOS's "Reduce motion", `reduced` keeps only fades.
pub(crate) const MOTION: &str = "core.motion";

/// The folders a scan walks; `~` is the home folder.
pub const SCAN_ROOTS: &str = "core.scan.roots";

/// The smallest size the Largest screen lists: a choice of `10-mb`, `50-mb`, `100-mb`, `500-mb` or `1-gb`.
pub const LARGEST_MIN_SIZE: &str = "core.largest.min-size";

/// The choice of [`LARGEST_MIN_SIZE`] used when none is stored.
const MIN_SIZE_DEFAULT: &str = "100-mb";

/// The sizes of [`LARGEST_MIN_SIZE`]'s choices, in decimal units (10 MB is 10 000 000 bytes, as the Finder counts).
const MIN_SIZES: [(&str, u64); 5] = [
    ("10-mb", 10_000_000),
    ("50-mb", 50_000_000),
    (MIN_SIZE_DEFAULT, 100_000_000),
    ("500-mb", 500_000_000),
    ("1-gb", 1_000_000_000),
];

/// The size a choice of [`LARGEST_MIN_SIZE`] stands for; an unknown choice is the default's.
pub fn min_size_of(choice: &str) -> ByteSize {
    let bytes_of = |name: &str| MIN_SIZES.iter().find(|(known, _)| *known == name).map(|(_, bytes)| *bytes);
    ByteSize::new(bytes_of(choice).or_else(|| bytes_of(MIN_SIZE_DEFAULT)).unwrap_or_default())
}

/// The opt-ins into the places the system guards: on, the next scan tries the place, and the system may ask the user
/// once. Off by default.
pub(crate) const ACCESS_OPT_INS: [&str; 4] =
    ["core.access.desktop", "core.access.documents", "core.access.downloads", "core.access.other-apps-data"];

/// The modules the user ignores, by id: their findings are left out of the results.
pub const IGNORE_MODULES: &str = "core.ignore.modules";

/// The groups the user ignores, by name: the findings of every module of one are left out of the results.
pub const IGNORE_GROUPS: &str = "core.ignore.groups";

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
    let min_size = setting(
        LARGEST_MIN_SIZE,
        SettingKind::Choice { options: MIN_SIZES.iter().map(|(name, _)| *name).collect() },
        SettingValue::Choice(MIN_SIZE_DEFAULT.into()),
        SettingGroup::Scan,
        SettingEffect::Now,
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
    let ids = |key: &str, of| {
        setting(key, SettingKind::Ids { of }, SettingValue::Ids(vec![]), SettingGroup::Ignored, SettingEffect::Now)
    };
    let paths = setting(
        IGNORE_PATHS,
        SettingKind::Paths,
        SettingValue::Paths(vec![]),
        SettingGroup::Ignored,
        SettingEffect::Now,
    );
    let ignored =
        [ids(IGNORE_MODULES, IdOf::Module), ids(IGNORE_GROUPS, IdOf::Group), ids(IGNORE_CATEGORIES, IdOf::Category)];
    [motion, roots, min_size].into_iter().chain(ACCESS_OPT_INS.map(opt_in)).chain(ignored).chain([paths]).collect()
}

#[cfg(test)]
mod tests {
    mod negative {
        use super::super::{core_settings, min_size_of};
        use crate::ByteSize;

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
        fn an_unknown_min_size_choice_is_the_default() {
            assert_eq!(min_size_of("2-gb"), ByteSize::new(100_000_000));
            assert_eq!(min_size_of(""), min_size_of("100-mb"));
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
        use super::super::{core_settings, min_size_of};
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
                    at("core.largest.min-size", SettingGroup::Scan, SettingEffect::Now),
                    at("core.access.desktop", SettingGroup::Permissions, SettingEffect::NextScan),
                    at("core.access.documents", SettingGroup::Permissions, SettingEffect::NextScan),
                    at("core.access.downloads", SettingGroup::Permissions, SettingEffect::NextScan),
                    at("core.access.other-apps-data", SettingGroup::Permissions, SettingEffect::NextScan),
                    at("core.ignore.modules", SettingGroup::Ignored, SettingEffect::Now),
                    at("core.ignore.groups", SettingGroup::Ignored, SettingEffect::Now),
                    at("core.ignore.categories", SettingGroup::Ignored, SettingEffect::Now),
                    at("core.ignore.paths", SettingGroup::Ignored, SettingEffect::Now),
                ]
            );
        }

        #[test]
        fn the_min_size_choices_are_decimal() {
            let sizes = ["10-mb", "50-mb", "100-mb", "500-mb", "1-gb"].map(|choice| min_size_of(choice).bytes());
            let options = core_settings()
                .into_iter()
                .find(|setting| setting.key().as_str() == "core.largest.min-size")
                .map(|setting| (setting.kind().clone(), setting.default().clone()));

            assert_eq!(sizes, [10_000_000, 50_000_000, 100_000_000, 500_000_000, 1_000_000_000]);
            assert_eq!(
                options,
                Some((
                    crate::SettingKind::Choice { options: vec!["10-mb", "50-mb", "100-mb", "500-mb", "1-gb"] },
                    SettingValue::Choice("100-mb".into())
                ))
            );
        }

        #[test]
        fn motion_follows_the_system_and_nothing_is_ignored_by_default() {
            let settings = core_settings();
            let default = |key: &str| {
                settings.iter().find(|setting| setting.key().as_str() == key).map(SettingDescriptor::default)
            };

            assert_eq!(default("core.motion"), Some(&SettingValue::Choice("system".into())));
            assert_eq!(default("core.ignore.modules"), Some(&SettingValue::Ids(vec![])));
            assert_eq!(default("core.ignore.groups"), Some(&SettingValue::Ids(vec![])));
            assert_eq!(default("core.ignore.categories"), Some(&SettingValue::Ids(vec![])));
            assert_eq!(default("core.ignore.paths"), Some(&SettingValue::Paths(vec![])));
        }
    }
}
