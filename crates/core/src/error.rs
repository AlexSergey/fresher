//! The one error shape every command returns to the UI.

use serde::Serialize;
use ts_rs::TS;

use crate::{BlockedReason, Capability, CleanupId, FindingId, Operation, SettingKey, Text, TextArg, TextKey};

/// Why a command was refused; serialized with a `code` tag. The UI shows `text()`, never the `Display` form, which is
/// for logs.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "code", rename_all = "kebab-case")]
pub enum CommandError {
    /// One operation at a time.
    #[error("another operation is running: {}", .running.as_str())]
    Busy { running: Operation },
    /// A cancel with nothing to cancel.
    #[error("no operation is running")]
    NotRunning,
    #[error("finding {id} is not in this session")]
    UnknownFinding { id: FindingId },
    #[error("there is no module `{id}`")]
    UnknownModule { id: String },
    #[error("there is no setting `{key}`")]
    UnknownSetting { key: String },
    #[error("the value does not fit setting `{key}`")]
    InvalidSettingValue { key: SettingKey },
    #[error("there are no texts for locale `{locale}`")]
    UnknownLocale { locale: String },
    /// The settings file is from a newer version of the app: it is never overwritten.
    #[error("the settings are read-only: the file is from a newer version of the app")]
    SettingsReadOnly,
    /// The settings could not be written.
    #[error("the settings could not be saved")]
    Storage,
    /// A query of the disk state before any scan committed one.
    #[error("there is no scan to read")]
    NoScan,
    /// A node of an older state than the engine holds.
    #[error("the node is of an older scan")]
    StaleNode,
    /// An action whose need is not granted.
    #[error("the capability `{}` is missing", .capability.id())]
    MissingCapability { capability: Capability },
    /// A capability with no settings pane: a tool, or one this system lacks.
    #[error("the capability `{}` has no settings pane", .capability.id())]
    NoSettingsPane { capability: Capability },
    /// A bug: the operation stopped safely.
    #[error("an internal error stopped the operation")]
    Internal,
    /// A selection names a category that is not in this session.
    #[error("there is no category `{id}`")]
    UnknownCategory { id: String },
    /// A selection that picks nothing.
    #[error("nothing is selected")]
    NothingToClean,
    /// A selection that expands to more items than a plan may hold.
    #[error("the selection holds more than {limit} items")]
    TooManyItems { limit: u32 },
    /// A confirmation of a plan that is unknown, replaced, discarded or expired.
    #[error("the plan is gone")]
    PlanGone,
    /// A confirmation whose summary differs from the plan's: the UI showed something else. The plan is gone.
    #[error("the confirmation does not match the plan")]
    PlanMismatch,
    /// No cleanup can start now; nothing was removed.
    #[error("cleaning is blocked: {reason:?}")]
    CleaningBlocked { reason: BlockedReason },
    #[error("there is no cleanup {id} in the journal")]
    UnknownCleanup { id: CleanupId },
    /// An item index the cleanup does not have.
    #[error("the cleanup has no such item")]
    UnknownCleanupItem,
    /// An item that is no longer in the Trash.
    #[error("the item is not in the Trash")]
    NotInTrash,
    /// A finding without a path of its own (a tool's object): it cannot be ignored yet.
    #[error("the finding cannot be ignored")]
    CannotIgnore,
    /// A volume index past the storage map's list.
    #[error("the storage map has no such volume")]
    UnknownVolume,
    /// A group no registered module belongs to.
    #[error("no module of group `{id}` is registered")]
    UnknownGroup { id: String },
}

impl CommandError {
    /// Every code, in declaration order: each has a text `core.error.<code>` in the core's dictionary.
    pub const CODES: [&str; 26] = [
        "busy",
        "not-running",
        "unknown-finding",
        "unknown-module",
        "unknown-setting",
        "invalid-setting-value",
        "unknown-locale",
        "settings-read-only",
        "storage",
        "no-scan",
        "stale-node",
        "missing-capability",
        "no-settings-pane",
        "internal",
        "unknown-category",
        "nothing-to-clean",
        "too-many-items",
        "plan-gone",
        "plan-mismatch",
        "cleaning-blocked",
        "unknown-cleanup",
        "unknown-cleanup-item",
        "not-in-trash",
        "cannot-ignore",
        "unknown-volume",
        "unknown-group",
    ];

