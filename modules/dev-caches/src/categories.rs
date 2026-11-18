//! The module's categories and their danger levels (design, section 4): code, since a category id is a
//! `&'static str`; `tools.toml` names them by id and is checked against this list.

use fresher_core::DangerLevel;

/// Every category, in the order the module lists them: the downloads (Low: offline, the next install fails) first,
/// then what a tool builds again with no download (Safe).
pub(crate) const CATEGORIES: [(&str, DangerLevel); 9] = [
    ("dev-caches.npm", DangerLevel::Low),
    ("dev-caches.yarn", DangerLevel::Low),
    ("dev-caches.homebrew", DangerLevel::Low),
    ("dev-caches.cargo", DangerLevel::Low),
    ("dev-caches.cocoapods", DangerLevel::Low),
    ("dev-caches.gradle", DangerLevel::Low),
    ("dev-caches.pip", DangerLevel::Low),
    ("dev-caches.go-build", DangerLevel::Safe),
    ("dev-caches.xcode-derived-data", DangerLevel::Safe),
];
