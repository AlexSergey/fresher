//! User-facing text as a key into the dictionary and its arguments; the core never produces English prose.

use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use serde::Serialize;
use ts_rs::TS;

use crate::ids::is_kebab_case;
use crate::{ByteSize, Timestamp};

/// A key of the dictionary: two or more dot-separated kebab-case segments, the first one the namespace
/// (`core.error.busy`, `demo.caches.name`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, TS)]
pub struct TextKey(String);

/// A text key that is not two or more dot-separated kebab-case segments.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[error("`{key}` is not a text key: two or more dot-separated kebab-case segments")]
pub struct TextKeyError {
    pub key: String,
}

impl TextKey {
    pub fn new(key: impl Into<String>) -> Result<Self, TextKeyError> {
        let key = key.into();
        let valid = key.contains('.') && key.split('.').all(is_kebab_case);
        if valid { Ok(Self(key)) } else { Err(TextKeyError { key }) }
    }

    /// A key known to be valid: built from validated parts by the core itself.
    pub(crate) fn from_valid(key: String) -> Self {
        debug_assert!(Self::new(key.as_str()).is_ok(), "{key} is not a valid text key");
        Self(key)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The first segment: `core` or a module id.
    pub fn namespace(&self) -> &str {
        self.0.split('.').next().unwrap_or_default()
    }
}

impl Borrow<str> for TextKey {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TextKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A text to show: its key and the values of its `{name}` placeholders, formatted by the UI.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct Text {
    key: TextKey,
    args: BTreeMap<String, TextArg>,
}

impl Text {
    pub const fn new(key: TextKey) -> Self {
        Self { key, args: BTreeMap::new() }
    }

    /// The text with the placeholder `name` set to `arg`.
    #[must_use]
    pub fn with(mut self, name: impl Into<String>, arg: TextArg) -> Self {
        self.args.insert(name.into(), arg);
        self
    }

    pub const fn key(&self) -> &TextKey {
        &self.key
    }

    pub const fn args(&self) -> &BTreeMap<String, TextArg> {
        &self.args
    }
}

/// The value of a placeholder; the UI formats counts, sizes and dates for the user's locale.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum TextArg {
    Text(String),
    Count(#[ts(type = "number")] u64),
    Size(ByteSize),
    Date(Timestamp),
    Path(PathBuf),
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::TextKey;

        #[test]
        fn rejects_an_empty_text_key_segment() {
            for key in ["", "core", "core.", ".core", "core..name", "core.error.", "core.Error", "core.err_or"] {
                assert!(TextKey::new(key).is_err(), "{key:?}");
            }
        }
    }

    mod positive {
        use crate::{ByteSize, Text, TextArg, TextKey};

        #[test]
        fn accepts_dot_separated_kebab_case_keys() {
            let key = TextKey::new("node-modules.stale.if-removed").unwrap();

            assert_eq!(key.namespace(), "node-modules");
            assert_eq!(key.as_str(), "node-modules.stale.if-removed");
            assert_eq!(key.to_string(), "node-modules.stale.if-removed");
        }

        #[test]
        fn serializes_text_args_in_key_order() {
            let text = Text::new(TextKey::new("demo.found").unwrap())
                .with("size", TextArg::Size(ByteSize::new(5)))
                .with("count", TextArg::Count(2))
                .with("app", TextArg::Text("Editor".into()));

            let json = serde_json::to_string(&text).unwrap();

            assert_eq!(text.key().as_str(), "demo.found");
            assert_eq!(text.args().len(), 3);
            assert_eq!(
                json,
                concat!(
                    r#"{"key":"demo.found","args":{"#,
                    r#""app":{"kind":"text","value":"Editor"},"#,
                    r#""count":{"kind":"count","value":2},"#,
                    r#""size":{"kind":"size","value":5}}}"#,
                )
            );
        }
    }
}
