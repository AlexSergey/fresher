//! The artifact table, as data: `artifacts.toml`, parsed and checked against the core when the module is built.

use fresher_core::ProjectKind;
use serde::Deserialize;

use crate::categories::Category;

/// The table shipped with the module.
const SHIPPED: &str = include_str!("../artifacts.toml");

/// Why an artifact table was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum ArtifactsError {
    #[error("the artifact table is not valid TOML: {0}")]
    Format(String),
    #[error("the artifact table lists nothing")]
    Empty,
    #[error("`{kind}` is not a kind of project")]
    UnknownKind { kind: String },
    #[error("`{category}` is not a category a row may name")]
    UnknownCategory { category: String },
    #[error("`{folder}` is not an artifact of `{kind}` projects")]
    NotAnArtifact { kind: String, folder: String },
    #[error("`{kind}` `{folder}` is listed twice")]
    Duplicate { kind: String, folder: String },
    #[error("`{kind}` `{folder}` has no row")]
    Missing { kind: String, folder: String },
    #[error("the command of `{kind}` `{folder}` is not one line of printable ASCII")]
    BadCommand { kind: String, folder: String },
}

/// One row: an artifact of a kind, its category and the command that makes it again.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    pub(crate) kind: ProjectKind,
    pub(crate) folder: String,
    pub(crate) category: Category,
    /// What the reason names; the app never runs it.
    pub(crate) command: String,
}

/// The artifact table, checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Artifacts {
    rows: Vec<Row>,
}

/// The module's name of a kind; the core's enum has none.
const fn kind_name(kind: ProjectKind) -> &'static str {
    match kind {
        ProjectKind::Node => "node",
        ProjectKind::Rust => "rust",
        ProjectKind::Gradle => "gradle",
        ProjectKind::Maven => "maven",
        ProjectKind::Python => "python",
        ProjectKind::SwiftPackage => "swift-package",
        ProjectKind::Go => "go",
        ProjectKind::Dart => "dart",
    }
}

impl Artifacts {
    pub(crate) fn shipped() -> Result<Self, ArtifactsError> {
        Self::parse(SHIPPED)
    }

    /// A table read from `text`: at least one row; each kind and category known, each folder an artifact of its kind
    /// in the core, no pair twice, every artifact of the core with a row, each command one line of printable ASCII.
    pub(crate) fn parse(text: &str) -> Result<Self, ArtifactsError> {
        let table: RawTable = toml::from_str(text).map_err(|error| ArtifactsError::Format(error.to_string()))?;
        if table.artifact.is_empty() {
            return Err(ArtifactsError::Empty);
        }
        let mut rows: Vec<Row> = Vec::with_capacity(table.artifact.len());
        for raw in table.artifact {
            let kind = ProjectKind::ALL
                .into_iter()
                .find(|kind| kind_name(*kind) == raw.kind)
                .ok_or_else(|| ArtifactsError::UnknownKind { kind: raw.kind.clone() })?;
            let category = Category::of_table(&raw.category)
                .ok_or_else(|| ArtifactsError::UnknownCategory { category: raw.category.clone() })?;
            if !kind.artifacts().contains(&raw.folder.as_str()) {
                return Err(ArtifactsError::NotAnArtifact { kind: raw.kind, folder: raw.folder });
            }
            if rows.iter().any(|row| row.kind == kind && row.folder == raw.folder) {
                return Err(ArtifactsError::Duplicate { kind: raw.kind, folder: raw.folder });
            }
            if !is_command(&raw.command) {
                return Err(ArtifactsError::BadCommand { kind: raw.kind, folder: raw.folder });
            }
            rows.push(Row { kind, folder: raw.folder, category, command: raw.command });
        }
        for kind in ProjectKind::ALL {
            for folder in kind.artifacts() {
                if !rows.iter().any(|row| row.kind == kind && row.folder == *folder) {
                    return Err(ArtifactsError::Missing {
                        kind: kind_name(kind).to_owned(),
                        folder: (*folder).to_owned(),
                    });
                }
            }
        }
        Ok(Self { rows })
    }

    /// The row of `folder` among the artifacts of `kind`.
    pub(crate) fn row(&self, kind: ProjectKind, folder: &str) -> Option<&Row> {
        self.rows.iter().find(|row| row.kind == kind && row.folder == folder)
    }
}

