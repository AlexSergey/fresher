//! Projects: folders a developer works in, known by their marker files, with the folders their tools regenerate.

use std::path::PathBuf;

use crate::{Sizes, Timestamp};

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

    /// Folders inside the kind's artifact folders that only make the next build faster, relative to the project root,
    /// `*` one name: Rust's `target/*/incremental`, `target/*/*/incremental` (a target triple's) and
    /// `target/llvm-cov-target`; none for the other kinds. A cache folder's name is its pattern's last segment.
    pub const fn caches(self) -> &'static [&'static str] {
        match self {
            Self::Rust => &["target/*/incremental", "target/*/*/incremental", "target/llvm-cov-target"],
            Self::Node | Self::Gradle | Self::Maven | Self::Python | Self::SwiftPackage | Self::Go | Self::Dart => &[],
        }
    }

    /// The kind a marker file name makes; `None` for any other name.
    pub fn of_marker(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.markers().contains(&name))
    }
}

/// When something last changed, as far as the walk could tell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LastChange {
    /// Every folder counted was read: this is the newest change.
    Known(Timestamp),
    /// A folder could not be read: the newest change is this one or later.
    AtLeast(Timestamp),
    /// The walk saw nothing to date it.
    Unknown,
}

impl LastChange {
    /// The newest time the walk saw, `Known` when every folder counted was read (`complete`), `AtLeast` otherwise;
    /// `Unknown` when there is no time.
    pub const fn of(newest: Option<Timestamp>, complete: bool) -> Self {
        match newest {
            None => Self::Unknown,
            Some(time) if complete => Self::Known(time),
            Some(time) => Self::AtLeast(time),
        }
    }
}

/// A folder of a project that its tools make again, with what the walk saw of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Artifact {
    pub path: PathBuf,
    /// Everything inside it, as the walk summed it.
    pub sizes: Sizes,
    /// The newest change of it and of every folder inside it.
    pub last_change: LastChange,
}

/// A project found during the scan's walk, with what the walk saw of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    /// The folder holding the markers.
    pub root: PathBuf,
    /// Its kinds, in the order of `ProjectKind::ALL`; a folder may be several (a Rust crate with a `package.json`).
    pub kinds: Vec<ProjectKind>,
    /// The kinds' artifact folders that the walk saw in it, by path.
    pub artifacts: Vec<Artifact>,
    /// The kinds' cache folders that exist inside its artifact folders, by path; none inside another.
    pub caches: Vec<Artifact>,
    /// The newest change of its own folders (its artifact folders, its `.git` and the projects inside it, with their
    /// artifact folders and `.git`, left out) and of its git branches (`.git/refs/heads`).
    pub last_activity: LastChange,
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

        #[test]
        fn every_cache_pattern_starts_with_one_of_its_kind_s_artifacts() {
            for kind in ProjectKind::ALL {
                for pattern in kind.caches() {
                    let first = pattern.split('/').next().unwrap();
                    assert!(kind.artifacts().contains(&first), "{kind:?} {pattern}");
                    assert!(pattern.split('/').count() > 1, "{pattern}");
                }
            }
        }
    }

    mod positive {
        use crate::{LastChange, ProjectKind, Timestamp};

        #[test]
        fn rust_has_its_incremental_and_coverage_caches_and_the_other_kinds_none() {
            assert_eq!(
                ProjectKind::Rust.caches(),
                ["target/*/incremental", "target/*/*/incremental", "target/llvm-cov-target"]
            );
            for kind in ProjectKind::ALL.into_iter().filter(|kind| *kind != ProjectKind::Rust) {
                assert_eq!(kind.caches(), [] as [&str; 0], "{kind:?}");
            }
        }

        #[test]
        fn last_change_of_reads_complete_as_known() {
            let time = Timestamp::from_secs(7);

            assert_eq!(LastChange::of(Some(time), true), LastChange::Known(time));
            assert_eq!(LastChange::of(Some(time), false), LastChange::AtLeast(time));
            assert_eq!(LastChange::of(None, true), LastChange::Unknown);
            assert_eq!(LastChange::of(None, false), LastChange::Unknown);
        }

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
