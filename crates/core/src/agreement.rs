//! The agreement the user accepts before the first cleanup: its version, its lines and what was accepted.
//!
//! Changing a line means raising [`AGREEMENT_VERSION`] and adding the text `core.agreement.changes.v<n>` (what changed
//! in version `n`), in the same change: a user who accepted an older version is asked again and told what changed.

use serde::Serialize;
use ts_rs::TS;

use crate::{Text, TextKey, Timestamp};

/// The version of the agreement's text.
pub const AGREEMENT_VERSION: u32 = 1;

/// The agreement's lines, in order: each is the text `core.agreement.line.<id>`.
pub const AGREEMENT_LINES: [&str; 5] = ["no-warranty", "your-decision", "best-judgement", "backup", "trash"];

/// An acceptance of the agreement: which version, and when.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Acceptance {
    pub version: u32,
    pub at: Timestamp,
}

/// The answer of `get_agreement`: the current agreement's texts and what the user accepted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AgreementView {
    /// The current version.
    pub version: u32,
    pub title: Text,
    pub intro: Text,
    pub lines: Vec<Text>,
    /// The last acceptance, of whatever version; `None` when the agreement was never accepted.
    pub accepted: Option<Acceptance>,
    /// What changed since the accepted version, when an older one was accepted.
    pub changes: Option<Text>,
}

impl AgreementView {
    /// The view of the current agreement.
    pub fn new(accepted: Option<Acceptance>) -> Self {
        Self::of(AGREEMENT_VERSION, accepted)
    }

    /// The view of version `current` of the agreement.
    pub(crate) fn of(current: u32, accepted: Option<Acceptance>) -> Self {
        let text = |key: String| Text::new(TextKey::from_valid(key));
        let older = accepted.is_some_and(|acceptance| acceptance.version > 0 && acceptance.version < current);
        Self {
            version: current,
            title: text("core.agreement.title".into()),
            intro: text("core.agreement.intro".into()),
            lines: AGREEMENT_LINES.iter().map(|id| text(format!("core.agreement.line.{id}"))).collect(),
            accepted,
            changes: older.then(|| text(format!("core.agreement.changes.v{current}"))),
        }
    }

    /// Whether the current version is the one accepted.
    pub fn is_accepted(&self) -> bool {
        self.accepted.is_some_and(|acceptance| acceptance.version == self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::{Acceptance, AgreementView};
    use crate::Timestamp;

    fn accepted(version: u32) -> Acceptance {
        Acceptance { version, at: Timestamp::from_secs(1_000) }
    }

    mod negative {
        use super::{AgreementView, accepted};

        #[test]
        fn an_unaccepted_agreement_is_not_accepted() {
            let view = AgreementView::new(None);

            assert!(!view.is_accepted());
            assert_eq!(view.changes, None);
        }

        #[test]
        fn an_older_acceptance_is_not_accepted_and_names_what_changed() {
            let view = AgreementView::of(2, Some(accepted(1)));

            assert!(!view.is_accepted());
            assert_eq!(
                view.changes.map(|text| text.key().as_str().to_owned()),
                Some("core.agreement.changes.v2".into())
            );
        }

        #[test]
        fn a_version_0_acceptance_shows_no_changes() {
            let view = AgreementView::of(2, Some(accepted(0)));

            assert!(!view.is_accepted());
            assert_eq!(view.changes, None);
        }
    }

    mod positive {
        use serde_json::json;

        use super::{AgreementView, accepted};
        use crate::AGREEMENT_VERSION;

        #[test]
        fn the_current_acceptance_is_accepted() {
            let view = AgreementView::new(Some(accepted(AGREEMENT_VERSION)));

            assert!(view.is_accepted());
            assert_eq!(view.changes, None);
            assert_eq!(serde_json::to_value(&view).unwrap()["accepted"], json!({ "version": 1, "at": 1_000 }));
        }

        #[test]
        fn the_lines_are_the_five_keys_in_order() {
            let view = AgreementView::new(None);
            let keys: Vec<_> = view.lines.iter().map(|text| text.key().as_str()).collect();

            assert_eq!(
                keys,
                [
                    "core.agreement.line.no-warranty",
                    "core.agreement.line.your-decision",
                    "core.agreement.line.best-judgement",
                    "core.agreement.line.backup",
                    "core.agreement.line.trash",
                ]
            );
            assert_eq!(view.title.key().as_str(), "core.agreement.title");
            assert_eq!(view.intro.key().as_str(), "core.agreement.intro");
        }
    }
}
