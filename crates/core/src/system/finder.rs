//! The projects of a scan, found during the one shared walk from the system's own interests (each project kind's
//! marker files, artifact folders and cache folders, owned by `ModuleId::system()`), then described by what the walk
//! saw of them.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::{
    Artifact, Claim, Completeness, DiskWalk, EntryKind, Interest, LastChange, Match, Pattern, Project, ProjectKind,
    Sizes, Subtree, SubtreeQuery,
};

/// A folder with this name holds installed packages, never a project of the user's.
const PACKAGES: &str = "node_modules";

/// The folder of a project's git repository.
const GIT: &str = ".git";

/// What one of the finder's interests looks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Look {
    /// A folder holding a marker of the kind.
    Marker(ProjectKind),
    /// A folder named like an artifact of some kind.
    Artifact,
    /// A folder named like the last segment of some kind's cache pattern.
    Cache,
}

/// The finder's interests, in a fixed order, with what each looks for.
fn looks() -> Vec<(Pattern, Look)> {
    let markers = ProjectKind::ALL.into_iter().flat_map(|kind| {
        kind.markers().iter().map(move |marker| (Pattern::MarkerFile((*marker).into()), Look::Marker(kind)))
    });
    let names: BTreeSet<&str> = ProjectKind::ALL.iter().flat_map(|kind| kind.artifacts().iter().copied()).collect();
    let artifacts = names.into_iter().map(|name| (Pattern::FolderName(name.into()), Look::Artifact));
    let caches: BTreeSet<&str> = ProjectKind::ALL
        .iter()
        .flat_map(|kind| kind.caches().iter().filter_map(|cache| cache.rsplit('/').next()))
        .collect();
    let caches = caches.into_iter().map(|name| (Pattern::FolderName(name.into()), Look::Cache));
    markers.chain(artifacts).chain(caches).collect()
}

/// Finds the projects of one walk from the matches of its own interests, then asks the walk what it saw of them.
#[derive(Debug)]
pub struct ProjectFinder {
    looks: Vec<Look>,
    roots: BTreeMap<PathBuf, BTreeSet<ProjectKind>>,
    artifacts: BTreeSet<PathBuf>,
    caches: BTreeSet<PathBuf>,
}

impl Default for ProjectFinder {
    fn default() -> Self {
        Self {
            looks: looks().into_iter().map(|(_, look)| look).collect(),
            roots: BTreeMap::new(),
            artifacts: BTreeSet::new(),
            caches: BTreeSet::new(),
        }
    }
}

/// A project found, before the walk is asked about it.
struct Found {
    root: PathBuf,
    kinds: BTreeSet<ProjectKind>,
    artifacts: Vec<PathBuf>,
    caches: Vec<PathBuf>,
}

impl ProjectFinder {
    /// The interests the walk carries for the projects, in a fixed order, owned by `ModuleId::system()`: a
    /// `MarkerFile` per marker, a `FolderName` per artifact name and per cache name, all `Claim::Item`, so the walk
    /// goes on inside and no module loses a match.
    pub fn interests() -> Vec<Interest> {
        looks().into_iter().filter_map(|(pattern, _)| Interest::new(pattern, Claim::Item, Vec::new()).ok()).collect()
    }

    /// Takes one match; `local` indexes `interests()`. Only folders count.
    pub fn add(&mut self, local: usize, matched: &Match) {
        if matched.kind != EntryKind::Folder {
            return;
        }
        match self.looks.get(local) {
            Some(Look::Marker(kind)) => {
                self.roots.entry(matched.path.clone()).or_default().insert(*kind);
            }
            Some(Look::Artifact) => {
                self.artifacts.insert(matched.path.clone());
            }
            Some(Look::Cache) => {
                self.caches.insert(matched.path.clone());
            }
            None => {}
        }
    }

    /// The projects, by root: the markers of one folder merged into one project with every kind; a project inside
    /// another's artifact folder, or inside a folder of installed packages, dropped; each with its artifacts, its
    /// caches and its last activity, asked of `walk` in one call. `None` (a walk that did not end) gives every size
    /// `Sizes::ZERO` partial and every time `Unknown`. Reads nothing but the walk's answers.
    pub fn finish(self, walk: Option<&dyn DiskWalk>) -> Vec<Project> {
        let found = self.found();
        let Some(walk) = walk else {
            return found.into_iter().map(unseen).collect();
        };
        let queries: Vec<SubtreeQuery> =
            found.iter().enumerate().flat_map(|(index, project)| queries(&found, index, project)).collect();
        let mut answers = walk.subtrees(&queries).into_iter();
        found.into_iter().map(|project| seen(project, &mut answers)).collect()
    }

    /// The projects kept, by root, with the artifact and cache folders matched in them.
    fn found(&self) -> Vec<Found> {
        let artifact_folders: Vec<PathBuf> =
            self.roots.iter().flat_map(|(root, kinds)| artifact_paths(root, kinds)).collect();
        let nested = |root: &Path| {
            root.iter().any(|name| name == PACKAGES)
                || artifact_folders.iter().any(|artifact| root.starts_with(artifact) && root != artifact.as_path())
        };
        self.roots
            .iter()
            .filter(|(root, _)| !nested(root))
            .map(|(root, kinds)| {
                let artifacts: Vec<PathBuf> =
                    artifact_paths(root, kinds).into_iter().filter(|path| self.artifacts.contains(path)).collect();
                let caches = self.caches_of(root, kinds, &artifacts);
                Found { root: root.clone(), kinds: kinds.clone(), artifacts, caches }
            })
            .collect()
    }

