# 004/04/01 — The arena

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner

## Files

- `crates/scanner/Cargo.toml` — dependencies `fresher-core`, `thiserror`, `tracing`; dev-dependencies
  `fresher-testing`, `proptest`.
- `crates/scanner/src/tree/node.rs` — `Node` (a plain struct of about 96 bytes): `name: NameRef`
  (`u32` offset, `u16` length), `parent`, `first_child`, `next_sibling` (`NodeIndex(u32)`, `NONE = u32::MAX`),
  `flags: NodeFlags` (a `u16` of named bits: folder, kept file, link, opaque, collapsed, avoided, other volume,
  unreadable kind), `own: Sums` (the folded files' count and three sizes), `total: Sums` (the subtree's, filled by
  `freeze`), `folders: u32` (folders below), `modified: i64` (seconds, `i64::MIN` for none), `hits: HitRange`,
  `owner: u32` (an index into the claims table, or none).
- `crates/scanner/src/tree/names.rs` — `Names`: one `Vec<u8>` of every name; `push(&[u8]) -> NameRef`;
  `get(NameRef) -> &[u8]`. A name over `u16::MAX` bytes cannot exist on APFS and is refused as an error.
- `crates/scanner/src/tree/arena.rs` — `Tree { nodes: Vec<Node>, names: Names, hits: Vec<Hit>, roots: Vec<NodeIndex> }`:
  - `add_root(name, identity)`, `add_child(parent, name, kind) -> NodeIndex` (prepends to the parent's list: O(1));
  - `freeze(&mut self)`: sorts every folder's children by name bytes (a stable order on any thread schedule), then
    computes `total` and `folders` bottom-up in one pass over a post-order; `Partial` completeness rises from an
    unreadable node to every ancestor;
  - `path(index) -> PathBuf` rebuilt from the parents (bytes, `OsStr::from_bytes`);
  - `lookup(path) -> Option<NodeIndex>`: descends from the root that contains the path, comparing names; used by the
    rescan (07) to place a changed folder;
  - `heap_bytes() -> usize`: the capacity of every buffer, exact.
- `crates/scanner/src/tree/mod.rs`, `crates/scanner/src/lib.rs` — `mod tree;` (crate-private; the crate's public API
  is the `Scanner` of 05).

## Produces

`Tree` for the folding (03), the aggregator (05/03), the codec (06/02), the rescan (07) and the disk state (05/05, 08).

## Checks

- Negative: `a_name_too_long_is_refused`, `lookup_of_a_path_outside_every_root_is_none`,
  `lookup_does_not_match_a_prefix_of_a_name` (`Lib` is not `Library`), `an_unreadable_node_makes_every_ancestor_partial`.
- Positive: `freeze_sorts_children_by_name`, `freeze_sums_every_subtree`, `path_rebuilds_the_full_path`,
  `children_added_in_any_order_freeze_to_the_same_tree` (two build orders, equal trees).
- Property: `totals_equal_the_sum_of_the_leaves` (random trees), `freeze_is_idempotent`.
- **Bar**: `heap_bytes_per_node_stays_under_96` on a tree of 100 000 nodes with realistic name lengths (12 bytes on
  average), so the 3-million-entry estimate of the design holds. `cargo nextest run -p fresher-scanner`; coverage ≥
  90 %.

## Conclusions

- `crates/scanner/src/tree/`: `node.rs` (`NodeIndex`, `Sums`, `Flag`/`NodeFlags`, `Hit`, `Node`), `names.rs`
  (`Names`, `NameRef`), `arena.rs` (`Tree`, `Root`, `Children`), `mod.rs` (`TreeError`), a test helper `testing.rs`
  (`canonical`: every node depth-first without its index, so trees built in different orders compare equal).
  Crate-private; `mod tree` carries `#[expect(dead_code)]` until the walker (05) uses it.
- **Measured**: a `Node` is **72 bytes**; the tree of 100 000 nodes with names of 12 bytes on average holds
  **83 heap bytes a node** after `freeze` (bar: under 96).
- **Deviation — one set of sums a node, not `own` and `total`.** Both, with the plan's other fields, made a node
  104 bytes (over 115 bytes a node with names), past the bar. The node holds its own bytes (a folder's folded files, a
  kept file's sizes) until `freeze` and its subtree's after; `freeze` is a one-way step (a frozen tree refuses every
  change with `TreeError::Frozen`, a second `freeze` does nothing). A folder's own bytes can be derived after it
  (its total minus its children's), which the codec (06/02) and the rescan (07) can use. `folders` follows the same
  rule: before `freeze` it holds only the folders not kept as nodes (`add_hidden_folders`, for an opaque folder).
- `Sums` is a `u32` file count and three `u64` sizes; the node stores them flat (no padding). A kept file or link
  node counts as one file, so a folder's file count is a plain sum. Completeness is a flag: `Partial` is set by
  `freeze` on every node that was not read in full (unreadable, avoided, an entry of it unreadable, a sum that
  overflowed) and on all its ancestors.
- `freeze` sums in reverse arena order instead of a separate post-order: a child is always added after its parent,
  so the reverse is a bottom-up order, one pass with no stack. It also shrinks every buffer to its length (the bar
  counts capacity).
- `add_root` takes the root's `FolderIdentity` (kept in `roots: Vec<Root>`), for the walker's and the rescan's
  identity checks. The node's flags also carry `Incomplete` (an entry of the folder could not be read) and `Partial`.
- Hits of a node lie side by side in `hits`; `add_hits` on a node that already has hits copies them to a fresh run at
  the end (a marker hit found after the folder's name hit; rare).
- Dependencies: `fresher-core`, `thiserror`; dev `proptest`. `tracing` and `fresher-testing` are added by step 02,
  which first uses them (`cargo machete` refuses an unused one).
- Tests: the plan's checks plus a frozen tree refusing changes, an avoided or incomplete folder and an overflow being
  `Partial`, hits staying with their node, and the property `build_order_does_not_change_the_frozen_tree`.
