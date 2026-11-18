//! The module's one setting: after how many days unchanged the files of an app that is no longer installed are
//! offered.

use fresher_core::{ModuleId, ModuleSettings, SettingDescriptor, SettingKey, SettingKind, SettingValue, Unit};

use crate::AppLeftoversError;

/// The setting's name after `app-leftovers.`.
const UNINSTALLED_AFTER_DAYS: &str = "uninstalled-after-days";
/// Never fewer than a week: a removed app's files are not offered the day after.
const MIN: i64 = 7;
const MAX: i64 = 3650;
const DEFAULT: u32 = 30;

/// `app-leftovers.uninstalled-after-days`: a number of days from 7 to 3650, 30 by default.
pub(crate) fn uninstalled_after_days_setting(module: ModuleId) -> Result<SettingDescriptor, AppLeftoversError> {
    let key = SettingKey::new(format!("{module}.{UNINSTALLED_AFTER_DAYS}"))?;
    let kind = SettingKind::Number { min: MIN, max: MAX, unit: Some(Unit::Days) };
    Ok(SettingDescriptor::new(key, kind, SettingValue::Number(i64::from(DEFAULT)))?)
}

/// The days; a value the engine did not resolve counts as the default. The schema already resolves a stored value
/// out of range, or not a number, to the default; the clamp only guards against a value that bypassed the schema.
pub(crate) fn uninstalled_after_days(settings: &ModuleSettings) -> u32 {
    let days = settings.number(UNINSTALLED_AFTER_DAYS).unwrap_or(i64::from(DEFAULT)).clamp(MIN, MAX);
    u32::try_from(days).unwrap_or(DEFAULT)
}

#[cfg(test)]
mod tests {
    use fresher_core::{ModuleId, ModuleSettings, SettingsSchema, StoredSettings};

    /// The module's settings resolved from the settings file's `stored` values.
    fn settings(stored: &str) -> ModuleSettings {
        let module = crate::AppLeftoversModule::new().unwrap();
        let stored: StoredSettings = serde_json::from_str(stored).unwrap();
        SettingsSchema::new([fresher_core::Module::descriptor(&module)])
            .unwrap()
            .resolve(&stored)
            .module(ModuleId::new("app-leftovers").unwrap())
    }

    mod negative {
        use fresher_core::{ModuleId, ModuleSettings};

        use super::settings;
        use crate::settings::uninstalled_after_days;

        #[test]
        fn a_value_that_is_not_resolved_or_that_the_schema_refuses_is_the_default() {
            assert_eq!(uninstalled_after_days(&ModuleSettings::empty(ModuleId::new("app-leftovers").unwrap())), 30);
            for stored in
                [r#"{"app-leftovers.uninstalled-after-days": 0}"#, r#"{"app-leftovers.uninstalled-after-days": "x"}"#]
            {
                assert_eq!(uninstalled_after_days(&settings(stored)), 30, "{stored}");
            }
        }
    }

    mod positive {
        use fresher_core::{ModuleId, SettingKind, SettingValue, Unit};

        use super::settings;
        use crate::settings::{uninstalled_after_days, uninstalled_after_days_setting};

        #[test]
        fn the_setting_is_a_number_of_days_thirty_by_default() {
            let setting = uninstalled_after_days_setting(ModuleId::new("app-leftovers").unwrap()).unwrap();

            assert_eq!(setting.key().as_str(), "app-leftovers.uninstalled-after-days");
            assert_eq!(setting.kind(), &SettingKind::Number { min: 7, max: 3650, unit: Some(Unit::Days) });
            assert_eq!(setting.default(), &SettingValue::Number(30));
            assert_eq!(uninstalled_after_days(&settings("{}")), 30);
        }

        #[test]
        fn a_stored_value_is_read() {
            assert_eq!(uninstalled_after_days(&settings(r#"{"app-leftovers.uninstalled-after-days": 90}"#)), 90);
        }
    }
}
