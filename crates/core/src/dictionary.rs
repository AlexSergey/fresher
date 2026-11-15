//! The texts of one language: a flat, sorted map from text key to text, merged from the core and every module.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

use crate::TextKey;

/// Texts by key. Placeholders are written `{name}`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, TS)]
pub struct Dictionary(BTreeMap<TextKey, String>);

/// Why a dictionary was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum DictionaryError {
    #[error("the dictionary is not valid JSON: {reason}")]
    Json { reason: String },
    /// The document is not an object (`key` is `None`), or the value of `key` is not a string.
    #[error("the dictionary is not a flat object of strings")]
    NotFlat { key: Option<String> },
    #[error("`{key}` is not a text key: two or more dot-separated kebab-case segments")]
    InvalidKey { key: String },
    #[error("`{key}` is outside the dictionary's namespace")]
    OutsideNamespace { key: String },
    #[error("`{key}` is defined twice")]
    Duplicate { key: String },
    #[error("the text of `{key}` is empty")]
    EmptyText { key: String },
}

impl Dictionary {
    /// Parses a flat JSON object whose keys are all in `namespace` (`core`, a module id) and whose values are
    /// non-empty strings. A key written twice is refused.
    pub fn parse(namespace: &str, json: &str) -> Result<Self, DictionaryError> {
        let json_error = |error: serde_json::Error| DictionaryError::Json { reason: error.to_string() };
        if !serde_json::from_str::<serde_json::Value>(json).map_err(json_error)?.is_object() {
            return Err(DictionaryError::NotFlat { key: None });
        }
        let Entries(entries) = serde_json::from_str(json).map_err(json_error)?;
        let mut texts = BTreeMap::new();
        for (key, value) in entries {
            let serde_json::Value::String(text) = value else {
                return Err(DictionaryError::NotFlat { key: Some(key) });
            };
            let text_key = TextKey::new(key.as_str()).map_err(|_| DictionaryError::InvalidKey { key: key.clone() })?;
            if text_key.namespace() != namespace {
                return Err(DictionaryError::OutsideNamespace { key });
            }
            if text.trim().is_empty() {
                return Err(DictionaryError::EmptyText { key });
            }
            if texts.insert(text_key, text).is_some() {
                return Err(DictionaryError::Duplicate { key });
            }
        }
        Ok(Self(texts))
    }

    /// Adds every text of `other`; refuses the whole merge when a key is already here.
    pub fn merge(&mut self, other: Self) -> Result<(), DictionaryError> {
        if let Some(key) = other.0.keys().find(|key| self.0.contains_key(*key)) {
            return Err(DictionaryError::Duplicate { key: key.as_str().into() });
        }
        self.0.extend(other.0);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    /// Every key, sorted.
    pub fn keys(&self) -> impl Iterator<Item = &TextKey> {
        self.0.keys()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The `{name}` placeholders of the text of `key`, sorted; `None` when there is no such text.
    pub fn placeholders(&self, key: &str) -> Option<BTreeSet<&str>> {
        let text = self.get(key)?;
        let names = text.split('{').skip(1).filter_map(|rest| rest.split_once('}').map(|(name, _)| name));
        Some(names.filter(|name| !name.is_empty()).collect())
    }
}

/// The entries of a JSON object in order, duplicates kept (a `Map` would keep only the last one).
struct Entries(Vec<(String, serde_json::Value)>);

impl<'de> Deserialize<'de> for Entries {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EntriesVisitor;

        impl<'de> Visitor<'de> for EntriesVisitor {
            type Value = Entries;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON object")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Entries, A::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry()? {
                    entries.push(entry);
                }
                Ok(Entries(entries))
            }
        }

        deserializer.deserialize_map(EntriesVisitor)
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{Dictionary, DictionaryError};

