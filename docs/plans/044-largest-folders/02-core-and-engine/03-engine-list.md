# 044/02/03 — The engine's Folders list

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** core, engine

`Engine::largest_folders`: the query, the judgement's verdict, the rows and the page; a plain folder is no longer a
pick. The design: [`../00-design.md`](../00-design.md), sections 4, 5 and 7 (the pick); agent's calls 6 to 10, 13,
16, 17 and 19.

## Read first

02/01 and 02/02's commits; `crates/core/src/largest/{mod.rs,page.rs}`; `crates/engine/src/largest/{mod.rs,
judge.rs,rule.rs,page.rs,largest_tests.rs,judge_tests.rs}`; `crates/engine/src/{own.rs,own_tests.rs}`;
`crates/engine/src/results.rs` (`Committed`).

## Files

- `crates/core/src/largest/folders.rs` (new) — `FoldersQuery`, `LeftFor` (with `between(newest, taken)`),
  `FolderRow`, `FoldersPage` exactly as the design's section 5; `crates/core/src/largest/mod.rs` and `lib.rs` export
  them.
- `crates/engine/src/largest/folders.rs` (new) — `enum Verdict { List, Enter, Drop }`, `fn verdict(ignored: Ignored,
  judged: (Offer, bool)) -> Verdict` (the design's table: the ignore list first, so ignored wins over holding
  explained items), `pub(crate) fn left_alone(ground: &Ground<'_>, home: NodeId, rule:
  &HomeItems, min: ByteSize, before: Timestamp) -> Vec<(NodeInfo, PathBuf, Timestamp)>` (the worklist, sorted largest
  first, ties in path order) and `pub(crate) fn folders_page(listed, query: &FoldersQuery, ground, home_sizes, min,
  days) -> FoldersPage` (`total`, `sizes`, `projects` over every listed folder; rows from `offset`, at most
  `query.limit.min(MAX_PAGE)`).
- `crates/engine/src/largest/mod.rs` — `mod folders;`; `Engine::largest_folders(&self, query: &FoldersQuery) ->
  Result<FoldersPage, CommandError>` (doc: refused as `largest`; a state without the home answers an empty page; the
  size and the days from the settings, the time from the scan, never the clock); the `HomeItems` built once by a
  helper shared with `largest`; `Ground::ignored(&self, path) -> Ignored` taken out of `Ground::judge`, which calls it
  (Files' order unchanged); `pickable` answers `None` for a `NodeKind::Folder` that is not `is_package`
  (its doc says so).
- `crates/engine/src/largest/page.rs` — `from_home` moves to `largest/mod.rs` as `pub(crate)`, used by both pages.
- `crates/engine/src/bench.rs` (`LargestBench`) and `crates/e2e/tests/largest_budget.rs` — `folders_of_the_large_tree`
  (ignored in the hooks): the list over the `LARGE` state at `min` 0 and `days` 0, every folder entered, under 50 ms
  and 5 MB; run by hand in release, the numbers in plan 02's conclusions.
- `app/tauri/src/{largest_commands.rs,apps_commands.rs}` — their tests that pick a folder (`an_offered_row_is_picked_at_high`
  picks `Projects`) pick a file of it instead; the sandbox harness's pick in `large-files.toml` is a file already.

## Interfaces produced

- `fresher_core::{FoldersQuery, FoldersPage, FolderRow, LeftFor}`; `Engine::largest_folders`. 02/04 and 02/05 call
  it.

## Tests (negatives first)

`crates/core/src/largest/folders.rs`:
- `mod negative`: `an_unknown_field_of_a_folders_query_is_refused`, `a_newest_after_the_scan_is_zero_months`.
- `mod positive`: `under_two_years_counts_months`, `from_two_years_counts_years`,
  `left_for_serializes_as_the_ui_reads_it` (`{"unit":"months","count":14}`).

`crates/engine/src/largest/folders_tests.rs` (new), on `FixedDiskState` homes with `with_newest`:
- `mod negative`: `no_scan_is_refused`, `a_running_scan_is_refused`, `a_state_without_the_home_answers_an_empty_page`,
  `the_home_itself_is_never_listed`, `the_library_and_the_trash_are_never_listed`,
  `a_hidden_folder_an_app_bundle_or_a_package_is_never_listed`, `a_standard_folder_itself_is_never_listed`,
  `a_protected_folder_and_what_is_inside_are_never_listed`, `an_ignored_folder_is_never_listed`,
  `a_folder_explained_whole_is_never_listed`, `an_ignored_folder_holding_explained_items_is_never_listed`,
  `a_folder_under_an_ignored_folder_is_never_listed`, `a_folder_a_cleanup_removed_is_not_listed`,
  `a_folder_with_a_recent_change_is_not_listed`, `a_folder_below_the_setting_is_not_listed`.
- `mod positive`: `a_stale_folder_is_listed_with_its_place_and_age`,
  `the_stale_children_of_a_standard_folder_are_listed`,
  `a_folder_holding_an_ignored_path_lists_its_other_stale_children`,
  `a_folder_holding_explained_items_is_listed_whole`, `a_folder_with_a_git_folder_is_a_project`,
  `the_days_follow_the_stale_projects_setting`, `rows_come_largest_first_and_page_with_the_totals`.
- `verdict`, in the same file: one test per row of the design's table, and `ignored_wins_over_holding_explained_items`
  (`Ignored::AtOrAbove` with `Withheld(HoldsExplained)` is `Drop`, `Ignored::Inside` with it is `Enter`).
- `judge_tests.rs` passes unchanged (Files' judgement did not move).

`crates/engine/src/own_tests.rs`:
- `mod negative`: `a_plain_folder_is_not_picked` (`NotOwnItem`).
- `mod positive`: `a_package_is_still_picked`. The tests that picked a folder through `largest_place` pick a file or
  a package instead.

## Checks

- `cargo nextest run -p fresher-core -p fresher-engine -p fresher-app`, `npm run sandbox:test`
- `cargo nextest run -p fresher-e2e` (`largest.rs`, `largest_home/`: a folder pick there becomes a refusal or a file
  pick; `fixtures/golden/largest-flow.json` rewritten with `UPDATE_GOLDEN=1` only if a folder pick was in it, the
  diff read before it is kept)
- `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`

## Docs

None yet (04/01).

Commit: `feat(engine): the folders of the home left alone, and no folder picks`.
