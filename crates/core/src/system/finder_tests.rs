use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::{
    ByteSize, ClaimedPath, DiskState, DiskWalk, EntryFacts, EntryKind, Match, Pattern, Project, Sizes, Subtree,
    SubtreeQuery, Timestamp, WalkError, WalkProgress, WalkRequest, WalkSummary,
};

use super::ProjectFinder;

/// A walk that answers `subtrees` from a map and records the queries; it never walks.
#[derive(Debug, Default)]
struct Answers {
    answers: BTreeMap<PathBuf, Subtree>,
    asked: Mutex<Vec<Vec<SubtreeQuery>>>,
}

impl Answers {
    fn with(answers: &[(&str, Subtree)]) -> Self {
        let answers = answers.iter().map(|(path, subtree)| (PathBuf::from(path), *subtree)).collect();
        Self { answers, asked: Mutex::default() }
    }

    fn asked(&self) -> Vec<Vec<SubtreeQuery>> {
        self.asked.lock().unwrap().clone()
    }
}

impl DiskWalk for Answers {
    fn walk(
        &self,
        _request: &WalkRequest<'_>,
        _cancel: &crate::CancelToken,
        _on_match: &mut dyn FnMut(Match),
        _on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<WalkSummary, WalkError> {
        Err(WalkError::Cancelled)
    }

    fn commit(&self, _claims: &[ClaimedPath]) -> Result<Arc<dyn DiskState>, WalkError> {
        Err(WalkError::Cancelled)
    }

    fn subtrees(&self, queries: &[SubtreeQuery]) -> Vec<Option<Subtree>> {
        self.asked.lock().unwrap().push(queries.to_vec());
        queries.iter().map(|query| self.answers.get(&query.folder).copied()).collect()
    }
}

/// What the walk saw below a folder: `bytes` in it, its newest folder time and whether it was read in full.
fn subtree(newest: Option<u64>, complete: bool, bytes: u64) -> Subtree {
    let sizes = Sizes::complete(ByteSize::new(bytes), ByteSize::new(bytes));
    Subtree { sizes, newest: newest.map(Timestamp::from_secs), complete }
}

/// The finder's interest index of `pattern`.
fn index(pattern: &Pattern) -> usize {
    ProjectFinder::interests().iter().position(|interest| interest.pattern() == pattern).unwrap()
}

fn folder(pattern: &Pattern, path: &str) -> (usize, Match) {
    let matched = Match {
        interest: 0,
        path: PathBuf::from(path),
        kind: EntryKind::Folder,
        sizes: Sizes::ZERO,
        modified: Some(Timestamp::from_secs(100)),
        changed: None,
        accessed: None,
        inside: None,
        siblings: Vec::new(),
        holds: Vec::new(),
        marker: None,
        facts: EntryFacts::default(),
    };
    (index(pattern), matched)
}

fn marker(name: &str, path: &str) -> (usize, Match) {
    folder(&Pattern::MarkerFile(name.into()), path)
}

/// A folder matched by its name: an artifact's or a cache's.
fn named(name: &str, path: &str) -> (usize, Match) {
    folder(&Pattern::FolderName(name.into()), path)
}

fn finder(matches: Vec<(usize, Match)>) -> ProjectFinder {
    let mut finder = ProjectFinder::default();
    for (local, matched) in matches {
        finder.add(local, &matched);
    }
    finder
}

fn collect(matches: Vec<(usize, Match)>) -> Vec<Project> {
    finder(matches).finish(None)
}

fn paths(artifacts: &[crate::Artifact]) -> Vec<PathBuf> {
    artifacts.iter().map(|artifact| artifact.path.clone()).collect()
}

/// A Rust project at `/home/tool` with its `target` and the cache folders at `caches`.
fn rust_with_caches(caches: &[&str]) -> Vec<(usize, Match)> {
    let mut matches = vec![marker("Cargo.toml", "/home/tool"), named("target", "/home/tool/target")];
    for cache in caches {
        let name = cache.rsplit('/').next().unwrap();
        matches.push(named(name, cache));
    }
    matches
}

mod negative {
    use std::path::PathBuf;

    use crate::{Completeness, EntryKind, LastChange, Sizes, Timestamp};

    use super::{Answers, ProjectFinder, collect, finder, marker, named, paths, rust_with_caches, subtree};

