# 004/06/02 — The tree codec

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner

## Files

- `crates/scanner/src/codec/varint.rs` — unsigned LEB128 for `u64` and a zig-zag form for signed times; `read` never
  reads past its slice and fails on an over-long encoding.
- `crates/scanner/src/codec/header.rs` — the body's header: `TREE_FORMAT: u16` (1), the time the state was taken,
  for each root its canonical path, its volume id and its `JournalPosition` (absent for a volume with no journal),
  the avoid list, the interest fingerprint, the claims table (category ids as strings, referenced by index), the node
  and kept-file counts (so the arena is allocated once on load).
- `crates/scanner/src/codec/encode.rs` — `encode(tree, header) -> Vec<u8>`: a pre-order walk; per folder node: the
  name front-coded against the previous sibling (shared prefix length, then the rest), the flags, the child count,
  the folded count and three sums, the modification time as a delta from the parent's, the hits (interest index,
  marker check, sibling bits), the owner index; then its kept files (name, flags, three sizes, link and clone ids
  when present, hits). Totals are not written.
- `crates/scanner/src/codec/decode.rs` — `decode(bytes) -> Result<(Tree, Header), CodecError>`: rebuilds the arena
  from the pre-order and the child counts, checks every count against the bytes left, then `freeze` (totals, links,
  clones). Any inconsistency is a `CodecError`, never a panic and never a partial tree.
- `crates/scanner/src/lib.rs` — `mod codec;`. The codec is not wired into `Scanner` here (05 owns `scanner.rs` in
  the same wave): 07/01 makes `commit` save and the rescan load.

## Produces

The cache body the rescan (07) and the restore (07/04) load; the depth-first order plan 010 compares in one merge
pass.

## Checks

- Negative: `a_body_cut_anywhere_is_refused` (every prefix of an encoded `SMALL`-like tree),
  `a_child_count_larger_than_the_rest_is_refused`, `another_tree_format_is_refused`, `an_overlong_varint_is_refused`.
- Positive: `a_tree_survives_a_round_trip` (equal frozen trees, equal header), `front_coding_restores_every_name`,
  `an_encoded_tree_is_accepted_by_the_memory_cache` (through `MemoryScanCache` with the policy's cap).
- Property: `any_tree_round_trips` (random trees: folders, kept files, links, clones, hits, owners, unreadable and
  avoided nodes); `decode_never_panics_on_any_bytes`.
- **Bar**: `cache_bytes_per_folder_node_stay_under_24` on an in-memory tree of 100 000 folder nodes with the name
  shapes of `TreeSpec::SMALL` (the helper of 04/01's heap test). The timings on `LARGE` (`cache/encode`,
  `cache/decode`) and the compression check of design call 8 come in 07/01, once the scanner saves for real.

## Conclusions

- `crates/scanner/src/codec/`: `varint.rs` (LEB128 and zig-zag; `Reader` checks every read, refuses a number with
  more bytes than it needs and a count the bytes left cannot hold), `header.rs` (`Header`, `RootRecord`,
  `TREE_FORMAT` = 1), `encode.rs` (`encode`, and `Layout`, which step 03 reuses), `decode.rs` (`decode`), `mod.rs`
  (`CodecError`); test helpers in `testing.rs`. Crate-private, `#[expect(dead_code)]` until 07/01 wires it.
- **The body**, after the header (format as two fixed bytes, the time taken, each root's path, volume id and journal
  position or none, the avoid list, the fingerprint as eight fixed bytes, the claims table): the number of roots and
  of nodes, then every node depth-first with children in name order, then the hard link groups and the clone groups.
  A node: its name front-coded against the previous sibling, its flags, for a folder its child count and its folders
  not kept as nodes, its own file count and three sizes, its modification time as a zig-zag delta from its parent's,
  its hits, its owner (0 for none, else index + 1) and, for a root, its folder identity. A group: device and id as
  deltas from the previous group's, the bytes it counts once, its member count and the depth-first positions of the
  members' nodes as deltas. A decoded node's arena index is its depth-first position.
- **Measured**: **22.1 bytes a folder node** (2 205 987 bytes for 100 000 folders whose names are 4 to 20 bytes plus
  a unique number, eight children a folder; bar: under 24). There is no `TreeSpec::SMALL`: the helper `wide_tree`
  takes the heap test's name shapes and appends a number, since siblings must differ. The timings on `LARGE` stay
  with 07/01.
- **The own bytes are derived** (a frozen node holds one set of sums, 04/01): total minus the children's totals minus
  the group bytes `freeze` credited to the node; folders not kept as nodes likewise. The decoder adds the groups
  again and `freeze` credits them to the same first member in depth-first order, so the totals equal. `encode`
  refuses a tree that is not frozen (`CodecError::NotFrozen`) and an owner outside the claims table.
- **Deviation — the link and clone tables are written, as groups.** The step lists link and clone ids on kept files
  only; since 04/03 a small hard-linked or cloned file is folded and its folder holds the group's member, so the
  groups are written whole (the rescan needs them to keep counting each group once). A group stores the bytes it
  counts once (its members' largest), not each member's; the totals are the same.
- **Deviation — kept files are not written after their folder's fields** but as nodes in the same depth-first list,
  in name order with the folders: one record layout, one front-coding run per folder, and the order plan 010 merges.
  The header holds the node count, not a separate kept-file count: the arena is allocated once from it.
- **Defence in depth**: a decoded name must be one plain name (not empty, `.`, `..`, no `/` or NUL), after its
  previous sibling in byte order; a root an absolute path; flags known bits, one kind, the folder flags on a folder,
  `Collapsed` with no children; every owner in the table; every member's node in the tree; no byte after the end.
- Tree accessors added (crate-private, for the codec): `Tree::with_capacity`, `is_frozen`, `set_own`, `set_flags`,
  `set_owner`, `shared(kind)`; `LinkSet::members` (`Member` made crate-visible); `NodeFlags::bits` and
  `from_bits`; `tree::testing` made crate-visible for the codec's tests.
- Tests: the step's checks, plus the header cut anywhere, a handmade body, bad names, siblings out of order, a
  folder flag on a file, an unknown owner, trailing bytes, an unfrozen tree, and the property
  `decode_never_panics_on_a_damaged_body`; a second write of a decoded tree gives the same bytes.
