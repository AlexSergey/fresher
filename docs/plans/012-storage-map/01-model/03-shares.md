# 012/01/03 — The shares

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** engine

## Files

- `crates/engine/src/storage/mod.rs` (new) — the module's doc; `mod shares;`.
- `crates/engine/src/storage/shares.rs` (new) — `Shares`, built by `Shares::of(state: &dyn DiskState, ignored:
  &IgnoreLists) -> Shares` (pure; `IgnoreLists` is the engine's existing view of `core.ignore.*`, or the two lists as
  plain arguments if no such type exists):
  - the **counted owners**: the owned nodes whose category is not ignored and whose path is not at or below an
    ignored path;
  - **per category**: the sum of `node.sizes − the sizes of its nearest counted owners inside`, the deepest owning its
    subtree — the rule of `claims::resolve`, on the tree's sizes;
  - **per folder**: `explained(id) -> Sizes`, the sum of the top-most counted owners at or below `id`, kept in a map
    from each ancestor of an owner to its sum (owners × depth entries, never one per node);
  - **per root**: `explained` and `unclaimed = root.sizes − explained` (completeness kept: a `Partial` root gives
    partial parts).
  Sizes are added with `checked_add`; a sum that overflows saturates at `u64::MAX` and is logged once (never a panic).
  **Corrected 2026-11-17 after review:** the nearest owners inside are subtracted over every owned node, counted or
  not, so an ignored owner inside a counted one keeps its bytes as unclaimed ("your folders"); and
  `explained(state, id)` of a node strictly inside a counted owner is the node's whole size less the uncounted owners
  inside it, so `node.sizes − explained(node)` never counts a byte twice.
- `crates/engine/src/lib.rs` — `mod storage;`.

## Produces

`Shares` for the by-meaning groups (04), the volume parts (02/04) and the place view (03/01).

## Checks

- Negative first: `an_ignored_category_is_unclaimed`, `an_owner_below_an_ignored_path_is_unclaimed`,
  `a_nested_owner_is_not_counted_twice` (an owner inside an owner: the outer category counts its sizes minus the
  inner's), `two_owners_side_by_side_do_not_share_bytes`, `an_overflowing_sum_saturates`.
- Positive: `explained_plus_unclaimed_is_the_root` on every fixture of `fixtures/homes/`;
  `a_folder_knows_its_explained_bytes`; `partial_roots_give_partial_parts`; a **property test** (`proptest`): for a
  random tree with random owners and random ignore lists, (a) the categories' shares add up to the roots' explained,
  (b) explained plus unclaimed is each root's size, (c) `explained(folder)` is never above the folder's size.
- `cargo nextest run -p fresher-engine`.

## Conclusions

Built in `1efde8d`, fixed in `53d6cee` after review (two Important findings, both about the step's own text).

- `Shares::of(state, ignored)` puts every owned byte in one place: the owner's bytes less the nearest owner inside.
  An ignored inner owner is subtracted from the outer one and its bytes go to your folders (the same rule as the
  engine's claims); the step's "counted" wording was wrong.
- `explained` takes the state and the node, `explained(state, id)`: inside a counted owner a folder is wholly
  explained, otherwise it is what the owners below it keep. The one-argument form read 0 for such a folder.
- Measured: the property test checks every category and folder against an oracle (the deepest owner of each byte) and
  fails when inner owners are not subtracted. The budget on the generated LARGE tree is in 03/02's conclusions.