    #[test]
    fn packages_and_projects_inside_artifact_folders_are_not_projects() {
        let projects = collect(vec![
            marker("package.json", "/home/web/node_modules/left-pad"),
            marker("package.json", "/home/loose/node_modules/x"),
            marker("Cargo.toml", "/home/tool/target/package/tool-1.0"),
            marker("package.json", "/home/web"),
            marker("Cargo.toml", "/home/tool"),
            named("target", "/home/tool/target"),
        ]);

        let roots: Vec<_> = projects.iter().map(|project| project.root.to_str().unwrap()).collect();
        assert_eq!(roots, ["/home/tool", "/home/web"]);
    }

    #[test]
    fn a_project_deep_inside_another_s_artifact_folder_is_not_a_project() {
        let projects = collect(vec![
            marker("Cargo.toml", "/home/tool"),
            marker("package.json", "/home/tool/target/a/b/c/web"),
            marker("Cargo.toml", "/home/tool/targets/kept"),
        ]);

        let roots: Vec<_> = projects.iter().map(|project| project.root.to_str().unwrap()).collect();
        assert_eq!(roots, ["/home/tool", "/home/tool/targets/kept"]);
    }

    #[test]
    fn a_file_match_an_unknown_index_and_a_lone_artifact_make_nothing() {
        let (local, mut file) = marker("package.json", "/home/web");
        file.kind = EntryKind::File;
        let mut finder = ProjectFinder::default();
        finder.add(local, &file);
        finder.add(usize::MAX, &file);
        let (local, folder) = named("node_modules", "/home/web/node_modules");
        finder.add(local, &folder);

        assert_eq!(finder.finish(None), []);
    }

    #[test]
    fn an_artifact_of_another_kind_is_not_the_projects() {
        let projects = collect(vec![marker("go.mod", "/home/svc"), named("target", "/home/svc/target")]);

        assert_eq!(paths(&projects[0].artifacts), Vec::<PathBuf>::new());
    }

    #[test]
    fn a_cache_outside_its_kind_s_artifact_is_none() {
        let projects = collect(vec![
            marker("package.json", "/home/app"),
            named("build", "/home/app/build"),
            named("incremental", "/home/app/build/incremental"),
        ]);

        assert_eq!(projects[0].caches, []);
    }

    #[test]
    fn a_cache_of_another_kind_is_none() {
        let projects = collect(vec![
            marker("pom.xml", "/home/svc"),
            named("target", "/home/svc/target"),
            named("incremental", "/home/svc/target/debug/incremental"),
        ]);

        assert_eq!(paths(&projects[0].artifacts), [PathBuf::from("/home/svc/target")]);
        assert_eq!(projects[0].caches, []);
    }

    #[test]
    fn a_cache_whose_artifact_was_not_matched_is_none() {
        let projects = collect(vec![
            marker("Cargo.toml", "/home/tool"),
            named("incremental", "/home/tool/target/debug/incremental"),
        ]);

        assert_eq!(projects[0].artifacts, []);
        assert_eq!(projects[0].caches, []);
    }

    #[test]
    fn a_cache_deeper_than_its_pattern_is_none() {
        let projects = collect(rust_with_caches(&["/home/tool/target/a/b/c/incremental"]));

        assert_eq!(projects[0].caches, []);
    }

    #[test]
    fn a_cache_inside_another_cache_is_none() {
        let projects = collect(rust_with_caches(&[
            "/home/tool/target/llvm-cov-target",
            "/home/tool/target/llvm-cov-target/debug/incremental",
        ]));

        assert_eq!(paths(&projects[0].caches), [PathBuf::from("/home/tool/target/llvm-cov-target")]);
    }

    #[test]
    fn an_artifact_the_walk_has_no_folder_of_is_dropped() {
        let walk = Answers::with(&[("/home/tool", subtree(Some(10), true, 1))]);

        let projects = finder(rust_with_caches(&["/home/tool/target/debug/incremental"])).finish(Some(&walk));

        assert_eq!(projects[0].artifacts, []);
        assert_eq!(projects[0].caches, []);
        assert_eq!(projects[0].last_activity, LastChange::Known(Timestamp::from_secs(10)));
    }

    #[test]
    fn no_walk_gives_unknown_times_and_partial_sizes() {
        let projects = collect(rust_with_caches(&["/home/tool/target/debug/incremental"]));

        let partial = Sizes { completeness: Completeness::Partial, ..Sizes::ZERO };
        let project = &projects[0];
        assert_eq!(project.last_activity, LastChange::Unknown);
        for artifact in project.artifacts.iter().chain(&project.caches) {
            assert_eq!(artifact.sizes, partial);
            assert_eq!(artifact.last_change, LastChange::Unknown);
        }
        assert_eq!(paths(&project.artifacts), [PathBuf::from("/home/tool/target")]);
        assert_eq!(paths(&project.caches), [PathBuf::from("/home/tool/target/debug/incremental")]);
    }

