//! Projects: folders a developer works in, known by their marker files, with the folders their tools regenerate.

use std::path::PathBuf;

use crate::Timestamp;

/// A kind of project, known by its marker files.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProjectKind {
    Node,
    Rust,
    Gradle,
    Maven,
    Python,
    SwiftPackage,
    Go,
    Dart,
}

impl ProjectKind {
    /// Every kind, in a fixed order.
    pub const ALL: [Self; 8] =
        [Self::Node, Self::Rust, Self::Gradle, Self::Maven, Self::Python, Self::SwiftPackage, Self::Go, Self::Dart];

    /// The file names that make a folder a project of this kind.
    pub const fn markers(self) -> &'static [&'static str] {
        match self {
            Self::Node => &["package.json"],
            Self::Rust => &["Cargo.toml"],
            Self::Gradle => &["build.gradle", "build.gradle.kts", "settings.gradle", "settings.gradle.kts"],
            Self::Maven => &["pom.xml"],
            Self::Python => &["pyproject.toml", "setup.py"],
            Self::SwiftPackage => &["Package.swift"],
            Self::Go => &["go.mod"],
            Self::Dart => &["pubspec.yaml"],
        }
    }

    /// The folders next to the markers that the kind's tools make and can make again.
    pub const fn artifacts(self) -> &'static [&'static str] {
        match self {
            Self::Node => &["node_modules"],
            Self::Rust | Self::Maven => &["target"],
            Self::Gradle => &["build", ".gradle"],
            Self::Python => &[".venv", "venv"],
            Self::SwiftPackage => &[".build"],
            Self::Go => &[],
            Self::Dart => &[".dart_tool", "build"],
        }
    }

    /// The kind a marker file name makes; `None` for any other name.
    pub fn of_marker(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.markers().contains(&name))
    }
}

/// A project found during the scan's walk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    /// The folder holding the markers.
    pub root: PathBuf,
    /// Its kinds, in the order of `ProjectKind::ALL`; a folder may be several (a Rust crate with a `package.json`).
    pub kinds: Vec<ProjectKind>,
    /// The kinds' artifact folders that exist in it, by path.
    pub artifacts: Vec<PathBuf>,
    /// When the project folder itself last changed (a file added, removed or renamed at its top); `None` when the
    /// walk could not tell.
    pub last_activity: Option<Timestamp>,
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::ProjectKind;

        #[test]
        fn a_name_that_is_no_marker_makes_no_project() {
            for name in ["node_modules", "Cargo.lock", "package-lock.json", "PACKAGE.JSON", ""] {
                assert_eq!(ProjectKind::of_marker(name), None, "{name}");
            }
        }

        #[test]
        fn no_marker_belongs_to_two_kinds_and_no_artifact_is_a_marker() {
            let markers: Vec<_> = ProjectKind::ALL.iter().flat_map(|kind| kind.markers()).collect();
            let mut unique = markers.clone();
            unique.sort();
            unique.dedup();

            assert_eq!(unique.len(), markers.len());
            for kind in ProjectKind::ALL {
                assert!(kind.artifacts().iter().all(|artifact| ProjectKind::of_marker(artifact).is_none()));
            }
        }
    }

    mod positive {
        use crate::ProjectKind;

        #[test]
        fn every_kind_has_a_marker_that_names_it() {
            for kind in ProjectKind::ALL {
                assert!(!kind.markers().is_empty(), "{kind:?}");
                for marker in kind.markers() {
                    assert_eq!(ProjectKind::of_marker(marker), Some(kind));
                }
            }
            assert_eq!(ProjectKind::Node.artifacts(), ["node_modules"]);
            assert_eq!(ProjectKind::Go.artifacts(), [] as [&str; 0]);
        }
    }
}
