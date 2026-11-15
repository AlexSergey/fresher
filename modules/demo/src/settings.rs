//! The demo module's one setting: how many example findings it shows.

use fresher_core::{ModuleId, ModuleSettings, SettingDescriptor, SettingKey, SettingKind, SettingValue};

use crate::DemoError;

/// The setting's name after `demo.`.
const COUNT: &str = "count";
const MIN: i64 = 1;
const MAX: i64 = 50;
const DEFAULT: i64 = 8;

/// `demo.count`: a number from 1 to 50, 8 by default.
pub(crate) fn count_setting(module: ModuleId) -> Result<SettingDescriptor, DemoError> {
    let key = SettingKey::new(format!("{module}.{COUNT}"))?;
    let kind = SettingKind::Number { min: MIN, max: MAX, unit: None };
    Ok(SettingDescriptor::new(key, kind, SettingValue::Number(DEFAULT))?)
}

/// How many findings to show; a value the engine did not resolve counts as the default.
pub(crate) fn count(settings: &ModuleSettings) -> usize {
    let count = settings.number(COUNT).unwrap_or(DEFAULT).clamp(MIN, MAX);
    usize::try_from(count).unwrap_or(1)
}