    #[test]
    fn an_incomplete_answer_is_at_least() {
        let own_incomplete = Answers::with(&[
            ("/home/tool", subtree(Some(10), false, 1)),
            ("/home/tool/.git", subtree(Some(5), true, 1)),
            ("/home/tool/.git/refs/heads", subtree(Some(5), true, 1)),
            ("/home/tool/target", subtree(Some(30), false, 2)),
        ]);
        let branches_incomplete = Answers::with(&[
            ("/home/tool", subtree(Some(10), true, 1)),
            ("/home/tool/.git", subtree(Some(5), true, 1)),
            ("/home/tool/.git/refs/heads", subtree(Some(5), false, 1)),
        ]);

        let own = finder(rust_with_caches(&[])).finish(Some(&own_incomplete));
        let branches = finder(rust_with_caches(&[])).finish(Some(&branches_incomplete));

        assert_eq!(own[0].last_activity, LastChange::AtLeast(Timestamp::from_secs(10)));
        assert_eq!(own[0].artifacts[0].last_change, LastChange::AtLeast(Timestamp::from_secs(30)));
        assert_eq!(branches[0].last_activity, LastChange::AtLeast(Timestamp::from_secs(10)));
    }

    #[test]
    fn an_unreadable_git_folder_makes_the_activity_at_least() {
        let walk = Answers::with(&[
            ("/home/tool", subtree(Some(10), true, 1)),
            ("/home/tool/.git", subtree(Some(4), false, 1)),
        ]);

        let projects = finder(rust_with_caches(&[])).finish(Some(&walk));

        assert_eq!(projects[0].last_activity, LastChange::AtLeast(Timestamp::from_secs(10)));
    }

    #[test]
    fn neither_answer_having_a_time_is_unknown() {
        let walk = Answers::with(&[
            ("/home/tool", subtree(None, true, 1)),
            ("/home/tool/.git", subtree(None, true, 1)),
            ("/home/tool/.git/refs/heads", subtree(None, true, 1)),
            ("/home/tool/target", subtree(None, true, 3)),
        ]);

        let projects = finder(rust_with_caches(&[])).finish(Some(&walk));

        assert_eq!(projects[0].last_activity, LastChange::Unknown);
        assert_eq!(projects[0].artifacts[0].last_change, LastChange::Unknown);
    }
}

mod positive {
    use std::path::PathBuf;

    use crate::{ByteSize, Claim, LastChange, Pattern, ProjectKind, Sizes, SubtreeQuery, Timestamp};

    use super::{Answers, ProjectFinder, collect, finder, marker, named, paths, rust_with_caches, subtree};

    fn query(folder: &str, skip: &[&str]) -> SubtreeQuery {
        SubtreeQuery { folder: folder.into(), skip: skip.iter().map(PathBuf::from).collect() }
    }

    #[test]
    fn markers_of_one_folder_make_one_project_with_every_kind_and_its_artifacts() {
        let projects = collect(vec![
            named("node_modules", "/home/app/node_modules"),
            named("target", "/home/app/target"),
            marker("Cargo.toml", "/home/app"),
            marker("package.json", "/home/app"),
            marker("build.gradle.kts", "/home/android"),
            marker("settings.gradle.kts", "/home/android"),
            named("build", "/home/android/build"),
            named(".gradle", "/home/android/.gradle"),
        ]);

        assert_eq!(projects.len(), 2);
        let android = &projects[0];
        assert_eq!(android.kinds, [ProjectKind::Gradle]);
        assert_eq!(
            paths(&android.artifacts),
            [PathBuf::from("/home/android/.gradle"), PathBuf::from("/home/android/build")]
        );
        let app = &projects[1];
        assert_eq!(app.kinds, [ProjectKind::Node, ProjectKind::Rust]);
        assert_eq!(paths(&app.artifacts), [PathBuf::from("/home/app/node_modules"), PathBuf::from("/home/app/target")]);
    }

    #[test]
    fn every_marker_and_artifact_name_has_one_item_interest() {
        let interests = ProjectFinder::interests();
        let markers: usize = ProjectKind::ALL.iter().map(|kind| kind.markers().len()).sum();

        assert!(interests.iter().all(|interest| interest.claim() == Claim::Item));
        assert_eq!(interests.len(), markers + 8 + 2);
        let names: Vec<_> = interests[markers + 8..].iter().map(|interest| interest.pattern().clone()).collect();
        assert_eq!(names, [Pattern::FolderName("incremental".into()), Pattern::FolderName("llvm-cov-target".into())]);
    }

