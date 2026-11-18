//! The projects of a scan, found during the one shared walk: the engine's own interests (each project kind's marker
//! files and artifact folders), owned by `ModuleId::system()`, and the collector of their matches.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use fresher_core::{Claim, EntryKind, Interest, Match, Pattern, Project, ProjectKind, Timestamp};

/// A folder with this name holds installed packages, never a project of the user's.
const PACKAGES: &str = "node_modules";

/// What one of the engine's interests looks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Look {
    /// A folder holding a marker of the kind.
    Marker(ProjectKind),
    /// A folder named like an artifact of some kind.
    Artifact,
}

/// The engine's interests, in a fixed order, with what each looks for.
fn looks() -> Vec<(Pattern, Look)> {
    let markers = ProjectKind::ALL.into_iter().flat_map(|kind| {
        kind.markers().iter().map(move |marker| (Pattern::MarkerFile((*marker).into()), Look::Marker(kind)))
    });
    let names: BTreeSet<&str> = ProjectKind::ALL.iter().flat_map(|kind| kind.artifacts().iter().copied()).collect();
    let artifacts = names.into_iter().map(|name| (Pattern::FolderName(name.into()), Look::Artifact));
    markers.chain(artifacts).collect()
}

/// The interests the engine adds to every walk for the projects. Each claims only its entry, so the walk goes on
/// inside and no module loses a match.
pub(crate) fn interests() -> Vec<Interest> {
    looks().into_iter().filter_map(|(pattern, _)| Interest::new(pattern, Claim::Item, Vec::new()).ok()).collect()
}

/// A folder seen as a project, while the walk runs.
#[derive(Debug, Default)]
struct Found {
    kinds: BTreeSet<ProjectKind>,
    modified: Option<Timestamp>,
}

/// Collects the matches of the engine's interests during the walk; `finish` makes the projects.
#[derive(Debug)]
pub(crate) struct ProjectCollector {
    looks: Vec<Look>,
    roots: BTreeMap<PathBuf, Found>,
    artifacts: BTreeSet<PathBuf>,
}

impl Default for ProjectCollector {
    fn default() -> Self {
        Self {
            looks: looks().into_iter().map(|(_, look)| look).collect(),
            roots: BTreeMap::new(),
            artifacts: BTreeSet::new(),
        }
    }
}

impl ProjectCollector {
    /// Takes one match; `local` indexes the engine's own `interests()`.
    pub(crate) fn add(&mut self, local: usize, matched: &Match) {
        if matched.kind != EntryKind::Folder {
            return;
        }
        match self.looks.get(local) {
            Some(Look::Marker(kind)) => {
                let found = self.roots.entry(matched.path.clone()).or_default();
                found.kinds.insert(*kind);
                found.modified = found.modified.or(matched.modified);
            }
            Some(Look::Artifact) => {
                self.artifacts.insert(matched.path.clone());
            }
            None => {}
        }
    }

