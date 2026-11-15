//! How dangerous it is to remove something.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The danger level of a category or a finding, from `Safe` (comes back by itself) to `High` (the user's own data).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum DangerLevel {
    Safe,
    Low,
    Medium,
    High,
}

impl DangerLevel {
    /// Every level, from the safest to the most dangerous.
    pub const ALL: [Self; 4] = [Self::Safe, Self::Low, Self::Medium, Self::High];
}

#[cfg(test)]
mod tests {
    mod positive {
        use crate::DangerLevel;

        #[test]
        fn danger_levels_order_from_safe_to_high() {
            assert!(DangerLevel::ALL.is_sorted());
            assert!(DangerLevel::Safe < DangerLevel::High);
            assert_eq!(DangerLevel::ALL.iter().max(), Some(&DangerLevel::High));
        }

        #[test]
        fn serializes_as_kebab_case() {
            assert_eq!(serde_json::to_value(DangerLevel::Medium).unwrap(), "medium");
            assert_eq!(serde_json::from_str::<DangerLevel>("\"low\"").unwrap(), DangerLevel::Low);
        }
    }
}
