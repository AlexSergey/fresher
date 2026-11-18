//! The module's one setting: after how many days unchanged a temporary item is offered.

use fresher_core::{ModuleId, ModuleSettings, SettingDescriptor, SettingKey, SettingKind, SettingValue, Unit};

use crate::TempError;

/// The setting's name after `temp.`.
const OLDER_THAN_DAYS: &str = "older-than-days";
/// The check can only become stricter: never fewer than a week.
const MIN: i64 = 7;
const MAX: i64 = 30;
const DEFAULT: u32 = 7;

/// `temp.older-than-days`: a number of days from 7 to 30, 7 by default.
pub(crate) fn older_than_days_setting(module: ModuleId) -> Result<SettingDescriptor, TempError> {
    let key = SettingKey::new(format!("{module}.{OLDER_THAN_DAYS}"))?;
    let kind = SettingKind::Number { min: MIN, max: MAX, unit: Some(Unit::Days) };
    Ok(SettingDescriptor::new(key, kind, SettingValue::Number(i64::from(DEFAULT)))?)
}

/// The days; a value the engine did not resolve counts as the default, one out of range as its nearest limit.
pub(crate) fn older_than_days(settings: &ModuleSettings) -> u32 {
    let days = settings.number(OLDER_THAN_DAYS).unwrap_or(i64::from(DEFAULT)).clamp(MIN, MAX);
    u32::try_from(days).unwrap_or(DEFAULT)
}

#[cfg(test)]
mod tests {
    use fresher_core::{Module as _, ModuleSettings, SettingsSchema, StoredSettings};

    /// The module's settings resolved from the settings file's `stored` values.
    fn settings(stored: &str) -> ModuleSettings {
        let module = crate::TempModule::new().unwrap();
        let stored: StoredSettings = serde_json::from_str(stored).unwrap();
        SettingsSchema::new([module.descriptor()]).unwrap().resolve(&stored).module(module.descriptor().id())
    }

    mod negative {
        use fresher_core::{ModuleId, ModuleSettings};

        use super::settings;
        use crate::settings::older_than_days;

        #[test]
        fn a_value_that_is_not_resolved_is_the_default() {
            assert_eq!(older_than_days(&ModuleSettings::empty(ModuleId::new("temp").unwrap())), 7);
        }

        #[test]
        fn a_value_out_of_range_or_not_a_number_is_refused_for_the_default() {
            for stored in ["6", "31", "0", "-1", r#""x""#, "7.5"] {
                let stored = format!(r#"{{"temp.older-than-days": {stored}}}"#);

                assert_eq!(older_than_days(&settings(&stored)), 7, "{stored}");
            }
        }
    }

    mod positive {
        use fresher_core::{ModuleId, SettingKind, SettingValue, Unit};

        use super::settings;
        use crate::settings::{older_than_days, older_than_days_setting};

        #[test]
        fn the_setting_is_seven_to_thirty_days_seven_by_default() {
            let setting = older_than_days_setting(ModuleId::new("temp").unwrap()).unwrap();

            assert_eq!(setting.key().as_str(), "temp.older-than-days");
            assert_eq!(setting.kind(), &SettingKind::Number { min: 7, max: 30, unit: Some(Unit::Days) });
            assert_eq!(setting.default(), &SettingValue::Number(7));
        }

        #[test]
        fn a_value_in_range_is_read() {
            for days in [7, 14, 30] {
                let stored = format!(r#"{{"temp.older-than-days": {days}}}"#);

                assert_eq!(older_than_days(&settings(&stored)), days, "{stored}");
            }
            assert_eq!(older_than_days(&settings("{}")), 7);
        }
    }
}