    /// The projects, by root: the markers of one folder merged into one project with every kind; a project inside
    /// another project's artifact folder, or inside a folder of installed packages, dropped; each with the artifact
    /// folders of its kinds that the walk saw in it.
    pub(crate) fn finish(self) -> Vec<Project> {
        let artifact_folders: Vec<PathBuf> = self
            .roots
            .iter()
            .flat_map(|(root, found)| {
                found.kinds.iter().flat_map(|kind| kind.artifacts().iter().map(|name| root.join(name)))
            })
            .collect();
        let nested = |root: &Path| {
            root.iter().any(|name| name == PACKAGES)
                || artifact_folders.iter().any(|artifact| root.starts_with(artifact) && root != artifact.as_path())
        };
        self.roots
            .into_iter()
            .filter(|(root, _)| !nested(root))
            .map(|(root, found)| {
                let artifacts = found
                    .kinds
                    .iter()
                    .flat_map(|kind| kind.artifacts().iter().map(|name| root.join(name)))
                    .filter(|path| self.artifacts.contains(path))
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                Project { kinds: found.kinds.into_iter().collect(), artifacts, last_activity: found.modified, root }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fresher_core::{EntryFacts, EntryKind, Match, Pattern, Sizes, Timestamp};

    use super::{ProjectCollector, interests};

    /// The engine's interest index of `pattern`.
    fn index(pattern: &Pattern) -> usize {
        interests().iter().position(|interest| interest.pattern() == pattern).unwrap()
    }

    fn folder(pattern: &Pattern, path: &str, modified: u64) -> (usize, Match) {
        let matched = Match {
            interest: 0,
            path: PathBuf::from(path),
            kind: EntryKind::Folder,
            sizes: Sizes::ZERO,
            modified: Some(Timestamp::from_secs(modified)),
            changed: None,
            accessed: None,
            inside: None,
            siblings: Vec::new(),
            marker: None,
            facts: EntryFacts::default(),
        };
        (index(pattern), matched)
    }

    fn marker(name: &str, path: &str) -> (usize, Match) {
        folder(&Pattern::MarkerFile(name.into()), path, 100)
    }

    fn artifact(name: &str, path: &str) -> (usize, Match) {
        folder(&Pattern::FolderName(name.into()), path, 100)
    }

    fn collect(matches: Vec<(usize, Match)>) -> Vec<fresher_core::Project> {
        let mut collector = ProjectCollector::default();
        for (local, matched) in matches {
            collector.add(local, &matched);
        }
        collector.finish()
    }

    mod negative {
        use fresher_core::EntryKind;

        use super::{artifact, collect, marker};
        use crate::projects::ProjectCollector;

        #[test]
        fn packages_and_projects_inside_artifact_folders_are_not_projects() {
            let projects = collect(vec![
                marker("package.json", "/home/web/node_modules/left-pad"),
                marker("package.json", "/home/loose/node_modules/x"),
                marker("Cargo.toml", "/home/tool/target/package/tool-1.0"),
                marker("package.json", "/home/web"),
                marker("Cargo.toml", "/home/tool"),
                artifact("target", "/home/tool/target"),
            ]);

            let roots: Vec<_> = projects.iter().map(|project| project.root.to_str().unwrap()).collect();
            assert_eq!(roots, ["/home/tool", "/home/web"]);
        }

        #[test]
        fn a_file_match_an_unknown_index_and_a_lone_artifact_make_nothing() {
            let (local, mut file) = marker("package.json", "/home/web");
            file.kind = EntryKind::File;
            let mut collector = ProjectCollector::default();
            collector.add(local, &file);
            collector.add(usize::MAX, &file);
            let (local, folder) = artifact("node_modules", "/home/web/node_modules");
            collector.add(local, &folder);

            assert_eq!(collector.finish(), []);
        }

        #[test]
        fn an_artifact_of_another_kind_is_not_the_projects() {
            let projects = collect(vec![marker("go.mod", "/home/svc"), artifact("target", "/home/svc/target")]);

            assert_eq!(projects[0].artifacts, Vec::<std::path::PathBuf>::new());
        }
    }

    mod positive {
        use std::path::PathBuf;

        use fresher_core::{ProjectKind, Timestamp};

        use super::{artifact, collect, folder, marker};
        use crate::projects::interests;

        #[test]
        fn markers_of_one_folder_make_one_project_with_every_kind_and_its_artifacts() {
            let projects = collect(vec![
                artifact("node_modules", "/home/app/node_modules"),
                artifact("target", "/home/app/target"),
                marker("Cargo.toml", "/home/app"),
                folder(&fresher_core::Pattern::MarkerFile("package.json".into()), "/home/app", 7),
                marker("build.gradle.kts", "/home/android"),
                marker("settings.gradle.kts", "/home/android"),
                artifact("build", "/home/android/build"),
                artifact(".gradle", "/home/android/.gradle"),
            ]);

            assert_eq!(projects.len(), 2);
            let android = &projects[0];
            assert_eq!(android.kinds, [ProjectKind::Gradle]);
            assert_eq!(
                android.artifacts,
                [PathBuf::from("/home/android/.gradle"), PathBuf::from("/home/android/build")]
            );
            let app = &projects[1];
            assert_eq!(app.kinds, [ProjectKind::Node, ProjectKind::Rust]);
            assert_eq!(app.artifacts, [PathBuf::from("/home/app/node_modules"), PathBuf::from("/home/app/target")]);
            assert_eq!(app.last_activity, Some(Timestamp::from_secs(100)));
        }

        #[test]
        fn every_marker_and_artifact_name_has_one_item_interest() {
            let interests = interests();
            let markers: usize = ProjectKind::ALL.iter().map(|kind| kind.markers().len()).sum();

            assert!(interests.iter().all(|interest| interest.claim() == fresher_core::Claim::Item));
            assert_eq!(interests.len(), markers + 8);
        }
    }
}
