# 044/02/01 — `DiskState::left_alone`

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** core, scanner, testing, engine (test
doubles only)

The state answers the highest folders below a root that nothing changed since a time. The design:
[`../00-design.md`](../00-design.md), section 3; agent's call 5.

## Read first

Plan 01's commits; `crates/core/src/disk_state.rs` (`DiskState::items`, `ItemRule`, `ItemVisit`);
`crates/scanner/src/state/{mod.rs,items.rs,items_tests.rs,largest.rs}` (`offered`, `visit`, `Largest`,
`PathOrder`); `crates/testing/src/fixed_disk_state.rs` (`items`, `collect_items`, its tests);
`crates/engine/src/disk_tests.rs` (`SlowState`) and `crates/engine/src/largest/largest_tests.rs` (`Counting`).

## Files

- `crates/core/src/disk_state.rs` — `pub struct LeftAlone { pub folder: NodeId, pub newest: Timestamp }` (`Clone,
  Copy, Debug, PartialEq, Eq`) and `DiskState::left_alone(&self, root: NodeId, rule: &dyn ItemRule, min: ByteSize,
  before: Timestamp) -> Option<Vec<LeftAlone>>`, with the design's doc comment (section 3) word for word.
- `crates/core/src/lib.rs` — exports `LeftAlone`.
- `crates/scanner/src/state/left_alone.rs` (new) — `pub(super) fn left_alone(tree: &Tree, root: NodeIndex, rule: &dyn
  ItemRule, min: u64, before: i64) -> Vec<(NodeIndex, i64)>`: depth first with an explicit stack, children by name;
  a child is looked at when `items.rs`'s `offered(node, min)` holds, it is a folder (`Flag::Folder`, not `Collapsed`)
  and `rule.visit(..)` answers `Enter`; a folder looked at is answered when it is not `Partial`, its `newest` is not
  `NO_TIME` and it is older than `before`, else entered. Ranked by `Largest` (allocated, then path order) with no cap
  (`Largest::new(u32::MAX)` or an uncapped constructor added beside `capped`). `offered` becomes `pub(super)` in
  `items.rs`.
- `crates/scanner/src/state/mod.rs` — `mod left_alone;` and `TreeState::left_alone` (`None` for a root of another
  state; ids and `Timestamp`s out).
- `crates/testing/src/fixed_disk_state.rs` — `Node.newest: Option<Timestamp>`, `with_newest(path, newest)`, and
  `left_alone` (recursive as `collect_items`, sizes and newest as recorded, `Completeness::Partial` unknown).
- `crates/engine/src/disk_tests.rs`, `crates/engine/src/largest/largest_tests.rs` — `SlowState` and `Counting`
  delegate `left_alone` to their inner state.

## Interfaces produced

- `fresher_core::LeftAlone`, `DiskState::left_alone`; `FixedDiskState::with_newest`. 02/03 calls them with
  `HomeItems`.

## Tests (negatives first)

`crates/scanner/src/state/left_alone_tests.rs` (new, `#[path]` from `left_alone.rs`), over trees built with
`tree/testing.rs`:
- `mod negative`: `a_node_of_another_state_answers_none` (in `state/tests.rs`, beside the items' one),
  `the_root_itself_is_never_answered`, `a_folder_below_the_minimum_is_neither_answered_nor_entered`,
  `a_folder_with_a_recent_file_is_not_answered`, `a_partial_folder_is_not_answered`,
  `a_folder_without_a_newest_is_not_answered`, `a_claimed_folder_is_neither_answered_nor_entered`,
  `a_file_a_link_or_a_collapsed_subtree_is_never_answered`, `what_the_rule_skips_or_calls_an_item_is_not_answered`,
  `a_folder_inside_an_answered_one_is_not_answered`.
- `mod positive`: `a_stale_folder_is_answered_with_its_newest`,
  `the_stale_children_of_a_recent_folder_are_answered`, `answers_come_largest_first_ties_in_path_order`.

`crates/testing/src/fixed_disk_state.rs`:
- `mod negative`: `left_alone_of_a_node_of_another_state_answers_none`,
  `left_alone_skips_a_claimed_a_partial_or_a_recent_folder`.
- `mod positive`: `left_alone_answers_the_highest_stale_folder_of_each_branch`.

## Checks

- `cargo nextest run -p fresher-scanner -p fresher-testing -p fresher-core`
- `cargo nextest run -p fresher-engine` (the doubles compile)
- `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`

## Docs

None yet (04/01).

Commit: `feat(scanner): the folders left alone below a node`.
