# 035/01/02 — The scanner's items query

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** core, scanner, testing, engine

One new method of the core's `DiskState` port, answered by the scanner in one pass over the committed tree: the largest
**items** below a node, as a rule the caller gives decides. The scanner knows nothing of homes, packages or names'
meanings: the engine's rule (01/03) does. The walk, the cache format and the tree's memory do not change. The design:
[`../00-design.md`](../00-design.md), sections 1 and 6.

Read first: `crates/core/src/disk_state.rs`, `crates/scanner/src/state/mod.rs` (`TreeState`, `owned`, `info`),
`crates/scanner/src/state/largest.rs` (`largest_files`, `Largest`, `PathOrder`: the model of a bounded "largest
first"), `crates/scanner/src/tree/node.rs` (`Flag`: `KeptFile`, `Link`, `Special`, `LinkCopy`, `Collapsed`, `Opaque`),
`crates/scanner/src/state/query_tests.rs` (`Built`, how trees are built in tests), `crates/testing/src/fixed_disk_state.rs`,
`crates/engine/src/disk_tests.rs` (`SlowState`, another implementor).

## Files

- `crates/core/src/disk_state.rs` — beside the trait:

  ```rust
  /// How [`DiskState::items`] treats one node below its root.
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum ItemVisit {
      /// Not answered and not entered.
      Skip,
      /// Answered as one item (when large enough) and not entered.
      Item,
      /// Entered; never answered itself.
      Enter,
  }

  /// The caller's rule of an items query: what a node is, by its name (lossy UTF-8), its kind and its depth below the
  /// query's root (a child of the root is 1).
  pub trait ItemRule {
      fn visit(&self, name: &str, kind: NodeKind, depth: u32) -> ItemVisit;
  }
  ```

  and the trait method:

  ```rust
  /// The items below `root` (never `root` itself), the largest allocated size first, ties in path order, at most
  /// `limit`: each node `rule` calls an item, of at least `min` allocated bytes. Never asks `rule` about, enters or
  /// answers a node a category claimed, a link, a special entry or a hard link counted at another path; never enters
  /// a folder smaller than `min` (nothing in it can reach it). `None` for a node of another state.
  fn items(&self, root: NodeId, rule: &dyn ItemRule, min: ByteSize, limit: u32) -> Option<Vec<NodeId>>;
  ```

  A kept file the rule calls `Enter` counts as `Item` (a file has nothing to enter). Re-export `ItemRule` and
  `ItemVisit` from `lib.rs`.
- `crates/scanner/src/state/items.rs` (new) — the scanner's answer, one depth-first pass from `root` with an explicit
  stack (no recursion), children in name order so ties fall in path order: skip a node with an owner, a `Link`,
  `Special` or `LinkCopy` node, and a folder whose allocated total is below `min`; ask the rule of every other node
  with its `NodeKind` (as `TreeState::info` maps flags: `Collapsed` is `NodeKind::Collapsed`); keep the `Item`s of at
  least `min` in `Largest` (reuse it, from `largest.rs`, with its cap raised for this query to
  `fresher_core::MAX_ITEMS`: give `Largest::new` the cap to use, or a second constructor); enter the `Enter`
  folders. The name passed to the rule is the node's name as `String::from_utf8_lossy` gives it (no allocation for a
  UTF-8 name).
- `crates/scanner/src/state/mod.rs` — `impl DiskState for TreeState`: `items` maps `root` to its index (`None` for
  another state's id) and calls `items.rs`.
- `crates/testing/src/fixed_disk_state.rs` — `FixedDiskState::items`, the same contract over its built nodes (a simple
  recursive walk is fine in the test double); its own tests, negatives first.
- `crates/engine/src/disk_tests.rs` — `SlowState::items` delegates as its other methods do.

## Produces

`DiskState::items`, `ItemRule`, `ItemVisit` for 01/03.

## Checks

- `crates/scanner/src/state/items_tests.rs` (or the file's `#[cfg(test)]` module, as the crate does), built with
  `Built`, a rule in the test that names its answers by name:
  - `mod negative`, first: `a_node_of_another_state_answers_none`, `the_root_itself_is_never_an_item`,
    `a_claimed_folder_is_neither_entered_nor_answered`, `a_claimed_file_is_not_answered`, `a_link_is_never_asked_about`,
    `a_special_entry_is_never_asked_about`, `a_hard_link_counted_elsewhere_is_not_answered`,
    `a_skipped_folder_is_not_entered`, `an_item_below_the_minimum_is_not_answered`,
    `a_folder_below_the_minimum_is_not_entered` (the rule is never asked about its children: a counting rule);
  - `mod positive`: `items_come_largest_first`, `ties_come_in_path_order`, `a_folder_item_counts_its_whole_size_and_is_not_entered`,
    `an_entered_folder_is_not_itself_an_item`, `a_file_the_rule_would_enter_is_an_item`, `the_depth_counts_from_the_root`,
    `the_limit_keeps_the_largest`, `a_collapsed_subtree_is_offered_to_the_rule_as_collapsed`.
- A property (proptest, as the crate's other state tests): on random trees, every answered node is below `root`, has no
  owner, is at least `min`, the answer is sorted, and no answered node is inside another answered node.
- `cargo nextest run -p fresher-scanner -p fresher-testing -p fresher-engine`; clippy; fmt.
- `git diff -- crates/scanner/src/walk crates/scanner/src/codec crates/scanner/src/rescan crates/scanner/src/tree` is
  empty.
- Commit: `feat(scanner): the items query over the committed tree`.

## Conclusions

- `DiskState::items(root, rule, min, limit)` is one depth-first pass with an explicit stack, children in name order,
  ranked by allocated size with ties in path order; implemented by `TreeState`, `FixedDiskState` and the engine's
  `SlowState` test double.
- Every node below the minimum is skipped before the rule is asked (the answer is the same; fewer calls). A root that
  is claimed, a link, a special entry or below the minimum answers an empty list.
- A collapsed subtree answered `Enter` counts as one item (the ruling of the review; the trait doc says so), and the
  cap is `MAX_ITEMS + 1` so the engine can tell the list was truncated.
- 1 096 tests passed in scanner, testing and engine at this step; `git diff` of the walk, codec, rescan and tree is
  empty.
