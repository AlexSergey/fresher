//! The ids the module never claims, as data: `exclude.toml`, parsed and checked when the module is built.

use fresher_core::{AppIdError, BundleId};
use serde::Deserialize;

/// The table shipped with the module.
const SHIPPED: &str = include_str!("../exclude.toml");

/// The prefix that must stay in the table, so it cannot be dropped by mistake.
const APPLE: &str = "com.apple.";

/// Why an exclusion table was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum ExcludeError {
    #[error(
        "the exclusion table is not valid TOML, or an entry lacks its `why` or names not one of `id` and `prefix`: {0}"
    )]
    Format(String),
    #[error("the exclusion table lists nothing")]
    Empty,
    #[error("`{prefix}` is not a bundle id's start ending with a dot")]
    BadPrefix { prefix: String },
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
    /// In lower case, each ending with a dot.
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
            match (raw.prefix, raw.id) {
                (Some(prefix), None) => {
                    let valid = prefix.strip_suffix('.').is_some_and(|head| BundleId::new(head).is_ok());
                    if !valid {
                        return Err(ExcludeError::BadPrefix { prefix });
                    }
                    let folded = prefix.to_ascii_lowercase();
                    if prefixes.contains(&folded) {
                        return Err(ExcludeError::Duplicate { entry: prefix });
                    }
                    prefixes.push(folded);
                }
                (None, Some(id)) => {
                    let bundle = BundleId::new(id.as_str())?;
                    if ids.contains(&bundle) {
                        return Err(ExcludeError::Duplicate { entry: id });
                    }
                    ids.push(bundle);
                }
                _ => return Err(ExcludeError::Format("an entry names exactly one of `id` and `prefix`".into())),
            }
        }
        if !prefixes.iter().any(|prefix| prefix == APPLE) {
            return Err(ExcludeError::AppleMissing);
        }
        Ok(Self { prefixes, ids })
    }

    /// Whether the module keeps `id`: it starts with a prefix of the table (`com.apple.Safari`), or an id of the table
    /// names it (the same, or an id under it), case ignored. Not the other way round: `com.example` is not kept for
    /// the id `com.example.App` of the table.
    pub(crate) fn keeps(&self, id: &BundleId) -> bool {
        let folded = id.folded();
        self.prefixes.iter().any(|prefix| folded.starts_with(prefix.as_str()))
            || self.ids.iter().any(|excluded| excluded.names(id.as_str()))
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTable {
    #[serde(default)]
    exclude: Vec<RawEntry>,
}

/// An entry names one of `prefix` and `id`, and says why: `why` is required, and read by people only.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    prefix: Option<String>,
    id: Option<String>,
    #[expect(dead_code, reason = "required in the table, read by people only")]
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
        fn an_entry_with_an_unknown_field_both_or_neither_of_id_and_prefix_or_no_why_is_refused() {
            for entry in [
                "id = \"com.x.Y\"\nwhy = \"w\"\nnote = 1",
                "id = \"com.x.Y\"\nprefix = \"com.x.\"\nwhy = \"w\"",
                "why = \"w\"",
                "id = \"com.x.Y\"",
            ] {
                let text = format!("{APPLE}\n[[exclude]]\n{entry}\n");
                assert!(matches!(refused(&text), ExcludeError::Format(_)), "{entry}");
            }
        }

        #[test]
        fn a_table_with_no_entry_is_refused() {
            assert_eq!(refused(""), ExcludeError::Empty);
        }

        #[test]
        fn a_prefix_that_does_not_end_with_a_dot_or_is_not_an_id_start_is_refused() {
            for prefix in ["com.example", ".com.example.", "com..example.", "com example.", "", "."] {
                let text = format!("{APPLE}\n[[exclude]]\nprefix = \"{prefix}\"\nwhy = \"w\"\n");
                assert_eq!(refused(&text), ExcludeError::BadPrefix { prefix: prefix.into() }, "{prefix}");
            }
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
        fn a_table_without_the_apple_prefix_is_refused() {
            let text = "[[exclude]]\nprefix = \"com.example.\"\nwhy = \"w\"\n";
            assert_eq!(refused(text), ExcludeError::AppleMissing);
            let by_id = "[[exclude]]\nid = \"com.apple.Safari\"\nwhy = \"w\"\n";
            assert_eq!(refused(by_id), ExcludeError::AppleMissing);
        }

        #[test]
        fn the_shipped_table_does_not_keep_an_app_or_a_sibling_id_of_fresher() {
            let exclude = Exclude::shipped().unwrap();

            for other in
                ["com.example.App", "io.github.alexsergey.fresherx", "io.github.alexsergey", "com.applesauce.App"]
            {
                assert!(!exclude.keeps(&BundleId::new(other).unwrap()), "{other}");
            }
        }
    }

    mod positive {
        use fresher_core::BundleId;

        use crate::exclude::Exclude;

        fn id(id: &str) -> BundleId {
            BundleId::new(id).unwrap()
        }

        #[test]
        fn the_shipped_table_keeps_apples_ids_whatever_their_case() {
            let exclude = Exclude::shipped().unwrap();

            for apple in ["com.apple.Safari", "COM.APPLE.x", "com.apple.dt.Xcode"] {
                assert!(exclude.keeps(&id(apple)), "{apple}");
            }
        }

        #[test]
        fn the_shipped_table_keeps_fresher_and_every_id_under_it() {
            let exclude = Exclude::shipped().unwrap();

            for own in
                ["io.github.alexsergey.fresher", "io.github.alexsergey.fresher.dev", "IO.GITHUB.ALEXSERGEY.Fresher"]
            {
                assert!(exclude.keeps(&id(own)), "{own}");
            }
        }

        #[test]
        fn a_table_with_ids_and_prefixes_is_read() {
            let text =
                "[[exclude]]\nprefix = \"com.apple.\"\nwhy = \"w\"\n[[exclude]]\nid = \"com.x.Y\"\nwhy = \"w\"\n";

            let exclude = Exclude::parse(text).unwrap();

            assert_eq!((exclude.prefixes.len(), exclude.ids.len()), (1, 1));
            assert!(exclude.keeps(&id("com.x.y.Helper")));
        }
    }
}
