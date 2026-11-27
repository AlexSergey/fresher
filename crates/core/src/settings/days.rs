//! The "unchanged for N days" setting that several modules declare.

use super::{ModuleSettings, SettingDescriptor, SettingError, SettingKey, SettingKind, SettingValue, Unit};
use crate::ModuleId;

/// The module that declares [`STALE_PROJECTS_UNUSED_FOR`].
pub const STALE_PROJECTS_MODULE: &str = "stale-projects";

/// `stale-projects.unused-for-days`: declared by the stale projects module, read by the engine for Largest's Folders.
/// Never fewer than a month: a project is not stale the week after you left it.
pub const STALE_PROJECTS_UNUSED_FOR: DaySetting = DaySetting::new("unused-for-days", 30, 3650, 365);

/// A module's number-of-days setting: its name after `<module>.`, its range and its default. A module declares one as
/// a constant and uses it for both the descriptor and the reading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DaySetting {
    name: &'static str,
    min: u32,
    max: u32,
    default: u32,
}

impl DaySetting {
    pub const fn new(name: &'static str, min: u32, max: u32, default: u32) -> Self {
        Self { name, min, max, default }
    }

    /// The days when nothing is stored.
    pub const fn default_days(&self) -> u32 {
        self.default
    }

    /// `<module>.<name>`: a number of days from `min` to `max`. Refused when the name makes no key or the default is
    /// outside the range.
    pub fn descriptor(&self, module: ModuleId) -> Result<SettingDescriptor, SettingError> {
        let key = SettingKey::new(format!("{module}.{}", self.name))?;
        let kind = SettingKind::Number { min: i64::from(self.min), max: i64::from(self.max), unit: Some(Unit::Days) };
        SettingDescriptor::new(key, kind, SettingValue::Number(i64::from(self.default)))
    }

    /// The days; a value the engine did not resolve counts as the default, one out of range as its nearest limit.
    pub fn days(&self, settings: &ModuleSettings) -> u32 {
        let days = settings.number(self.name).unwrap_or(i64::from(self.default));
        let days = days.clamp(i64::from(self.min), i64::from(self.max));
        u32::try_from(days).unwrap_or(self.default)
    }
}

#[cfg(test)]
mod tests {
    use super::DaySetting;
    use crate::{Locales, ModuleDescriptor, ModuleGroup, ModuleId, ModuleSettings, SettingsSchema, StoredSettings};

    const SETTING: DaySetting = DaySetting::new("x-days", 7, 30, 7);

    fn demo() -> ModuleId {
        ModuleId::new("demo").unwrap()
    }

    /// The setting resolved through a one-setting module from the settings file's `stored` values.
    fn resolved(setting: &DaySetting, stored: &str) -> ModuleSettings {
        let settings = vec![setting.descriptor(demo()).unwrap()];
        let module = ModuleDescriptor::new(demo(), ModuleGroup::Debug, vec![], settings, Locales { en: "{}" }).unwrap();
        let stored: StoredSettings = serde_json::from_str(stored).unwrap();
        SettingsSchema::new([&module]).unwrap().resolve(&stored).module(demo())
    }

    mod negative {
        use super::{SETTING, demo, resolved};
        use crate::{DaySetting, ModuleSettings, SettingError};

        #[test]
        fn a_default_outside_the_range_is_refused() {
            let setting = DaySetting::new("x-days", 7, 30, 31);

            assert!(matches!(setting.descriptor(demo()), Err(SettingError::BadDefault { .. })));
        }

        #[test]
        fn a_name_that_makes_no_key_is_refused() {
            let setting = DaySetting::new("Bad_Name", 7, 30, 7);

            assert!(matches!(setting.descriptor(demo()), Err(SettingError::InvalidKey { .. })));
        }

        #[test]
        fn a_value_that_is_not_resolved_is_the_default() {
            assert_eq!(SETTING.days(&ModuleSettings::empty(demo())), 7);
        }

        #[test]
        fn a_stored_value_out_of_range_or_not_a_number_is_the_default() {
            for stored in ["0", "6", "31", "-1", r#""x""#, "7.5"] {
                let stored = format!(r#"{{"demo.x-days": {stored}}}"#);

                assert_eq!(SETTING.days(&resolved(&SETTING, &stored)), 7, "{stored}");
            }
        }
    }

    mod positive {
        use super::{SETTING, demo, resolved};
        use crate::{DaySetting, SettingKind, SettingValue, Unit};

        #[test]
        fn the_descriptor_is_a_number_of_days_in_its_range() {
            let descriptor = SETTING.descriptor(demo()).unwrap();

            assert_eq!(descriptor.key().as_str(), "demo.x-days");
            assert_eq!(descriptor.kind(), &SettingKind::Number { min: 7, max: 30, unit: Some(Unit::Days) });
            assert_eq!(descriptor.default(), &SettingValue::Number(7));
        }

        #[test]
        fn a_stored_value_in_range_is_read() {
            for days in [7, 14, 30] {
                let stored = format!(r#"{{"demo.x-days": {days}}}"#);

                assert_eq!(SETTING.days(&resolved(&SETTING, &stored)), days, "{stored}");
            }
            let wide = DaySetting::new("x-days", 7, 3650, 30);

            assert_eq!(wide.days(&resolved(&wide, r#"{"demo.x-days": 3650}"#)), 3650);
        }
    }
}