        #[test]
        fn rejects_a_key_outside_the_namespace() {
            let error = Dictionary::parse("demo", r#"{ "demo.a.name": "A", "core.error.busy": "Busy" }"#);

            assert_eq!(error, Err(DictionaryError::OutsideNamespace { key: "core.error.busy".into() }));
        }

        #[test]
        fn rejects_a_nested_object() {
            let nested = Dictionary::parse("demo", r#"{ "demo": { "a": "A" } }"#);
            let array = Dictionary::parse("demo", r#"["demo.a"]"#);

            assert_eq!(nested, Err(DictionaryError::NotFlat { key: Some("demo".into()) }));
            assert_eq!(array, Err(DictionaryError::NotFlat { key: None }));
        }

        #[test]
        fn rejects_an_empty_text() {
            let error = Dictionary::parse("demo", r#"{ "demo.a.name": " " }"#);

            assert_eq!(error, Err(DictionaryError::EmptyText { key: "demo.a.name".into() }));
        }

        #[test]
        fn rejects_an_invalid_key_and_broken_json() {
            let invalid = Dictionary::parse("demo", r#"{ "demo.A": "A" }"#);
            let broken = Dictionary::parse("demo", r#"{ "demo.a": "#);

            assert_eq!(invalid, Err(DictionaryError::InvalidKey { key: "demo.A".into() }));
            assert!(matches!(broken, Err(DictionaryError::Json { .. })));
        }

        #[test]
        fn rejects_a_key_written_twice_in_one_file() {
            let error = Dictionary::parse("demo", r#"{ "demo.a": "A", "demo.a": "B" }"#);

            assert_eq!(error, Err(DictionaryError::Duplicate { key: "demo.a".into() }));
        }

        #[test]
        fn rejects_a_duplicate_key_on_merge() {
            let mut texts = Dictionary::parse("demo", r#"{ "demo.a": "A", "demo.b": "B" }"#).unwrap();
            let again = Dictionary::parse("demo", r#"{ "demo.b": "B2", "demo.c": "C" }"#).unwrap();

            assert_eq!(texts.merge(again), Err(DictionaryError::Duplicate { key: "demo.b".into() }));
            assert_eq!(texts.get("demo.b"), Some("B"));
            assert_eq!(texts.get("demo.c"), None);
        }

        #[test]
        fn a_missing_text_has_no_placeholders() {
            assert_eq!(Dictionary::default().placeholders("demo.a"), None);
        }
    }

    mod positive {
        use std::collections::BTreeSet;

        use crate::{Dictionary, TextKey};

        #[test]
        fn parses_a_flat_dictionary() {
            let texts =
                Dictionary::parse("demo", r#"{ "demo.caches.name": "Caches", "demo.caches.what": "Copies" }"#).unwrap();

            assert_eq!(texts.len(), 2);
            assert_eq!(texts.get("demo.caches.name"), Some("Caches"));
            assert_eq!(texts.keys().map(TextKey::as_str).collect::<Vec<_>>(), ["demo.caches.name", "demo.caches.what"]);
            assert!(Dictionary::parse("demo", "{}").unwrap().is_empty());
        }

        #[test]
        fn merges_two_namespaces() {
            let mut texts = Dictionary::parse("core", r#"{ "core.error.busy": "Busy" }"#).unwrap();

            texts.merge(Dictionary::parse("demo", r#"{ "demo.a": "A" }"#).unwrap()).unwrap();

            assert_eq!(texts.keys().map(TextKey::as_str).collect::<Vec<_>>(), ["core.error.busy", "demo.a"]);
            assert_eq!(
                serde_json::to_value(&texts).unwrap(),
                serde_json::json!({ "core.error.busy": "Busy", "demo.a": "A" })
            );
        }

        #[test]
        fn lists_the_placeholders_of_a_text() {
            let texts =
                Dictionary::parse("demo", r#"{ "demo.found": "{count} items in {app}, {count} {} more {open" }"#)
                    .unwrap();

            assert_eq!(texts.placeholders("demo.found"), Some(BTreeSet::from(["app", "count"])));
        }
    }
}
