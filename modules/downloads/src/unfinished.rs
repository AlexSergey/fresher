//! The unfinished-download suffixes, as data: `unfinished.toml`, parsed and checked when the module is built.

use fresher_core::{AppIdError, BundleId};
use serde::Deserialize;

/// The table shipped with the module.
const SHIPPED: &str = include_str!("../unfinished.toml");

/// Why a suffix table was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum SuffixesError {
    #[error("the suffix table is not valid TOML: {0}")]
    Format(String),
    #[error("the suffix table lists nothing")]
    Empty,
    #[error("`{suffix}` is not a dot and a plain name")]
    BadSuffix { suffix: String },
    #[error("`{suffix}` is listed twice")]
    DuplicateSuffix { suffix: String },
    #[error("`{suffix}` names no app")]
    NoApp { suffix: String },
    #[error("`{suffix}` names an app that is not a bundle id: {source}")]
    BadApp {
        suffix: String,
        #[source]
        source: AppIdError,
    },
    #[error("`{suffix}` is a bundle and cannot have a placeholder")]
    PlaceholderOnBundle { suffix: String },
    #[error("the table needs exactly one bundle row, it has {count}")]
    BundleRows { count: usize },
}

/// What a suffix's entry is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Form {
    File,
    Bundle,
}

/// One row: a suffix, what it is, the apps that write it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    /// Lowercased.
    pub(crate) suffix: String,
    pub(crate) kind: Form,
    /// Checked running.
    pub(crate) apps: Vec<BundleId>,
    /// The browser keeps an empty file of the final name beside it.
    pub(crate) placeholder: bool,
}

/// The unfinished-download suffixes, checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Suffixes {
    rows: Vec<Row>,
}

impl Suffixes {
    pub(crate) fn shipped() -> Result<Self, SuffixesError> {
        Self::parse(SHIPPED)
    }

