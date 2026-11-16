# 004/04/03 — Folding, links and totals

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner

## Files

- `crates/scanner/src/tree/fold.rs` — `FolderBatch` (what a worker sends for one folder: the folder's index, its
  folded `Sums`, the kept entries with their names, kinds, sizes, ids and hits, the subfolders to queue) and
  `fold(listing, facts, hits, threshold) -> FolderBatch`: a file is kept when its allocated size is at or over the
  threshold (1 MiB, or 01's number), when its link count is above 1, when it has a clone id with a shared part, or when
  an interest hit it; every other file adds to the folder's count and sums. A link is kept (its own size only) when an
  interest hit it, otherwise folded as a zero-size entry.
- `crates/scanner/src/tree/apply.rs` — `Tree::apply(batch)`: appends the batch's nodes and hits under its folder; the
  only way the tree grows during a walk.
- `crates/scanner/src/tree/opaque.rs` — a subtree claimed `Whole`: `Tree::apply_opaque(folder, sums, folders, links)`
  stores its totals and flags it opaque; nothing below it becomes a node.
- `crates/scanner/src/tree/links.rs` — `LinkSet`: `(device, file_id) → first holder` for files with more than one
  link, filled by `apply`; at `freeze`, each group's sizes stay with the holder that comes first in depth-first name
  order and are removed from the others (which keep a zero-size record marked "a hard link of …"), so the totals do not
  depend on the order the workers ran in. Clones: `CloneSet` keyed by `(device, clone_id)`; the shared part
  (`allocated − reclaimable`) is counted once per group, at the first member in the same order; the private part per
  file.
- `crates/scanner/src/tree/arena.rs` — `freeze` runs the link and clone attribution before the bottom-up sums.

## Produces

`FolderBatch`, `fold` and `apply` for the walker (05/03); the deterministic totals the codec (06/02) and the queries
(08) rely on.

## Checks

- Negative: `a_small_file_is_not_kept`, `a_hard_link_group_is_counted_once`, `a_clone_groups_shared_part_is_counted_once`,
  `nothing_inside_an_opaque_folder_becomes_a_node`, `a_folders_own_link_count_is_not_a_hard_link`.
- Positive: `a_large_file_is_kept_with_its_three_sizes`, `a_matched_small_file_is_kept`,
  `link_bytes_go_to_the_first_path_in_name_order` (two build orders, same holder),
  `folded_sums_equal_the_files_they_replace`, `an_opaque_folder_keeps_its_totals`.
- Property: `folding_never_changes_a_total` (a random listing folded at any threshold gives the same folder total);
  `totals_do_not_depend_on_apply_order` (random batches applied in two orders freeze to equal trees).
- `cargo nextest run -p fresher-scanner`; coverage ≥ 90 %.

## Conclusions

- `crates/scanner/src/tree/`: `fold.rs` (`KEEP_THRESHOLD` = 1 MiB, `FolderBatch`, `Kept`, `SharedFile`, `fold`),
  `apply.rs` (`Tree::apply`, which also gives the node of each kept entry in batch order, so the walker can queue the
  folders), `opaque.rs` (`Tree::apply_opaque`), `links.rs` (`LinkSet`, `Resolved`); `freeze` now sorts, attributes the
  shared bytes, then sums.
- **Deviation — small hard links and clones are folded, not kept.** The step keeps a file whose link count is above
  1 or that has a clone id; the design (sections 4 and 5, agent's calls 2 and 21, measured in 01) folds them, since
  keeping them would more than double the nodes (1.26 M against 556 k on the measured home). A folded hard link or
  clone goes to the batch's `shared` list, counts as a file of its folder, and its bytes go to the link or clone
  table with the folder as its holder. A large or matched one is kept and its node is the holder.
- **Attribution at `freeze`**: the nodes get a depth-first rank (children sorted); each group's members are ordered by
  `(device, id, rank)` and the first holder gains the group's bytes. A hard link group's bytes (logical, private
  allocated, reclaimable) are counted once; a clone group's shared part (allocated − reclaimable) once — **the largest
  any member shares**, not the first member's, so a diverged clone first in order does not undercount; private parts
  per file. A kept file of a link group counted elsewhere has zero bytes and the flag `LinkCopy` ("a hard link of …").
  A folder holder ranks before its own children, so a folded member in a folder wins over a kept member below it;
  either way the folder's total is the same.
- `apply_opaque(folder, sums, folders, shared)` takes the subtree's shared files as its `links`; it refuses a folder
  that already has children (`TreeError::NotOpaque`). An unreadable place inside an opaque subtree is marked by the
  walker with `Flag::Incomplete` on the folder.
- A folded link counts as a file with no bytes; a kept link (matched) keeps its own sizes as the reader gave them. An
  entry with a read error counts for nothing and makes the batch `incomplete` (the folder becomes `Partial`, as the
  reference walk makes it). A folder's own link count is never read.
- The walker adds the folder's own (marker) hits to `FolderBatch::own_hits`; `apply` attaches them to the folder.
- Tests: the plan's checks plus an unreadable entry, an unmatched link, and the order of `apply`'s nodes; the
  properties run random listings with links and clones (`folding_never_changes_a_total`, also at thresholds 0 and
  `u64::MAX`) and random folder trees applied depth-first and breadth-first (`totals_do_not_depend_on_apply_order`).
  The scanner's lines are covered at 94–100 % a file.
