# 021/01/01 — What the walk saw below a folder

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** core, scanner, testing, engine,
e2e, app, docs

The engine must know, before the modules' `finish`, how big a project's artifact folder is and when the project's own
folders last changed. The walk has both in its tree, but the tree is reachable only after `commit`, which waits for
every `finish`. This step adds one read-only question to the walk port, answered from the last walk's tree. Nothing
else changes: no interest, no claim, no module. The design: [`../00-design.md`](../00-design.md), section 1 (agent's
calls 2 and 4).

## Files

- `crates/core/src/walk.rs` — `SubtreeQuery { pub folder: PathBuf, pub skip: Vec<PathBuf> }` (`Clone, Debug,
  PartialEq, Eq`) and `Subtree { pub sizes: Sizes, pub newest: Option<Timestamp>, pub complete: bool }` (`Clone, Copy,
  Debug, PartialEq, Eq`), documented as in the design; `DiskWalk` gains `fn subtrees(&self, queries: &[SubtreeQuery])
  -> Vec<Option<Subtree>>` with the design's doc (no disk is read; `None` for a path that is not a folder of the last
  walk's tree, and for every query before any walk). `crates/core/src/lib.rs` — both in the `walk` re-export.
- `crates/scanner/src/subtrees.rs` (new) — `pub(crate) fn subtrees(tree: &Tree, queries: &[SubtreeQuery]) ->
  Vec<Option<Subtree>>`: per query, the folder by `Tree::lookup`; `None` unless the node is a folder (`Flag::Folder`,
  not a link, not a kept file); the skips by `lookup`, kept as a set of node indices (a skip that is no node, or not
  below the folder, is ignored); an explicit stack over `Tree::children` (no recursion: trees are deep), never entering
  a skipped node; `newest` the max of `Node::modified` over every folder node visited, the queried folder included (a
  scan root has no time of its own); `complete` false when a visited node `is_incomplete()` or carries `OtherVolume`
  or `Collapsed`; an `Opaque` node (claimed whole) counts with its own time and is not entered (it has no children);
  `sizes` from the frozen node, built as `TreeState::info` builds them (`logical`, `allocated`, `reclaimable`, `Partial`
  when `Tree::completeness` says so). Kept files are not counted (design: folders only).
  `#[cfg(test)] #[path = "subtrees_tests.rs"] mod tests;` as `places_seen.rs` does.
- `crates/scanner/src/subtrees_tests.rs` (new) — trees built with the arena's test helpers (as `places_seen_tests.rs`).
- `crates/scanner/src/scanner.rs` — `impl DiskWalk for Scanner`: `subtrees` locks `self.last` (poison taken as
  `PoisonError::into_inner`, as `commit` does) and answers from `Walked::tree`; `None` for every query when there is no
  walk to commit. `crates/scanner/src/lib.rs` — `mod subtrees;`.