    #[test]
    fn the_own_folders_query_skips_artifacts_git_and_nested_projects_artifacts_and_git() {
        let walk = Answers::default();

        finder(vec![
            marker("package.json", "/home/mono"),
            named("node_modules", "/home/mono/node_modules"),
            marker("package.json", "/home/mono/packages/web"),
            named("node_modules", "/home/mono/packages/web/node_modules"),
            marker("Cargo.toml", "/home/mono-tool"),
            named("target", "/home/mono-tool/target"),
            named("incremental", "/home/mono-tool/target/debug/incremental"),
        ])
        .finish(Some(&walk));

        let skip_git = ["/home/mono/.git/objects", "/home/mono/.git/modules"];
        let web_git = ["/home/mono/packages/web/.git/objects", "/home/mono/packages/web/.git/modules"];
        let tool_git = ["/home/mono-tool/.git/objects", "/home/mono-tool/.git/modules"];
        assert_eq!(
            walk.asked(),
            [vec![
                query(
                    "/home/mono",
                    &[
                        "/home/mono/node_modules",
                        "/home/mono/.git",
                        "/home/mono/packages/web/node_modules",
                        "/home/mono/packages/web/.git",
                    ]
                ),
                query("/home/mono/.git/refs/heads", &[]),
                query("/home/mono/.git", &skip_git),
                query("/home/mono/node_modules", &[]),
                query(
                    "/home/mono/packages/web",
                    &["/home/mono/packages/web/node_modules", "/home/mono/packages/web/.git"]
                ),
                query("/home/mono/packages/web/.git/refs/heads", &[]),
                query("/home/mono/packages/web/.git", &web_git),
                query("/home/mono/packages/web/node_modules", &[]),
                query("/home/mono-tool", &["/home/mono-tool/target", "/home/mono-tool/.git"]),
                query("/home/mono-tool/.git/refs/heads", &[]),
                query("/home/mono-tool/.git", &tool_git),
                query("/home/mono-tool/target", &[]),
                query("/home/mono-tool/target/debug/incremental", &[]),
            ]]
        );
    }

    #[test]
    fn newer_branches_win_over_older_folders() {
        let walk = Answers::with(&[
            ("/home/tool", subtree(Some(100), true, 1)),
            ("/home/tool/.git", subtree(Some(150), true, 1)),
            ("/home/tool/.git/refs/heads", subtree(Some(200), true, 1)),
        ]);

        let projects = finder(rust_with_caches(&[])).finish(Some(&walk));

        assert_eq!(projects[0].last_activity, LastChange::Known(Timestamp::from_secs(200)));
    }

    #[test]
    fn a_project_without_git_is_known_from_its_folders() {
        let walk = Answers::with(&[("/home/tool", subtree(Some(100), true, 1))]);

        let projects = finder(rust_with_caches(&[])).finish(Some(&walk));

        assert_eq!(projects[0].last_activity, LastChange::Known(Timestamp::from_secs(100)));
    }

    #[test]
    fn every_rust_cache_shape_is_found() {
        let caches = [
            "/home/tool/target/aarch64-apple-darwin/release/incremental",
            "/home/tool/target/debug/incremental",
            "/home/tool/target/llvm-cov-target",
        ];

        let projects = collect(rust_with_caches(&caches));

        assert_eq!(paths(&projects[0].caches), caches.map(PathBuf::from));
    }

    #[test]
    fn artifacts_carry_the_walk_s_sizes() {
        let walk = Answers::with(&[
            ("/home/tool", subtree(Some(10), true, 1)),
            ("/home/tool/target", subtree(Some(20), true, 900)),
            ("/home/tool/target/debug/incremental", subtree(Some(30), true, 400)),
        ]);

        let projects = finder(rust_with_caches(&["/home/tool/target/debug/incremental"])).finish(Some(&walk));

        let (target, cache) = (&projects[0].artifacts[0], &projects[0].caches[0]);
        assert_eq!(target.sizes, Sizes::complete(ByteSize::new(900), ByteSize::new(900)));
        assert_eq!(target.last_change, LastChange::Known(Timestamp::from_secs(20)));
        assert_eq!(cache.sizes, Sizes::complete(ByteSize::new(400), ByteSize::new(400)));
        assert_eq!(cache.last_change, LastChange::Known(Timestamp::from_secs(30)));
    }

    #[test]
    fn the_walk_is_asked_once() {
        let walk = Answers::default();

        let projects = finder(vec![marker("Cargo.toml", "/home/a"), marker("go.mod", "/home/b")]).finish(Some(&walk));

        assert_eq!(projects.len(), 2);
        assert_eq!(walk.asked().len(), 1);
        assert_eq!(walk.asked()[0].len(), 6);
    }
}
