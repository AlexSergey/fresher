//! The module's categories and their danger levels (plan 030 design, section 5; plan 034 design, section 5): code,
//! since a category id is a `&'static str`; `places.toml` and `apps.toml` name them by id and are checked against
//! their own list.

use fresher_core::DangerLevel;

/// The categories of `places.toml`. Low, not Safe: the module cannot tell per app whether a re-download follows
/// (offline music, maps, a pending update).
pub(crate) const PLACE_CATEGORIES: [(&str, DangerLevel); 1] = [("app-caches.library-caches", DangerLevel::Low)];

/// The categories of `apps.toml`. Low: a browser downloads pictures and videos again, a web-engine app starts slower.
pub(crate) const APP_CATEGORIES: [(&str, DangerLevel); 2] =
    [("app-caches.browsers", DangerLevel::Low), ("app-caches.web-engine", DangerLevel::Low)];

/// Every category, in the order the module lists them: the places' then the table's.
pub(crate) const CATEGORIES: [(&str, DangerLevel); 3] = [
    ("app-caches.library-caches", DangerLevel::Low),
    ("app-caches.browsers", DangerLevel::Low),
    ("app-caches.web-engine", DangerLevel::Low),
];