- `crates/testing/src/naive_walk.rs` — `Walked` keeps, for every folder the walk recorded, its path, its sizes (as
  today), its own modification time (`Metadata::modified`, the stated facts do not touch it) and whether it was listed
  in full (read, not avoided, not on another volume, not another user's); a folder inside a folder claimed whole is
  not recorded, as the scanner keeps no node of it; a scan root records no time of its own. `subtrees`: the query's
  folder among them (`None` otherwise), `newest` over the recorded folders at or below it and below no skip,
  `complete` over the same, `sizes` its recorded sizes. The type's doc gains one line on `subtrees`.
- `crates/engine/src/testing.rs` — `ScriptedWalk::answering_subtrees(self, answers: Vec<(PathBuf, Subtree)>) -> Self`
  and `subtree_queries(&self) -> Arc<Mutex<Vec<Vec<SubtreeQuery>>>>` (recorded, as `commits`); `subtrees` answers a
  query whose folder is given, `None` otherwise.
- `crates/e2e/tests/demo.rs` (`NoWalk`, `HeldWalk`) and `app/tauri/src/testing.rs` (`NoWalk`, `HeldWalk`) — `subtrees`
  answers `None` for every query (`vec![None; queries.len()]`).
- `crates/scanner/tests/reference.rs` — a property test beside the existing ones: on a generated tree (the existing
  strategies), every folder queried with a random set of skips below it; the scanner's and the reference's answers
  are equal in `newest`, `complete` and `sizes.logical` (`allocated` differs by design: `std::fs` sees no clones).
- `crates/scanner/benches/scan.rs` — a group `subtrees`: after a full scan of the bench tree, one call with a query per
  `Projects/app*` folder of `LargeTree`, each skipping its `node_modules` (`bench_function("projects/<tree name>")`).
- `docs/architecture/README.md` — the `DiskWalk` row of the ports table: "…; `subtrees` answers what the last walk
  saw below a folder (sizes, newest folder time) before the commit".

## Produces

`fresher_core::{SubtreeQuery, Subtree}` and `DiskWalk::subtrees`, for 01/02 (the finder, the engine) and 01/03 (the
harness). `ScriptedWalk::marker` scripts a folder holding a marker file, so 01/02's engine tests can find a project
(pre-flight amendment).

## Checks

- `subtrees_tests.rs`, negatives first:
  - `mod negative`: `a_path_the_tree_has_no_node_of_is_none`, `a_link_and_a_kept_file_are_none`,
    `a_folder_inside_a_folder_claimed_whole_is_none`, `a_skipped_folder_s_newer_time_does_not_count`,
    `an_unreadable_folder_makes_it_incomplete`, `an_avoided_or_other_volume_folder_makes_it_incomplete`,
    `a_collapsed_folder_makes_it_incomplete`, `an_unreadable_folder_that_is_skipped_keeps_it_complete`,
    `a_kept_file_s_time_does_not_count`, `before_any_walk_every_answer_is_none` (a `Scanner` that never walked);
  - `mod positive`: `the_newest_time_is_the_newest_folder_below_it`,
    `a_folder_claimed_whole_counts_with_its_own_time`, `the_sizes_are_the_frozen_subtree_s_sums`,
    `a_skip_that_is_no_node_or_lies_elsewhere_is_ignored`, `answers_come_in_the_order_of_the_queries`,
    `a_restore_answers_from_the_cached_tree` (through the scanner, with a cache, as the restore tests do).
- `naive_walk.rs` tests: `a_folder_not_listed_makes_the_naive_answer_incomplete` (an avoided folder), then
  `the_naive_walk_answers_from_the_folders_it_recorded`.
- The reference property test `the_scanner_and_the_reference_agree_on_subtrees` passes (and fails, once by hand, when
  the scanner counts kept files' times: written in the conclusions, then reverted).
- `npm run bench` (`cargo bench -p fresher-scanner --bench scan`): the `scan` group before and after does not move by
  more than 5 %; the `subtrees` group's time written in the conclusions (rule 4).
- `cargo nextest run -p fresher-core -p fresher-scanner -p fresher-testing -p fresher-engine -p fresher-e2e -p
  fresher-app`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt --check`.

## Conclusions

Done as written, with the pre-flight amendments: the engine's use of `subtrees` is 01/02's (the `expect(dead_code)` on
`ScriptedWalk::{answering_subtrees, subtree_queries}` name 01/02), and `ScriptedWalk::marker(marker, path)` delivers a
`MarkerFile` match (its found folders now carry the pattern they match), with a test of its own. Notes:

- A skip equal to the queried folder, above it or elsewhere is never met on the way down from the folder, so it is
  ignored without a check; `NaiveWalk` keeps only the skips strictly below the folder. A folder claimed whole is
  recorded by the reference as listed in full when its sizes are complete, as the scanner marks it `Incomplete`
  otherwise.
- The reference property test generates its trees without links: the two walks sum them differently by design (the
  scanner's tree counts a link's own bytes, its target's length, and a hard link group's bytes once; the reference
  neither), so `sizes.logical` would differ for that reason alone. It dates every folder older than every file of the
  random trees; counting kept files' times in the scanner (once, by hand, then reverted) made it fail on its first
  cases (a root holding a matched file: `Some(1500000000)` against `None`).
- The bench's queries use the canonical root (the tree keeps it; the bench tree's path holds `..`).
- Bench, small tree, the Mac not quiet (load average 10 to 12): the `scan` group before and after, run back to back,
  moved within −8.5 % to +3.5 % (the full scan at 12 threads 64.5 ms → 65.1 ms, +1.1 %, not significant); no case
  slower by more than 5 %. `subtrees/projects/small` (20 projects in one call, each skipping its `node_modules`):
  11.6 µs.
