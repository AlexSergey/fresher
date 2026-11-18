//! The ids the module never claims, as data: `exclude.toml`, parsed and checked when the module is built.

use fresher_core::{AppIdError, BundleId};
use serde::Deserialize;

use crate::name::related;

/// The table shipped with the module.
const SHIPPED: &str = include_str!("../exclude.toml");

/// The prefix that must stay in the table, so it cannot be dropped by mistake.
const APPLE: &str = "com.apple.";

/// Why an exclusion table was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum ExcludeError {
    #[error("the exclusion table is not valid TOML: {0}")]
    Format(String),
    #[error("the exclusion table lists nothing")]
    Empty,
    #[error("`{prefix}` is not a bundle id's start ending with a dot")]
    BadPrefix { prefix: String },
    #[error("an entry names both an id and a prefix, or neither")]
    NotOneOf,
    #[error("the entry `{entry}` has no reason")]
    NoWhy { entry: String },
    #[error(transparent)]
    BadId(#[from] AppIdError),
    #[error("`{entry}` is listed twice")]
    Duplicate { entry: String },
    #[error("the table does not keep `{APPLE}`")]
    AppleMissing,
}

/// The ids and id prefixes the module never claims, checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Exclude {
    prefixes: Vec<String>,
    ids: Vec<BundleId>,
}

impl Exclude {
    pub(crate) fn shipped() -> Result<Self, ExcludeError> {
        Self::parse(SHIPPED)
    }

    /// A table read from `text`: at least one entry, each with a `why` and exactly one of `prefix` (a bundle id's
    /// start ending with `.`) and `id` (a bundle id), none twice (case ignored), and `com.apple.` among them.
    pub(crate) fn parse(text: &str) -> Result<Self, ExcludeError> {
        let table: RawTable = toml::from_str(text).map_err(|error| ExcludeError::Format(error.to_string()))?;
        if table.exclude.is_empty() {
            return Err(ExcludeError::Empty);
        }
        let (mut prefixes, mut ids): (Vec<String>, Vec<BundleId>) = (Vec::new(), Vec::new());
        for raw in table.exclude {
            let (entry, is_prefix) = match (raw.prefix, raw.id) {
                (Some(prefix), None) => (prefix, true),
                (None, Some(id)) => (id, false),
                _ => return Err(ExcludeError::NotOneOf),
            };
            if raw.why.trim().is_empty() {
                return Err(ExcludeError::NoWhy { entry });
            }
            if is_prefix {
                let Some(head) = entry.strip_suffix('.') else { return Err(ExcludeError::BadPrefix { prefix: entry }) };
                if BundleId::new(head).is_err() {
                    return Err(ExcludeError::BadPrefix { prefix: entry });
                }
                let folded = entry.to_ascii_lowercase();
                if prefixes.contains(&folded) {
                    return Err(ExcludeError::Duplicate { entry });
                }
                prefixes.push(folded);
            } else {
                let id = BundleId::new(entry.as_str())?;
                if ids.contains(&id) {
                    return Err(ExcludeError::Duplicate { entry });
                }
                ids.push(id);
            }
        }
        if !prefixes.iter().any(|prefix| prefix == APPLE) {
            return Err(ExcludeError::AppleMissing);
        }
        Ok(Self { prefixes, ids })
    }

