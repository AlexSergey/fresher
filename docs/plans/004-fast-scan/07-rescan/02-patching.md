# 004/07/02 — Patching the tree

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner

## Files

- `crates/scanner/src/rescan/patch.rs` — `patch(tree, change_set, reader, matcher, boundary, threads, cancel) ->
  Result<Tree, WalkError>`:
  1. each marked folder is read again by the same workers and the same `fold` as a full scan (05/03): its own sums,
     kept files and hits are replaced; a subfolder that is new is queued for a full walk; a subfolder that is gone is
     dropped with its subtree; a subfolder whose identity changed (same name, another file id) is dropped and walked
     anew;
  2. a node marked for a full re-walk (a journal subtree, an opaque or collapsed node) has its subtree dropped and
     walked again from it;
  3. the result is a new arena built by copying the unchanged nodes in pre-order and inserting the re-read ones, so
     no hole stays in memory; then `freeze` recomputes the totals, the links and the clones.
- `crates/scanner/src/rescan/emit.rs` — `emit(tree, on_match)`: every hit of the patched tree, in pre-order, as the
  `Match` a full scan would have sent (a `Whole` claim with its subtree's totals, the marker check, the siblings).
- `crates/scanner/src/walk/worker.rs` — the job gains `replace: Option<NodeIndex>` so the patch reuses the worker
  loop unchanged.
- `crates/scanner/src/scanner.rs` — `WalkMode::Auto`: load, gather, patch, emit, `ScanKind::Rescan {
  changed_folders }`; on any `FullScanReason`, a full scan with `fallback` set.

## Produces

The rescan that 03 checks and 04 exposes.

## Checks

- Negative: `a_removed_subtree_leaves_no_node_and_no_bytes`, `a_folder_replaced_under_the_same_name_is_walked_anew`,
  `an_unchanged_folder_is_not_read` (counting reader), `a_cancelled_patch_returns_cancelled_and_commits_nothing`.
- Positive: `a_new_file_is_counted_after_a_rescan`, `a_file_grown_in_place_is_counted_after_a_rescan`,
  `a_new_subtree_is_walked`, `a_change_inside_a_collapsed_subtree_rereads_it`,
  `matches_after_a_rescan_equal_those_of_a_full_scan`.
- Property: `a_rescan_equals_a_full_scan` — on random trees with a random script of changes (create, remove, grow,
  rename, new and removed subtrees) applied to the fixture and reported exactly by `FakeChangeJournal`, the patched
  tree equals the tree of a full scan of the changed fixture, node for node.
- `cargo nextest run -p fresher-scanner`; coverage ≥ 90 %.

## Conclusions

- `rescan/patch.rs`: `patch(old, set, walk, cancel, on_progress) -> Result<Patched, WalkError>` in three passes. (1)
  The folders of `reread` are read by the workers with the full scan's `read` and `fold` (`Walk::read_all`: the
  worker loop unchanged, one report a job, nothing queued below). (2) A new arena is built by copying the cached tree
  in pre-order: a copied node keeps its own bytes (its total minus its children's and minus the group bytes `freeze`
  credited it, `rescan/frozen.rs`), its hidden folders, its time, its hits and its group members; owners are not
  copied (the claims are committed again). A folder read again gets its listing through `Tree::apply`; each of its
  subfolders still listed is copied from the cached node of the same name, unless its identity is not the one it
  was read with (replaced: walked anew); a new one is queued; a gone one is never copied. A node of `rewalk` gets
  its name hits and nothing below it, and is queued; an unreadable or other-volume node is identified and decided by
  the boundary again (still on another volume: marked, not read). (3) The queued folders go through the full walk's
  aggregator into the new arena (`Walk::finish`, split out of `Walk::run`), which freezes it. Nothing dropped is
  copied, so no hole stays.
- **Changed from the step**: the job gains no `replace` field. Reading the changed folders first and applying their
  reports while the arena is built reuses the worker loop as it is; only `Walk::run` was split into `run` +
  `finish`, and `Walk::read_all` added.
- Hits: a folder's name hits come from its parent's listing, its marker hits from its own. A node reached through a
  parent read again keeps only its marker hits, with the parent's new sibling names; a folder read again without its
  parent (a parent of a changed folder) keeps the sibling names of its cached marker hits — its parent's listing did
  not change. This was found by the property test.
- `rescan/emit.rs`: `emit(tree, matcher, interests, on_match)` sends every hit in pre-order as the full scan does: a
  folder's hit with no size, a whole claim of an opaque folder with the sum of its files (partial when it is
  `Incomplete`), a kept file with its sizes as read (its own bytes plus its group members), a link with none; a whole
  claim on a folder that is not opaque is not sent.
- `rescan/summary.rs`: the summary of a rescan is rebuilt from the tree: entries (folded files and kept entries of
  every folder read outside an opaque one), unreadable folders by kind, not-scanned places. An entry error counts
  once per folder (the tree keeps only `Incomplete`).
- `WalkMode::Auto`: load, gather, patch, emit, `ScanKind::Rescan { changed_folders }` (the nodes of the change set);
  any `FullScanReason` runs a full scan with `fallback` set; a cancel while the journal is read is `Cancelled`.
- Tests: the step's checks, each comparing the rescan with a full scan of the changed disk (the tree node for node,
  times and groups included, the matches and the counts). The property test `a_rescan_equals_a_full_scan` runs 64
  random homes with 1 to 4 random changes (create, remove, grow, move, new and removed subtrees; removal moves the
  entry out of the tree, the tests never delete); about three in four rescan, the others pass the limit and scan in
  full. Coverage is held by `npm run pre-push` (see 07/04).