    /// A table read from `text`: at least one row; each suffix a dot and a plain name (ASCII letters and digits), none
    /// twice (case ignored); each row with at least one app, each a bundle id; no placeholder on a bundle; exactly one
    /// bundle row.
    pub(crate) fn parse(text: &str) -> Result<Self, SuffixesError> {
        let table: RawTable = toml::from_str(text).map_err(|error| SuffixesError::Format(error.to_string()))?;
        if table.suffix.is_empty() {
            return Err(SuffixesError::Empty);
        }
        let mut rows: Vec<Row> = Vec::with_capacity(table.suffix.len());
        for raw in table.suffix {
            if !is_suffix(&raw.suffix) {
                return Err(SuffixesError::BadSuffix { suffix: raw.suffix });
            }
            let suffix = raw.suffix.to_ascii_lowercase();
            if rows.iter().any(|row| row.suffix == suffix) {
                return Err(SuffixesError::DuplicateSuffix { suffix: raw.suffix });
            }
            if raw.apps.is_empty() {
                return Err(SuffixesError::NoApp { suffix: raw.suffix });
            }
            let apps = raw
                .apps
                .iter()
                .map(|app| BundleId::new(app.as_str()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|source| SuffixesError::BadApp { suffix: raw.suffix.clone(), source })?;
            if raw.kind == Form::Bundle && raw.placeholder {
                return Err(SuffixesError::PlaceholderOnBundle { suffix: raw.suffix });
            }
            rows.push(Row { suffix, kind: raw.kind, apps, placeholder: raw.placeholder });
        }
        let count = rows.iter().filter(|row| row.kind == Form::Bundle).count();
        if count != 1 {
            return Err(SuffixesError::BundleRows { count });
        }
        Ok(Self { rows })
    }

    /// The rows, in the table's order.
    pub(crate) fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// The row of `kind` whose suffix ends `name` (ASCII case ignored), `name` being longer than it.
    pub(crate) fn row_of(&self, name: &str, kind: Form) -> Option<&Row> {
        self.rows().iter().find(|row| row.kind == kind && ends_with_longer(name, &row.suffix))
    }

    /// The suffix of the one bundle row (`.download`), lowercased.
    pub(crate) fn bundle_suffix(&self) -> Option<&str> {
        self.rows.iter().find(|row| row.kind == Form::Bundle).map(|row| row.suffix.as_str())
    }
}

/// Whether `name` ends with `suffix`, ASCII case ignored, and is longer than it.
pub(crate) fn ends_with_longer(name: &str, suffix: &str) -> bool {
    let (name, suffix) = (name.as_bytes(), suffix.as_bytes());
    name.len() > suffix.len() && name[name.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
}

/// A dot and a plain name: ASCII letters and digits, at least one.
fn is_suffix(text: &str) -> bool {
    text.strip_prefix('.').is_some_and(|name| !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_alphanumeric()))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTable {
    #[serde(default)]
    suffix: Vec<RawRow>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRow {
    suffix: String,
    kind: Form,
    #[serde(default)]
    apps: Vec<String>,
    #[serde(default)]
    placeholder: bool,
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fmt::Write as _;

        use crate::unfinished::{Form, Suffixes, SuffixesError};

        const CHROME: &str = r#"apps = ["com.google.Chrome"]"#;

        fn table(rows: &[(&str, &str, &str)]) -> String {
            let mut text = String::new();
            for (suffix, kind, rest) in rows {
                writeln!(
                    text,
                    "[[suffix]]
suffix = \"{suffix}\"
kind = \"{kind}\"
{rest}
"
                )
                .expect("a string takes text");
            }
            text
        }

        fn refused(text: &str) -> SuffixesError {
            Suffixes::parse(text).expect_err("the table is refused")
        }

        #[test]
        fn text_that_is_not_toml_is_refused() {
            assert!(matches!(refused("[[suffix"), SuffixesError::Format(_)));
        }

        #[test]
        fn an_unknown_field_is_refused() {
            let text = table(&[(".part", "file", "apps = [\"a.b\"]\nwhy = \"x\"")]);
            assert!(matches!(refused(&text), SuffixesError::Format(_)));
        }

        #[test]
        fn a_table_with_no_row_is_refused() {
            assert_eq!(refused(""), SuffixesError::Empty);
        }

        #[test]
        fn a_suffix_that_is_not_a_dot_and_a_plain_name_is_refused() {
            for suffix in ["part", ".", ".a.b", ".a b", ".a/b"] {
                let text = table(&[(suffix, "file", CHROME), (".download", "bundle", CHROME)]);
                assert_eq!(refused(&text), SuffixesError::BadSuffix { suffix: suffix.into() }, "{suffix:?}");
            }
        }

        #[test]
        fn a_suffix_listed_twice_in_another_case_is_refused() {
            let text = table(&[(".part", "file", CHROME), (".PART", "file", CHROME), (".download", "bundle", CHROME)]);
            assert_eq!(refused(&text), SuffixesError::DuplicateSuffix { suffix: ".PART".into() });
        }

        #[test]
        fn a_row_with_no_app_is_refused() {
            let text = table(&[(".part", "file", "")]);
            assert_eq!(refused(&text), SuffixesError::NoApp { suffix: ".part".into() });
        }

        #[test]
        fn an_app_that_is_not_a_bundle_id_is_refused() {
            let text = table(&[(".part", "file", "apps = [\"not an id\"]")]);
            assert!(matches!(refused(&text), SuffixesError::BadApp { .. }));
        }

        #[test]
        fn a_placeholder_on_a_bundle_is_refused() {
            let text = table(&[(".download", "bundle", "apps = [\"a.b\"]\nplaceholder = true")]);
            assert_eq!(refused(&text), SuffixesError::PlaceholderOnBundle { suffix: ".download".into() });
        }

        #[test]
        fn a_table_with_no_bundle_row_is_refused() {
            let text = table(&[(".part", "file", CHROME)]);
            assert_eq!(refused(&text), SuffixesError::BundleRows { count: 0 });
        }

        #[test]
        fn a_name_that_is_only_a_suffix_or_of_the_other_kind_has_no_row() {
            let table = Suffixes::shipped().expect("the shipped table is valid");

            assert_eq!(table.row_of(".part", Form::File), None);
            assert_eq!(table.row_of("movie.part.mp4", Form::File), None);
            assert_eq!(table.row_of("trip.download", Form::File), None);
            assert_eq!(table.row_of("movie.part", Form::Bundle), None);
        }

        #[test]
        fn a_table_with_two_bundle_rows_is_refused() {
            let text = table(&[(".download", "bundle", CHROME), (".bundle", "bundle", CHROME)]);
            assert_eq!(refused(&text), SuffixesError::BundleRows { count: 2 });
        }
    }

    mod positive {
        use crate::unfinished::{Form, Suffixes};

        #[test]
        fn the_shipped_table_has_five_rows_and_one_bundle() {
            let table = Suffixes::shipped().expect("the shipped table is valid");

            let suffixes: Vec<&str> = table.rows().iter().map(|row| row.suffix.as_str()).collect();
            assert_eq!(suffixes, [".crdownload", ".part", ".opdownload", ".partial", ".download"]);
            let placeholders: Vec<&str> =
                table.rows().iter().filter(|row| row.placeholder).map(|row| row.suffix.as_str()).collect();
            assert_eq!(placeholders, [".part"]);
            assert_eq!(table.bundle_suffix(), Some(".download"));
            assert_eq!(table.rows().iter().filter(|row| row.kind == Form::Bundle).count(), 1);
        }

        #[test]
        fn a_row_is_found_by_its_suffix_in_any_case() {
            let table = Suffixes::shipped().expect("the shipped table is valid");

            let suffix = |name: &str, kind: Form| table.row_of(name, kind).map(|row| row.suffix.as_str());
            assert_eq!(suffix("movie.mp4.CRDOWNLOAD", Form::File), Some(".crdownload"));
            assert_eq!(suffix("a.part", Form::File), Some(".part"));
            assert_eq!(suffix("trip.zip.Download", Form::Bundle), Some(".download"));
        }

        #[test]
        fn a_suffix_is_kept_lowercased() {
            let text = "[[suffix]]\nsuffix = \".Download\"\nkind = \"bundle\"\napps = [\"a.b\"]\n";

            let table = Suffixes::parse(text).expect("the table is valid");

            assert_eq!(table.bundle_suffix(), Some(".download"));
        }
    }
}