    /// The `code` the error is serialized with.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Busy { .. } => "busy",
            Self::NotRunning => "not-running",
            Self::UnknownFinding { .. } => "unknown-finding",
            Self::UnknownModule { .. } => "unknown-module",
            Self::UnknownSetting { .. } => "unknown-setting",
            Self::InvalidSettingValue { .. } => "invalid-setting-value",
            Self::UnknownLocale { .. } => "unknown-locale",
            Self::SettingsReadOnly => "settings-read-only",
            Self::Storage => "storage",
            Self::NoScan => "no-scan",
            Self::StaleNode => "stale-node",
            Self::MissingCapability { .. } => "missing-capability",
            Self::NoSettingsPane { .. } => "no-settings-pane",
            Self::Internal => "internal",
            Self::UnknownCategory { .. } => "unknown-category",
            Self::NothingToClean => "nothing-to-clean",
            Self::TooManyItems { .. } => "too-many-items",
            Self::PlanGone => "plan-gone",
            Self::PlanMismatch => "plan-mismatch",
            Self::CleaningBlocked { .. } => "cleaning-blocked",
            Self::UnknownCleanup { .. } => "unknown-cleanup",
            Self::UnknownCleanupItem => "unknown-cleanup-item",
            Self::NotInTrash => "not-in-trash",
            Self::CannotIgnore => "cannot-ignore",
            Self::UnknownVolume => "unknown-volume",
            Self::UnknownGroup { .. } => "unknown-group",
        }
    }

    /// The text for the user: key `core.error.<code>`, with the variant's fields as arguments.
    pub fn text(&self) -> Text {
        let text = Text::new(TextKey::from_valid(format!("core.error.{}", self.code())));
        match self {
            Self::Busy { running } => text.with("running", TextArg::Text(running.as_str().into())),
            Self::UnknownFinding { id } => text.with("id", TextArg::Count(id.get().into())),
            Self::UnknownModule { id } | Self::UnknownCategory { id } | Self::UnknownGroup { id } => {
                text.with("id", TextArg::Text(id.clone()))
            }
            Self::UnknownSetting { key } => text.with("key", TextArg::Text(key.clone())),
            Self::InvalidSettingValue { key } => text.with("key", TextArg::Text(key.as_str().into())),
            Self::UnknownLocale { locale } => text.with("locale", TextArg::Text(locale.clone())),
            Self::TooManyItems { limit } => text.with("limit", TextArg::Count((*limit).into())),
            Self::UnknownCleanup { id } => text.with("id", TextArg::Count(id.get().into())),
            Self::NotRunning
            | Self::SettingsReadOnly
            | Self::Storage
            | Self::NoScan
            | Self::StaleNode
            | Self::MissingCapability { .. }
            | Self::NoSettingsPane { .. }
            | Self::Internal
            | Self::NothingToClean
            | Self::PlanGone
            | Self::PlanMismatch
            | Self::CleaningBlocked { .. }
            | Self::UnknownCleanupItem
            | Self::NotInTrash
            | Self::CannotIgnore
            | Self::UnknownVolume => text,
        }
    }
}

/// A refused command as the UI receives it: the error, and the text to show for it, built by `CommandError::text()`.
/// The UI shows `text` as it is and never rebuilds it from the error.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Refusal {
    pub error: CommandError,
    pub text: Text,
}