    /// The matched cache folders of a project: inside one of its artifacts, at a path relative to its root that one of
    /// its kinds' patterns names, and inside none of its other caches.
    fn caches_of(&self, root: &Path, kinds: &BTreeSet<ProjectKind>, artifacts: &[PathBuf]) -> Vec<PathBuf> {
        let patterns: Vec<&str> = kinds.iter().flat_map(|kind| kind.caches().iter().copied()).collect();
        let named: Vec<&PathBuf> = self
            .caches
            .range(root.to_path_buf()..)
            .take_while(|path| path.starts_with(root))
            .filter(|path| artifacts.iter().any(|artifact| path.starts_with(artifact)))
            .filter(|path| path.strip_prefix(root).is_ok_and(|relative| patterns.iter().any(|p| names(relative, p))))
            .collect();
        named
            .iter()
            .filter(|path| !named.iter().any(|other| other != *path && path.starts_with(other)))
            .map(|path| (*path).clone())
            .collect()
    }
}

/// The artifact folders a project of these kinds may have, by path, each once.
fn artifact_paths(root: &Path, kinds: &BTreeSet<ProjectKind>) -> BTreeSet<PathBuf> {
    kinds.iter().flat_map(|kind| kind.artifacts().iter().map(|name| root.join(name))).collect()
}

/// Whether `relative` is the path `pattern` names, segment by segment, `*` one name.
fn names(relative: &Path, pattern: &str) -> bool {
    let segments: Vec<_> = relative.iter().collect();
    let wanted: Vec<&str> = pattern.split('/').collect();
    segments.len() == wanted.len()
        && segments.iter().zip(&wanted).all(|(segment, wanted)| *wanted == "*" || *segment == *wanted)
}

/// The queries of one project, in order: its own folders, its branches, its `.git`, each artifact, each cache.
fn queries(found: &[Found], index: usize, project: &Found) -> Vec<SubtreeQuery> {
    let git = project.root.join(GIT);
    let mut skip = project.artifacts.clone();
    skip.push(git.clone());
    for inner in found.iter().skip(index + 1).take_while(|inner| inner.root.starts_with(&project.root)) {
        skip.extend(inner.artifacts.iter().cloned());
        skip.push(inner.root.join(GIT));
    }
    let mut queries = vec![
        SubtreeQuery { folder: project.root.clone(), skip },
        SubtreeQuery { folder: git.join("refs").join("heads"), skip: Vec::new() },
        SubtreeQuery { folder: git.clone(), skip: vec![git.join("objects"), git.join("modules")] },
    ];
    let folders = project.artifacts.iter().chain(&project.caches);
    queries.extend(folders.map(|folder| SubtreeQuery { folder: folder.clone(), skip: Vec::new() }));
    queries
}

/// A project described by the walk's answers to its queries, taken from `answers` in the order of `queries`.
fn seen(project: Found, answers: &mut impl Iterator<Item = Option<Subtree>>) -> Project {
    let own = answers.next().flatten();
    let branches = answers.next().flatten();
    let git = answers.next().flatten();
    let newest = own.and_then(|own| own.newest).max(branches.and_then(|branches| branches.newest));
    // No branches folder is complete only when the walk read the `.git` folder in full, or there is none.
    let branches_complete = branches.map_or_else(|| git.is_none_or(|git| git.complete), |branches| branches.complete);
    let complete = own.is_some_and(|own| own.complete) && branches_complete;
    let mut described = |paths: Vec<PathBuf>| -> Vec<Artifact> {
        let mut described = Vec::with_capacity(paths.len());
        for path in paths {
            if let Some(subtree) = answers.next().flatten() {
                let last_change = LastChange::of(subtree.newest, subtree.complete);
                described.push(Artifact { path, sizes: subtree.sizes, last_change });
            }
        }
        described
    };
    let artifacts = described(project.artifacts);
    let caches = described(project.caches);
    Project {
        root: project.root,
        kinds: project.kinds.into_iter().collect(),
        artifacts,
        caches,
        last_activity: LastChange::of(newest, complete),
    }
}

/// A project the walk was not asked about: partial empty sizes, every time unknown.
fn unseen(project: Found) -> Project {
    let unknown = |path| Artifact {
        path,
        sizes: Sizes { completeness: Completeness::Partial, ..Sizes::ZERO },
        last_change: LastChange::Unknown,
    };
    Project {
        root: project.root,
        kinds: project.kinds.into_iter().collect(),
        artifacts: project.artifacts.into_iter().map(unknown).collect(),
        caches: project.caches.into_iter().map(unknown).collect(),
        last_activity: LastChange::Unknown,
    }
}

#[cfg(test)]
#[path = "finder_tests.rs"]
mod tests;
