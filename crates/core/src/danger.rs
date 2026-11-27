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

/// Whether the engine offers High findings at all. The app sets the switch when it starts, from the build's `high`
/// flag (`flags.toml`); tests and the sandbox autotest name it explicitly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum HighLevel {
    /// Every level is offered.
    Offered,
    /// No High finding leaves the engine, and every way to remove one is refused.
    Withheld,
}

impl HighLevel {
    /// Whether a finding or a category at `level` is offered: false only for `High` while withheld.
    pub const fn offers(self, level: DangerLevel) -> bool {
        !matches!((self, level), (Self::Withheld, DangerLevel::High))
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{DangerLevel, HighLevel};

        #[test]
        fn withheld_does_not_offer_high() {
            assert!(!HighLevel::Withheld.offers(DangerLevel::High));
        }
    }

    mod positive {
        use crate::{DangerLevel, HighLevel};

        #[test]
        fn offered_offers_every_level() {
            assert!(DangerLevel::ALL.iter().all(|level| HighLevel::Offered.offers(*level)));
        }

        #[test]
        fn withheld_offers_safe_to_medium() {
            for level in [DangerLevel::Safe, DangerLevel::Low, DangerLevel::Medium] {
                assert!(HighLevel::Withheld.offers(level), "{level:?}");
            }
        }

        #[test]
        fn the_switch_serializes_as_kebab_case() {
            assert_eq!(serde_json::to_value(HighLevel::Withheld).unwrap(), "withheld");
            assert_eq!(serde_json::from_str::<HighLevel>("\"offered\"").unwrap(), HighLevel::Offered);
        }

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
