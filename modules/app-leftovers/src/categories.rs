//! The module's categories and their danger levels (design, section 4): code, since a category id is a
//! `&'static str`; `places.toml` names them by id and is checked against this list.

use fresher_core::DangerLevel;

/// Every category, in the order the module lists them: caches, logs and saved state (Safe), settings and web data
/// (Medium), then support folders and containers, where a sandboxed app keeps its documents (High).
pub(crate) const CATEGORIES: [(&str, DangerLevel); 3] = [
    ("app-leftovers.caches", DangerLevel::Safe),
    ("app-leftovers.settings", DangerLevel::Medium),
    ("app-leftovers.data", DangerLevel::High),
];
