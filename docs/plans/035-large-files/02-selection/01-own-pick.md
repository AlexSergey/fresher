# 035/02/01 — The pick, the plan and the commands

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** engine, app, docs

**Deletion safety lives here: use the strongest model and review twice.** A row the user ticks on the Largest screen is
turned by the engine into a finding of the core's category `core.own-file` (High), held apart from the scan's
findings; the plan builder finds it like any other picked finding, and the cleaner, the journal and put back work on it
unchanged. Nothing is ever picked by the engine itself, nothing is pre-selected, and the cleaner is not changed. The
design: [`../00-design.md`](../00-design.md), sections 3 and 5.

Read first: `docs/restrictions/deletion-safety.md`, `docs/architecture/removal.md`, `crates/engine/src/results.rs`
(`findings`, `visible`, `finding`, `begin`, `keep_state`, `remove`, `next_id`), `crates/engine/src/cleanup/pending.rs`
(where the plan's findings are taken: `plan::expand(selection, &results.visible(), …)`),
`crates/engine/src/cleanup/operation.rs` (`end`: `results.remove(&removed, &cleared)`, `FindingsRemoved`),
`crates/core/src/finding.rs` (`NewFinding::new`, `Finding::new`), 01/01 (`own_file()`, `CommandError::NotOwnItem`) and
01/03 (`judge`, the `gone` parameter), `app/tauri/src/storage_commands.rs` and `app/tauri/build.rs`.

## Files

- `crates/engine/src/own.rs` (new) — `impl Engine { pub fn pick_own_item(&self, node: NodeId) -> Result<Finding,
  CommandError> }`:
  - refused `NoScan`, `Busy` (a scan or a cleanup running: the same rule as a plan's preparation), `StaleNode` for a
    node of another state, and `NotOwnItem` unless `judge` (01/03) answers `Offer::Offered` for the node now (the
    ignore lists and the gone paths as they are now);
  - the same node answers the same finding (looked up in `Results.own` by path);
  - otherwise the finding: `NewFinding::new(&own_file(), "own/<path from the home>", Subject::Path(path),
    node.sizes, Text { key: core.own-file.title, args: name }, Text { key: core.own-file.reason.file or .folder },
    Removal::MoveToTrash(path))`, `Finding::new(id, …, Removability::Removable)` (the rehearsal reads the disk and
    refuses what it must: design agent's call 7), the id the session's next free one (`Results::next_id`, bumped, so
    the next scan's ids still follow);
  - a pure helper builds the finding from plain values, so its fields are unit-tested without an engine.
- `crates/engine/src/results.rs`:
  - `own: Vec<Finding>` (id order) and `gone: Vec<PathBuf>`; `own()`, `own_by_path(&Path)`, `pick(finding)`;
  - `begin` and `keep_state` clear both (a new scan, a new state);
  - `finding(id)` looks in `own` too (`reveal_finding` then works on a picked row);
  - `remove(removed, cleared)` drops removed own findings as it drops the scan's and pushes each removed own finding's
    path to `gone`; it answers them among the dropped ids, so `FindingsRemoved` names them;
  - the overview's totals, `page`, `visible`, `explained_under`, the claims and the shares never read `own`.
- `crates/engine/src/cleanup/pending.rs` — the plan's findings are `results.visible()` followed by `results.own()`
  (the own ids are larger than every scan id, so the list stays in id order, as `plan::expand` needs). No change to
  `plan.rs`.
- `crates/engine/src/largest/page.rs`, `place.rs` (01/03) — pass `results.gone` in, and fill `picked` from
  `own_by_path`.
- `app/tauri/src/largest_commands.rs` (new) — `get_largest(engine, query: LargestQuery) -> Result<LargestPage,
  Refusal>`, `get_largest_place(engine, folder: Option<NodeId>, offset: u32, limit: u32) -> Result<OwnPlace,
  Refusal>`, `pick_own_item(engine, node: NodeId) -> Result<Finding, Refusal>`, each `#[tauri::command(async)]`, thin,
  as `storage_commands.rs`; their tests on `crate::testing::Shell`, negatives first (`no_scan_is_refused`,
  `a_stale_node_is_refused`), then one answer each.
- `app/tauri/src/storage_commands.rs` — `get_storage_place` and its test removed (design agent's call 11); the engine's
  `storage_place` stays.
- `app/tauri/build.rs` (`COMMANDS`), `app/tauri/src/lib.rs` (`generate_handler!`), `app/tauri/capabilities/default.json`
  (`allow-…`) — `get_largest`, `get_largest_place`, `pick_own_item` **in the place of `get_storage_place`** (so the
  rebase onto other plans' appended commands is mechanical). The test that every command is allowed passes.
- `app/ui/src/ipc/bindings.generated.ts` — regenerated (the new types reach the UI through the commands).
- `docs/restrictions/deletion-safety.md`, "Unknown is not junk" — a third case, written as the rule's: the Largest
  screen's rows are not claimed by any rule and never offered as junk; a row becomes an item only when the user ticks
  it, at High, never pre-selected, through the normal confirmation, with the "Enforced by" and the tests of this step.
- `docs/categories/core-own-file.md` (new) and its row in `docs/categories/README.md` — the category in the catalogue's
  format: id, danger High, the five texts, "made by: no module; the user picks each item on the Largest screen".

## Checks

Negatives first everywhere.

- `own.rs`: `mod negative` `nothing_is_picked_before_a_scan`, `nothing_is_picked_while_a_scan_runs`,
  `a_node_of_another_state_is_stale`, `a_claimed_node_cannot_be_picked`, `a_standard_folder_cannot_be_picked`,
  `the_library_cannot_be_picked`, `a_hidden_file_cannot_be_picked`, `an_ignored_place_cannot_be_picked`,
  `a_folder_holding_explained_bytes_cannot_be_picked`, `a_removed_row_cannot_be_picked_again`; `mod positive`
  `a_picked_file_is_an_own_file_finding_at_high`, `a_picked_folder_reads_as_a_folder`, `picking_twice_answers_one_finding`,
  `a_picked_id_follows_the_scan_s_ids`, `the_next_scan_s_ids_follow_the_picks`.
- `results.rs`: `mod negative` `a_picked_finding_is_in_no_total_page_or_share`, `a_new_scan_forgets_the_picks`,
  `a_new_state_forgets_the_gone_paths`; `mod positive` `a_removed_pick_is_dropped_and_its_path_is_gone`,
  `a_picked_finding_is_found_by_id`.
- The cleanup, through the engine's test harness with the fake cleaner (`crates/engine/src/cleanup/testing.rs`):
  `a_picked_row_goes_through_the_plan_at_high` (the plan view's highest danger is High and its item's category is
  `core.own-file`), `a_picked_row_inside_a_picked_folder_is_covered` (the existing `Covered` rule), `a_removed_pick_is_named_by_findings_removed`,
  `a_cleanup_of_a_pick_records_its_category_in_the_journal`.
- `cargo nextest run -p fresher-engine -p fresher-app`; clippy; fmt; the bindings test; `npm run check:crate-deps`.
- `git diff -- crates/cleaner crates/engine/src/cleanup/plan.rs` is empty.
- Commit: `feat(engine): a picked own file through the confirmed plan, and the largest commands`.

## Conclusions

- `Engine::pick_own_item(node)` holds the machine lock from its idle check to the end; it refuses `Busy`, `NoScan`,
  `StaleNode` and `NotOwnItem` unless the one judgement (`judge_reached`) says offered at that moment. The answer is a
  `core.own-file` finding at High with `Removal::MoveToTrash`, kept in `Results.own` and never in a total, a card, the
  map, the claims or a page.
- The plan builder adds the picks still offered (judged again at prepare); confirm refuses `PlanGone` when a pick is no
  longer offered. The home itself is never pickable. A removed pick leaves the lists until a rescan (`Results.gone` is
  kept across a cancelled scan, cleared with a new state).
- The commands `get_largest`, `get_largest_place` and `pick_own_item` stand where `get_storage_place` was, and
  `get_storage_place` is removed from the build, the handler and the capabilities. The bindings gained 11 types.
- Texts for a pick's title and reason are core helpers (`own_file_title`, `own_file_reason`): the engine may not build
  a `TextKey`. `deletion-safety.md` has a third case under "Unknown is not junk"; `docs/categories/core-own-file.md`
  is the category's page.
- Two reviews (one adversarial: every attack refused). 1 285 tests passed in engine, app and core at this step.
