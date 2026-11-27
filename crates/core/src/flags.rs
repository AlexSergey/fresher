//! What a build ships with: the flags set before the build, never at runtime.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::HighLevel;

/// What a build ships with. It is set before the build from `flags.toml` by the app's build script (plan 043) and
/// never changes at runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct BuildFlags {
    /// Whether the engine offers High findings.
    pub high: HighLevel,
    /// Whether the sidebar shows Largest.
    pub largest: LargestScreen,
}

/// Whether the sidebar shows the Largest screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum LargestScreen {
    /// The sidebar shows Largest.
    Shown,
    /// The sidebar does not show Largest.
    Hidden,
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::LargestScreen;

        #[test]
        fn largest_refuses_an_unknown_value() {
            assert!(serde_json::from_str::<LargestScreen>("\"off\"").is_err());
        }
    }

    mod positive {
        use crate::{BuildFlags, HighLevel, LargestScreen};

        #[test]
        fn largest_serializes_as_kebab_case() {
            assert_eq!(serde_json::to_string(&LargestScreen::Shown).unwrap(), "\"shown\"");
            assert_eq!(serde_json::to_string(&LargestScreen::Hidden).unwrap(), "\"hidden\"");
            assert_eq!(serde_json::from_str::<LargestScreen>("\"shown\"").unwrap(), LargestScreen::Shown);
            assert_eq!(serde_json::from_str::<LargestScreen>("\"hidden\"").unwrap(), LargestScreen::Hidden);
        }

        #[test]
        fn build_flags_serialize_as_the_ui_reads_them() {
            let flags = BuildFlags { high: HighLevel::Withheld, largest: LargestScreen::Hidden };
            assert_eq!(serde_json::to_string(&flags).unwrap(), r#"{"high":"withheld","largest":"hidden"}"#);
        }
    }
}
