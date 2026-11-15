//! The one error shape every command returns to the UI.

use serde::Serialize;
use ts_rs::TS;

use crate::{FindingId, Operation, SettingKey, Text, TextArg, TextKey};

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
    /// A bug: the operation stopped safely.
    #[error("an internal error stopped the operation")]
    Internal,
}

impl CommandError {
    /// Every code, in declaration order: each has a text `core.error.<code>` in the core's dictionary.
    pub const CODES: [&str; 10] = [
        "busy",
        "not-running",
        "unknown-finding",
        "unknown-module",
        "unknown-setting",
        "invalid-setting-value",
        "unknown-locale",
        "settings-read-only",
        "storage",
        "internal",
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
            Self::Internal => "internal",
        }
    }

    /// The text for the user: key `core.error.<code>`, with the variant's fields as arguments.
    pub fn text(&self) -> Text {
        let text = Text::new(TextKey::from_valid(format!("core.error.{}", self.code())));
        match self {
            Self::Busy { running } => text.with("running", TextArg::Text(running.as_str().into())),
            Self::UnknownFinding { id } => text.with("id", TextArg::Count(id.get().into())),
            Self::UnknownModule { id } => text.with("id", TextArg::Text(id.clone())),
            Self::UnknownSetting { key } => text.with("key", TextArg::Text(key.clone())),
            Self::InvalidSettingValue { key } => text.with("key", TextArg::Text(key.as_str().into())),
            Self::UnknownLocale { locale } => text.with("locale", TextArg::Text(locale.clone())),
            Self::NotRunning | Self::SettingsReadOnly | Self::Storage | Self::Internal => text,
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
mod tests {
    use crate::{CommandError, FindingId, Operation, SettingKey};

    /// One error of every variant. The `match` has no wildcard: a new variant fails to compile until it is added here,
    /// and then the tests below check its code.
    fn one_of_each() -> Vec<CommandError> {
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
            | CommandError::Internal => error,
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
            sample(CommandError::Internal),
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
                ]
            );
        }
    }
}
