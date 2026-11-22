//! The module's categories and their danger levels (design, section 5): code, since a category id is a
//! `&'static str`; `places.toml` names them by id and is checked against this list.

use fresher_core::DangerLevel;

/// Every category, in the order the module lists them.
pub(crate) const CATEGORIES: [(&str, DangerLevel); 4] = [
    ("xcode.device-support", DangerLevel::Low),
    ("xcode.previews", DangerLevel::Safe),
    ("xcode.simulator-caches", DangerLevel::Safe),
    ("xcode.archives", DangerLevel::High),
];
