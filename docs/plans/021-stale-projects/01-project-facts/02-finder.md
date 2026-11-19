# 021/01/02 — The project finder in the core, used by the engine

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** core, engine, e2e

The projects' collector moves from the engine to the core as `ProjectFinder`, so that the engine and the module
harness find the same projects from the same walk (agent's call 5), and it learns what the walk saw of them: each
artifact folder's sizes, Rust's caches inside `target`, and the project's last activity from its own folders and its
git branches. The engine switches to it in the same step (the `Project` type changes, so they cannot be split); 01/03
gives it to the harness. The design: [`../00-design.md`](../00-design.md), sections 2.1 to 2.3 (agent's calls 1 to 3,
6, 7, 13, 14).

## Files

- `crates/core/src/system/projects.rs` —
  - `pub enum LastChange { Known(Timestamp), AtLeast(Timestamp), Unknown }` (`Clone, Copy, Debug, PartialEq, Eq`),
    documented as in the design, with `pub fn of(newest: Option<Timestamp>, complete: bool) -> Self` (`None` →
    `Unknown`; complete → `Known`; otherwise `AtLeast`);
  - `pub struct Artifact { pub path: PathBuf, pub sizes: Sizes, pub last_change: LastChange }` (`Clone, Debug,
    PartialEq, Eq`);
  - `Project` becomes `{ root, kinds, artifacts: Vec<Artifact>, caches: Vec<Artifact>, last_activity: LastChange }`,
    each field documented as in the design (section 2.2);
  - `pub const fn caches(self) -> &'static [&'static str]`: `Rust` → `["target/*/incremental",
    "target/*/*/incremental", "target/llvm-cov-target"]`, every other kind `&[]`. A cache folder's name is its
    pattern's last segment; no other API.
- `crates/core/src/system/finder.rs` (new) — the body of `crates/engine/src/projects.rs` moved (its `Look`, `looks()`,
  the `PACKAGES` rule, the merge of markers and the drop of nested projects, its tests), renamed and extended:
  - `Look` gains `Cache` (a folder named like the last segment of some kind's cache pattern); `looks()` adds one
    `FolderName` per such name after the artifact names, in name order, `Claim::Item`;
  - `pub struct ProjectFinder` (`Debug`, `Default`); `pub fn interests() -> Vec<Interest>`; `pub fn add(&mut self,
    local: usize, matched: &Match)` (folders only, as today; a cache name's match kept with the artifact ones);
    `pub fn finish(self, walk: Option<&dyn DiskWalk>) -> Vec<Project>`:
    1. the projects as today (by root; markers of one folder merged; a project inside another's artifact folder or
       inside `node_modules` dropped);
    2. each project's artifacts: the matched folders at `root/<artifact>` of its kinds (as today);
    3. its caches: the matched cache folders whose path relative to `root` matches one of its kinds' `caches()`
       segment by segment (`*` one name), and that lie inside none of the project's other caches;
    4. one `walk.subtrees(&queries)` call for every project, in this order per project: its own folders
       (`SubtreeQuery { folder: root, skip }`, `skip` = its artifacts, `root/.git`, and for every project whose root
       lies strictly inside this one, that project's artifacts and its `.git`), its branches (`root/.git/refs/heads`,
       no skip), each artifact, each cache (no skip);
    5. an artifact or a cache whose answer is `None` is dropped; otherwise `Artifact { path, sizes, last_change:
       LastChange::of(newest, complete) }`;
    6. `last_activity`: the newer of the two first answers' `newest`; complete when the own-folders answer is complete
       and the branches' answer is complete or `None`; through `LastChange::of`;
    7. `walk` `None`: every artifact and cache kept with `Sizes { completeness: Completeness::Partial, ..Sizes::ZERO }`
       and `Unknown`, every `last_activity` `Unknown`.
  - Never a status change, never a file's time, never a read of the disk: only the walk's answers.
- `crates/core/src/system/mod.rs` — `mod finder;`, `pub use finder::ProjectFinder;`, `pub use projects::{Artifact,
  LastChange, Project, ProjectKind};`. `crates/core/src/lib.rs` — the new names in the `system` re-export.
- `crates/core/src/system/facts.rs` — its `Project { … }` test literal takes the new fields; `set_projects`' doc: "Called
  by the engine after the walk, with what the walk saw of them."
- `crates/engine/src/projects.rs` removed, `mod projects;` out of `crates/engine/src/lib.rs`.
- `crates/engine/src/session.rs` — `use fresher_core::ProjectFinder`; `all_interests` adds
  `ProjectFinder::interests()` under `ModuleId::system()`; `let mut projects = ProjectFinder::default();`; after the
  walk, `system.set_projects(projects.finish(walk))` with `walk` = `Some(&*shared.ports.walk)` when `walked` is `Ok`
  (a walk or a restore that ended), `None` otherwise; `run`'s doc says the projects carry what the walk saw of them.
- `crates/engine/src/session.rs`'s tests (pre-flight C3: the engine's project tests are there, not in
  `facts_tests.rs`) —
  `the_projects_carry_what_the_walk_saw_of_them_before_any_finish` (a `ScriptedWalk` with
  `answering_subtrees`: the module's `finish` sees the sizes and `Known` time; the queries were asked after the walk
  and before the commit), `a_walk_that_did_not_end_asks_nothing` (cancelled: no subtree query).
- `crates/e2e/tests/projects.rs` — its assertions read the new shape: the artifacts by `path`; `tool.last_activity`
  is `Known`; plus, in `mod positive`, `a_nested_package_s_artifacts_are_not_the_outer_project_s_activity` (a second
  home with dated folders: an outer project 400 days old and a nested package whose `node_modules` is 3 days old; the
  outer project's `last_activity` is `Known` 400 days ago).

## Produces

`fresher_core::{ProjectFinder, Artifact, LastChange}`, `ProjectKind::caches`, the new `Project`, set by the engine for
every module's `finish`; for 01/03 and plan 02.

## Checks

A `DiskWalk` fake for the core's own tests (in `finder.rs`'s `tests`: answers `subtrees` from a map and records the
queries; its `walk` and `commit` return `WalkError::NothingToRestore`), negatives first:

- `projects.rs`: `mod negative` `every_cache_pattern_starts_with_one_of_its_kind_s_artifacts`; `mod positive`
  `rust_has_its_incremental_and_coverage_caches_and_the_other_kinds_none`, `last_change_of_reads_complete_as_known`.
- `finder.rs`, the moved tests unchanged in meaning (`packages_and_projects_inside_artifact_folders_are_not_projects`,
  `a_file_match_an_unknown_index_and_a_lone_artifact_make_nothing`, `an_artifact_of_another_kind_is_not_the_projects`,
  `markers_of_one_folder_make_one_project_with_every_kind_and_its_artifacts`,
  `every_marker_and_artifact_name_has_one_item_interest` now counting the two cache names), then:
  - `mod negative`: `a_cache_outside_its_kind_s_artifact_is_none` (`app/build/incremental` of a Node project),
    `a_cache_of_another_kind_is_none` (`svc/target/debug/incremental` of a Maven-only project),
    `a_cache_deeper_than_its_pattern_is_none` (`target/a/b/c/incremental`),
    `a_cache_inside_another_cache_is_none` (`target/llvm-cov-target/debug/incremental`),
    `an_artifact_the_walk_has_no_folder_of_is_dropped`, `no_walk_gives_unknown_times_and_partial_sizes`,
    `an_incomplete_answer_is_at_least`, `neither_answer_having_a_time_is_unknown`;
  - `mod positive`: `the_own_folders_query_skips_artifacts_git_and_nested_projects_artifacts_and_git` (the recorded
    queries), `newer_branches_win_over_older_folders`, `a_project_without_git_is_known_from_its_folders`,
    `every_rust_cache_shape_is_found` (`target/debug/incremental`, `target/aarch64-apple-darwin/release/incremental`,
    `target/llvm-cov-target`), `artifacts_carry_the_walk_s_sizes`, `the_walk_is_asked_once`.
- `cargo nextest run -p fresher-core -p fresher-engine -p fresher-e2e --test projects`; the whole
  `cargo nextest run -p fresher-e2e` (every flow still passes: the projects only gained facts); clippy; fmt.

## Conclusions

Done as written, with the pre-flight amendments:

- C3: the two engine tests are in `session.rs`'s `mod tests`; `facts_tests.rs` is untouched. The `expect(dead_code)` on
  `ScriptedWalk::{answering_subtrees, subtree_queries}` is gone (`crates/engine/src/testing.rs`).
- C4: each project has a third query, `root/.git` skipping `objects` and `modules`; the per-project order is own
  folders, branches, `.git`, artifacts, caches. A missing branches answer counts as complete only when the `.git`
  answer is missing or complete (`an_unreadable_git_folder_makes_the_activity_at_least`).
- C10: the e2e's dated home dates `mono`, `mono/packages` and `mono/packages/web` at 400 days and only
  `mono/packages/web/node_modules` at 3 days.
- C12 (a group's activity is the newest of its members'): the group is the module's (design 3.4, plan 02/02). The
  finder gives every member, the nested ones included, its own `last_activity` (its own folders and its own
  `.git/refs/heads`), which is what 02/02 needs to take the newest of a group.

Notes:

- The finder's tests are in `crates/core/src/system/finder_tests.rs` (`#[path]` from `finder.rs`), as the scanner's
  `subtrees_tests.rs`.
- A cache must also lie inside one of the project's matched artifact folders (the design's "inside its artifact
  folders"); with Rust's patterns all starting at `target`, this only drops a cache whose `target` was not matched.
- The finder keeps no match time any more: `last_activity` comes only from the walk's answers.
- Review fix: a folder claimed whole below a queried folder (not skipped) makes the answer incomplete, in the scanner
  and in `NaiveWalk` alike (the reference property test keeps them equal), so a project with a whole-claimed folder in
  its source is at most `AtLeast`. The cache-inside-a-matched-artifact check cannot be reached from a real walk (a
  cache is only walked inside a walked `target`, which is matched) but can from the finder's API; it stays, with
  `a_cache_whose_artifact_was_not_matched_is_none`.