/// One non-empty line of printable ASCII, with no space at either end.
fn is_command(command: &str) -> bool {
    !command.is_empty()
        && command.trim() == command
        && command.bytes().all(|byte| byte == b' ' || byte.is_ascii_graphic())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTable {
    #[serde(default)]
    artifact: Vec<RawRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRow {
    kind: String,
    folder: String,
    category: String,
    command: String,
}

#[cfg(test)]
mod tests {
    use super::{Artifacts, SHIPPED};

    /// The shipped table with `from` replaced by `to`, once.
    fn changed(from: &str, to: &str) -> String {
        assert!(SHIPPED.contains(from), "{from}");
        SHIPPED.replacen(from, to, 1)
    }

    mod negative {
        use super::{Artifacts, SHIPPED, changed};
        use crate::table::ArtifactsError;

        #[test]
        fn text_that_is_not_toml_is_refused() {
            assert!(matches!(Artifacts::parse("[[artifact"), Err(ArtifactsError::Format(_))));
        }

        #[test]
        fn an_unknown_field_is_refused() {
            let text = changed("command = \"npm install\"", "command = \"npm install\"\nowner = \"me\"");

            assert!(matches!(Artifacts::parse(&text), Err(ArtifactsError::Format(_))));
        }

        #[test]
        fn an_empty_text_is_refused() {
            assert_eq!(Artifacts::parse(""), Err(ArtifactsError::Empty));
        }

        #[test]
        fn an_unknown_kind_is_refused() {
            let text = changed("kind = \"node\"", "kind = \"cobol\"");

            assert_eq!(Artifacts::parse(&text), Err(ArtifactsError::UnknownKind { kind: "cobol".into() }));
        }

        #[test]
        fn the_rust_build_cache_is_not_a_category_a_row_may_name() {
            let text = changed("category = \"dependencies\"", "category = \"rust-build-cache\"");

            assert_eq!(
                Artifacts::parse(&text),
                Err(ArtifactsError::UnknownCategory { category: "rust-build-cache".into() })
            );
        }

        #[test]
        fn a_folder_the_core_does_not_list_for_the_kind_is_refused() {
            let text = changed("folder = \"node_modules\"", "folder = \"dist\"");

            assert_eq!(
                Artifacts::parse(&text),
                Err(ArtifactsError::NotAnArtifact { kind: "node".into(), folder: "dist".into() })
            );
        }

        #[test]
        fn a_pair_listed_twice_is_refused() {
            let text = format!(
                "{SHIPPED}\n[[artifact]]\nkind = \"node\"\nfolder = \"node_modules\"\ncategory = \"dependencies\"\ncommand = \"npm ci\"\n"
            );

            assert_eq!(
                Artifacts::parse(&text),
                Err(ArtifactsError::Duplicate { kind: "node".into(), folder: "node_modules".into() })
            );
        }

        #[test]
        fn an_artifact_of_the_core_without_a_row_is_refused() {
            let without = SHIPPED.replacen(
                "[[artifact]]\nkind = \"python\"\nfolder = \"venv\"\ncategory = \"python-environments\"\ncommand = \"python3 -m venv venv\"\n",
                "",
                1,
            );

            assert_eq!(
                Artifacts::parse(&without),
                Err(ArtifactsError::Missing { kind: "python".into(), folder: "venv".into() })
            );
        }

        #[test]
        fn an_empty_command_is_refused() {
            let text = changed("command = \"npm install\"", "command = \"\"");

            assert_eq!(
                Artifacts::parse(&text),
                Err(ArtifactsError::BadCommand { kind: "node".into(), folder: "node_modules".into() })
            );
        }

        #[test]
        fn a_blank_or_space_padded_command_is_refused() {
            for command in ["   ", " npm install", "npm install "] {
                let text = changed("command = \"npm install\"", &format!("command = \"{command}\""));

                assert!(matches!(Artifacts::parse(&text), Err(ArtifactsError::BadCommand { .. })), "{command:?}");
            }
        }

        #[test]
        fn a_command_of_two_lines_is_refused() {
            let text = changed("command = \"npm install\"", "command = \"npm\\ninstall\"");

            assert!(matches!(Artifacts::parse(&text), Err(ArtifactsError::BadCommand { .. })));
        }

        #[test]
        fn a_command_outside_printable_ascii_is_refused() {
            let text = changed("command = \"npm install\"", "command = \"npm install \\u00e9\"");

            assert!(matches!(Artifacts::parse(&text), Err(ArtifactsError::BadCommand { .. })));
        }
    }

    mod positive {
        use fresher_core::ProjectKind;

        use super::Artifacts;
        use crate::categories::Category;
        use crate::table::kind_name;

        #[test]
        fn the_shipped_table_has_a_row_for_every_artifact_of_the_core() {
            let table = Artifacts::shipped().unwrap();

            for kind in ProjectKind::ALL {
                for folder in kind.artifacts() {
                    assert!(table.row(kind, folder).is_some(), "{} {folder}", kind_name(kind));
                }
            }
            assert_eq!(table.rows.len(), 10);
        }

        #[test]
        fn python_environments_are_the_only_medium_rows() {
            let table = Artifacts::shipped().unwrap();

            for row in &table.rows {
                let medium = row.category.danger() == fresher_core::DangerLevel::Medium;
                assert_eq!(medium, row.category == Category::PythonEnvironments, "{}", row.folder);
            }
            assert!(table.rows.iter().any(|row| row.category == Category::PythonEnvironments));
        }

        #[test]
        fn a_row_is_found_by_kind_and_folder() {
            let table = Artifacts::shipped().unwrap();

            assert_eq!(table.row(ProjectKind::Dart, "build").map(|row| row.command.as_str()), Some("flutter build"));
            assert!(table.row(ProjectKind::Go, "target").is_none());
        }
    }
}