impl From<CommandError> for Refusal {
    fn from(error: CommandError) -> Self {
        let text = error.text();
        Self { error, text }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::{BlockedReason, Capability, CleanupId, CommandError, FindingId, Operation, SettingKey};

    /// One error of every variant. The `match` has no wildcard: a new variant fails to compile until it is added here,
    /// and then the tests below check its code.
    pub(crate) fn one_of_each() -> Vec<CommandError> {
        let sample = |error: CommandError| match error {
            CommandError::Busy { .. }
            | CommandError::NotRunning
            | CommandError::UnknownFinding { .. }
            | CommandError::UnknownModule { .. }
            | CommandError::UnknownSetting { .. }
            | CommandError::InvalidSettingValue { .. }
            | CommandError::UnknownLocale { .. }
            | CommandError::SettingsReadOnly
            | CommandError::Storage
            | CommandError::NoScan
            | CommandError::StaleNode
            | CommandError::MissingCapability { .. }
            | CommandError::NoSettingsPane { .. }
            | CommandError::Internal
            | CommandError::UnknownCategory { .. }
            | CommandError::NothingToClean
            | CommandError::TooManyItems { .. }
            | CommandError::PlanGone
            | CommandError::PlanMismatch
            | CommandError::CleaningBlocked { .. }
            | CommandError::UnknownCleanup { .. }
            | CommandError::UnknownCleanupItem
            | CommandError::NotInTrash
            | CommandError::CannotIgnore
            | CommandError::UnknownVolume
            | CommandError::UnknownGroup { .. } => error,
        };
        vec![
            sample(CommandError::Busy { running: Operation::Scan }),
            sample(CommandError::NotRunning),
            sample(CommandError::UnknownFinding { id: FindingId::new(9) }),
            sample(CommandError::UnknownModule { id: "gone".into() }),
            sample(CommandError::UnknownSetting { key: "demo.gone".into() }),
            sample(CommandError::InvalidSettingValue { key: SettingKey::new("demo.deep").unwrap() }),
            sample(CommandError::UnknownLocale { locale: "fr".into() }),
            sample(CommandError::SettingsReadOnly),
            sample(CommandError::Storage),
            sample(CommandError::NoScan),
            sample(CommandError::StaleNode),
            sample(CommandError::MissingCapability { capability: Capability::AllFiles }),
            sample(CommandError::NoSettingsPane { capability: Capability::OtherAppsData }),
            sample(CommandError::Internal),
            sample(CommandError::UnknownCategory { id: "demo.gone".into() }),
            sample(CommandError::NothingToClean),
            sample(CommandError::TooManyItems { limit: 10_000 }),
            sample(CommandError::PlanGone),
            sample(CommandError::PlanMismatch),
            sample(CommandError::CleaningBlocked { reason: BlockedReason::Journal }),
            sample(CommandError::UnknownCleanup { id: CleanupId::new(4) }),
            sample(CommandError::UnknownCleanupItem),
            sample(CommandError::NotInTrash),
            sample(CommandError::CannotIgnore),
            sample(CommandError::UnknownVolume),
            sample(CommandError::UnknownGroup { id: "debug".into() }),
        ]
    }

    mod negative {
        use super::one_of_each;
        use crate::{CommandError, Refusal};

        #[test]
        fn every_error_code_is_in_the_code_list() {
            let codes: Vec<_> = one_of_each().iter().map(CommandError::code).collect();

            assert_eq!(codes, CommandError::CODES);
        }

        #[test]
        fn a_refusal_never_carries_a_text_other_than_its_errors() {
            for error in one_of_each() {
                let refusal = Refusal::from(error.clone());

                assert_eq!(refusal.text, error.text());
                assert_eq!(refusal.error, error);
            }
        }

        #[test]
        fn a_refusal_never_serializes_the_errors_display() {
            for error in one_of_each() {
                let json = serde_json::to_string(&Refusal::from(error.clone())).unwrap();

                assert!(!json.contains(&error.to_string()), "{json}");
            }
        }
    }

    mod positive {
        use serde_json::json;

        use super::one_of_each;
        use crate::{CommandError, Operation, Refusal, TextArg};

        #[test]
        fn command_errors_serialize_with_a_code_tag() {
            for error in one_of_each() {
                let json = serde_json::to_value(&error).unwrap();
                assert_eq!(json["code"], error.code(), "{json}");
                assert_eq!(error.text().key().as_str(), format!("core.error.{}", error.code()));
                assert_ne!(error.to_string(), "");
            }
            assert_eq!(
                serde_json::to_value(CommandError::Busy { running: Operation::Scan }).unwrap(),
                json!({ "code": "busy", "running": "scan" })
            );
        }

        #[test]
        fn a_refusal_serializes_the_error_and_its_text() {
            let refusal = Refusal::from(CommandError::Busy { running: Operation::Scan });

            assert_eq!(
                serde_json::to_value(refusal).unwrap(),
                json!({
                    "error": { "code": "busy", "running": "scan" },
                    "text": { "key": "core.error.busy", "args": { "running": { "kind": "text", "value": "scan" } } },
                })
            );
        }

        #[test]
        fn busy_names_the_running_operation_in_its_text_args() {
            let text = CommandError::Busy { running: Operation::Clean }.text();

            assert_eq!(text.key().as_str(), "core.error.busy");
            assert_eq!(text.args().get("running"), Some(&TextArg::Text("clean".into())));
        }

        #[test]
        fn every_field_becomes_a_text_arg() {
            let arg_names = |error: &CommandError| error.text().args().keys().cloned().collect::<Vec<_>>();
            let errors = one_of_each();

            let names: Vec<_> = errors.iter().map(arg_names).collect();

            assert_eq!(
                names,
                [
                    vec!["running"],
                    vec![],
                    vec!["id"],
                    vec!["id"],
                    vec!["key"],
                    vec!["key"],
                    vec!["locale"],
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                    vec!["id"],
                    vec![],
                    vec!["limit"],
                    vec![],
                    vec![],
                    vec![],
                    vec!["id"],
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                    vec!["id"],
                ]
            );
        }
    }
}
