# 012/01/02 — Owned nodes and lookup

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
scanner, testing, engine

## Files

- `crates/core/src/disk_state.rs` — two methods of `DiskState`:
  - `fn owned(&self) -> Vec<(NodeId, CategoryId)>`: every node with an owner, in depth-first order (an ancestor before
    its descendants);
  - `fn find(&self, path: &Path) -> Option<NodeId>`: the node at `path`, compared by components; `None` outside every
    root or for no node.
- `crates/scanner/src/state/mod.rs` — `TreeState` answers both: `owned` is one pass over the arena's owner indices;
  `find` is the tree's existing `lookup`.
- `crates/testing/src/fixed_disk_state.rs` — `FixedDiskState` answers both from its list; a builder method
  `owned_by(path, category)` sets an owner (today the owner is always `None`).
- `crates/engine/src/disk_tests.rs` — `SlowState` (a test's state) delegates both.

## Produces

What the shares (03) and the attribution (04) read: the owners without walking the whole tree, and the app data folders
by path.

## Checks

- Negative: `a_state_without_claims_owns_nothing`, `find_outside_every_root_is_none`,
  `find_does_not_match_a_prefix_of_a_name` (`/a/bc` for `/a/b`), `a_claim_on_no_node_is_not_owned` (as the existing
  `a_claim_on_a_path_that_is_no_node_is_left_out`).
- Positive: `owned_lists_every_claimed_node_ancestors_first` (nested claims of `fixtures/homes/scan-nested-claims.toml`),
  `find_answers_the_node_of_every_path` (every node of a fixture tree found by its own `path`), the same two on
  `FixedDiskState`.
- `cargo nextest run -p fresher-scanner -p fresher-testing -p fresher-engine`.

## Conclusions

Built in `b72657e`, reviewed clean.

- `DiskState::owned()` (an ancestor always before its descendants) and `DiskState::find(path)` (by components, so
  `a/bc` is not `a/b`) in the core, the scanner's `TreeState`, `FixedDiskState` and the engine's slow state.
- Deviation: the step asked for a new `FixedDiskState::owned_by`; `with_owner` already did the job and was reused.
- Measured: `cargo nextest run -p fresher-scanner -p fresher-testing -p fresher-engine` 620 passed, 1 skipped.