    /// Whether the module never claims `id`: it starts with a prefix of the table (`com.apple.Safari`), or is a prefix
    /// without its dot (`com.apple`), or is related to an id of the table (the same, its helpers, or an id it extends),
    /// as installed apps are; case ignored. An entry only ever keeps, so it is read wide.
    pub(crate) fn covers(&self, id: &BundleId) -> bool {
        let folded = id.folded();
        let by_prefix = self
            .prefixes
            .iter()
            .any(|prefix| folded.starts_with(prefix.as_str()) || prefix.strip_suffix('.') == Some(folded));
        by_prefix || self.ids.iter().any(|excluded| related(excluded, id))
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTable {
    #[serde(default)]
    exclude: Vec<RawEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    prefix: Option<String>,
    id: Option<String>,
    #[serde(default)]
    why: String,
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::BundleId;

        use crate::exclude::{Exclude, ExcludeError};

        const APPLE: &str = "[[exclude]]\nprefix = \"com.apple.\"\nwhy = \"Apple\"\n";

        fn refused(text: &str) -> ExcludeError {
            Exclude::parse(text).unwrap_err()
        }

        #[test]
        fn text_that_is_not_toml_is_refused() {
            assert!(matches!(refused("[[exclude"), ExcludeError::Format(_)));
        }

        #[test]
        fn an_unknown_field_is_refused() {
            assert!(matches!(
                refused(&format!("{APPLE}\n[[exclude]]\nid = \"com.x.Y\"\nwhy = \"w\"\nnote = 1\n")),
                ExcludeError::Format(_)
            ));
        }

        #[test]
        fn a_table_with_no_entry_is_refused() {
            assert_eq!(refused(""), ExcludeError::Empty);
        }

        #[test]
        fn a_prefix_that_does_not_end_with_a_dot_or_is_not_an_id_start_is_refused() {
            for prefix in ["com.example", ".com.example.", "com..example.", "com example.", ""] {
                let text = format!("{APPLE}\n[[exclude]]\nprefix = \"{prefix}\"\nwhy = \"w\"\n");
                assert_eq!(refused(&text), ExcludeError::BadPrefix { prefix: prefix.into() }, "{prefix}");
            }
        }

        #[test]
        fn an_entry_with_both_or_neither_of_id_and_prefix_is_refused() {
            for entry in ["id = \"com.x.Y\"\nprefix = \"com.x.\"\nwhy = \"w\"", "why = \"w\""] {
                let text = format!("{APPLE}\n[[exclude]]\n{entry}\n");
                assert_eq!(refused(&text), ExcludeError::NotOneOf, "{entry}");
            }
        }

        #[test]
        fn an_entry_without_a_reason_is_refused() {
            let text = format!("{APPLE}\n[[exclude]]\nid = \"com.x.Y\"\nwhy = \" \"\n");
            assert_eq!(refused(&text), ExcludeError::NoWhy { entry: "com.x.Y".into() });
            let text = format!("{APPLE}\n[[exclude]]\nid = \"com.x.Y\"\n");
            assert_eq!(refused(&text), ExcludeError::NoWhy { entry: "com.x.Y".into() });
        }

        #[test]
        fn an_id_that_is_not_a_bundle_id_is_refused() {
            let text = format!("{APPLE}\n[[exclude]]\nid = \"not an id\"\nwhy = \"w\"\n");
            assert!(matches!(refused(&text), ExcludeError::BadId(_)));
        }

        #[test]
        fn an_entry_listed_twice_is_refused_whatever_its_case() {
            let id = format!(
                "{APPLE}\n[[exclude]]\nid = \"com.x.Y\"\nwhy = \"w\"\n[[exclude]]\nid = \"COM.X.y\"\nwhy = \"w\"\n"
            );
            assert_eq!(refused(&id), ExcludeError::Duplicate { entry: "COM.X.y".into() });
            let prefix = format!("{APPLE}\n[[exclude]]\nprefix = \"COM.APPLE.\"\nwhy = \"w\"\n");
            assert_eq!(refused(&prefix), ExcludeError::Duplicate { entry: "COM.APPLE.".into() });
        }

        #[test]
        fn an_id_that_only_shares_a_start_with_an_entry_is_not_covered() {
            let text =
                "[[exclude]]\nprefix = \"com.apple.\"\nwhy = \"w\"\n[[exclude]]\nid = \"com.x.Y\"\nwhy = \"w\"\n";
            let exclude = Exclude::parse(text).unwrap();

            for other in ["com.applesauce.App", "com.example.App", "com.x.Yz", "org.x.Y"] {
                assert!(!exclude.covers(&BundleId::new(other).unwrap()), "{other}");
            }
        }

        #[test]
        fn a_table_without_the_apple_prefix_is_refused() {
            let text = "[[exclude]]\nprefix = \"com.example.\"\nwhy = \"w\"\n";
            assert_eq!(refused(text), ExcludeError::AppleMissing);
            let by_id = "[[exclude]]\nid = \"com.apple.Safari\"\nwhy = \"w\"\n";
            assert_eq!(refused(by_id), ExcludeError::AppleMissing);
        }
    }

    mod positive {
        use fresher_core::BundleId;

        use crate::exclude::Exclude;

        fn id(id: &str) -> BundleId {
            BundleId::new(id).unwrap()
        }

        #[test]
        fn the_shipped_table_covers_every_apple_id_whatever_its_case() {
            let exclude = Exclude::shipped().unwrap();

            for apple in ["com.apple.Safari", "COM.APPLE.dt.Xcode", "com.apple"] {
                assert!(exclude.covers(&id(apple)), "{apple}");
            }
        }

        #[test]
        fn an_excluded_id_covers_itself_and_every_related_id() {
            let text =
                "[[exclude]]\nprefix = \"com.apple.\"\nwhy = \"w\"\n[[exclude]]\nid = \"com.x.Y\"\nwhy = \"w\"\n";
            let exclude = Exclude::parse(text).unwrap();

            assert!(exclude.covers(&id("com.x.y")));
            assert!(exclude.covers(&id("com.x.Y.Helper")));
            assert!(exclude.covers(&id("com.x")));
        }

        #[test]
        fn the_shipped_table_is_valid() {
            let exclude = Exclude::shipped().unwrap();

            assert_eq!(exclude.prefixes, ["com.apple."]);
            assert_eq!(exclude.ids.len(), 0);
        }

        #[test]
        fn a_table_with_ids_and_prefixes_is_read() {
            let text =
                "[[exclude]]\nprefix = \"com.apple.\"\nwhy = \"w\"\n[[exclude]]\nid = \"com.x.Y\"\nwhy = \"w\"\n";

            let exclude = Exclude::parse(text).unwrap();

            assert_eq!((exclude.prefixes.len(), exclude.ids.len()), (1, 1));
        }
    }
}
