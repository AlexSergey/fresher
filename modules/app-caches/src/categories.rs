//! The module's categories and their danger levels (design, section 5): code, since a category id is a
//! `&'static str`; `places.toml` names them by id and is checked against this list.

use fresher_core::DangerLevel;

/// Every category, in the order the module lists them. Low, not Safe: the module cannot tell per app whether a
/// re-download follows (offline music, maps, a pending update).
pub(crate) const CATEGORIES: [(&str, DangerLevel); 1] = [("app-caches.library-caches", DangerLevel::Low)];
