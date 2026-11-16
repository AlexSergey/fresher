# 004/07/01 — Save, load and the change set

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner

## Files

- `crates/scanner/src/scanner.rs` — `Scanner` takes an `Arc<dyn ScanCache>` and an `Arc<dyn ChangeJournal>`.
  `walk` notes each root volume's `ChangeJournal::position` before the first read (so a change during the walk is
  seen by the next rescan); `commit` encodes the state with `codec::fit` under the cache's cap and saves it under the
  roots' scope key. A save error is logged and does not fail the commit: the cache is safe to lose.
- `crates/scanner/src/state/mod.rs` — `TreeState` reports a collapsed node as `NodeKind::Collapsed` with no children.
- `crates/scanner/src/rescan/load.rs` — `Cached::load(cache, scope, request, matcher) -> Result<Cached,
  FullScanReason>`: no file → `NoCache`; a codec error → `BrokenCache`; another fingerprint → `OtherInterests`; other
  roots → `OtherRoots`; another avoid list → `OtherAvoidList`; a root volume whose id differs → `OtherVolume`; a
  volume with no journal position → `NoJournal`.
- `crates/scanner/src/rescan/change_set.rs` — `ChangeSet::gather(cached, journal, cancel) -> Result<ChangeSet,
  FullScanReason>`: `changes_since` per volume; a `JournalGap` → `JournalGap(kind)`; each changed folder placed with
  `Tree::lookup` (a path under no root is ignored; a path not in the tree goes to its nearest ancestor that is);
  `subtrees` and changes inside an opaque or collapsed node mark that node for a full re-walk; every unreadable and
  every other-volume node joins the set (design call 26); more folders than `RESCAN_LIMIT` → `TooManyChanges`.
  `RESCAN_LIMIT` is a share of the tree's folders, set by the bench of 07/03 (start: 20 %).
- `crates/scanner/benches/scan.rs` — groups `cache/encode`, `cache/decode`, `cache/fit` on `LARGE` (the numbers 06
  left to this step), with the compression check of design call 8 recorded in the notes.

## Produces

A loaded, validated tree and the set of nodes to read again, for the patch (02).

## Checks

- Negative, one per reason: `no_cache_means_a_full_scan`, `a_broken_cache_means_a_full_scan`,
  `other_interests_mean_a_full_scan`, `other_roots_mean_a_full_scan`, `another_avoid_list_means_a_full_scan`,
  `another_volume_means_a_full_scan`, `a_journal_gap_means_a_full_scan` (each `JournalGap` of the fake),
  `too_many_changes_mean_a_full_scan`, `a_change_outside_the_roots_is_ignored`.
- Positive: `a_changed_folder_is_placed_on_its_node`, `a_new_folder_is_placed_on_its_nearest_ancestor`,
  `a_change_inside_a_collapsed_subtree_marks_the_collapsed_node`, `unreadable_folders_always_join_the_set`,
  `the_position_is_taken_before_the_walk_reads` (the fake journal records the call order),
  `a_committed_scan_is_saved_and_loads_back`.
- `cargo nextest run -p fresher-scanner`; the cache numbers on `LARGE` in the notes (file size against the 8 MiB
  cap and the design's bar).

## Conclusions

- `Scanner::new` takes the journal and the cache after the volumes (`new(reader, volumes, journal, cache, anchor,
  options)`); `ScannerOptions::cache_body_limit` carries the store's body limit, which the app passes from
  `CachePolicy::DEFAULT` where it composes `FileScanCache` (on the data folder) and `MacChangeJournal` (its replay
  limited to 11 s: half the developer's warm full scan of 01). The tests, the benches and the e2e use
  `MemoryScanCache`, `FileScanCache` on a temporary folder and `FakeChangeJournal`.
- `walk` prepares the roots, the boundary, the matcher and the scope (`Scanner::prepare`: no folder read), then asks
  `ChangeJournal::position` once per volume before the first read. `commit` clears the owners, records the claims,
  and saves the state through `fit` within the cache's cap and the body limit; a body that does not fit or a cache
  that refuses it is logged and not saved.
- `rescan/load.rs`: `Cached::load(cache, scope, Use::Rescan | Use::Restore)`. The scope (`rescan::Scope`) is the
  scope key, the canonical roots with their volumes and identities, the avoid list (sorted, the request's and the
  standing one) and the matcher's fingerprint. Reasons, in order: `NoCache`, `BrokenCache` (a store error or a codec
  error, logged), `OtherRoots`, `OtherVolume`, `OtherAvoidList`, `OtherFingerprint` (the core's name for the step's
  "other interests"), then for a rescan only `JournalGap(NoJournal)` (a root's volume had no position) and
  `JournalGap(RootChanged)` (a root's identity is not the cached one: the journal reports a moved root only on its
  parent, so the identity is compared on every rescan).
- `rescan/change_set.rs`: `ChangeSet::gather(cached, Gather { journal, now, sibling_markers, cancel })` gives two
  sets of cached nodes: `reread` (folders whose own entries are read again) and `rewalk` (nodes walked again from
  scratch). A position older than 14 days is `JournalGap(TooOld)` before the journal is asked (design call 1).
  **Changed from the step**: the parent of every folder the journal names is read too. The tree keeps no file id and
  no folder time but its parent's listing does: reading the parent gives the folder's new modification time (a
  rule's age check reads it), its identity (a folder replaced under the same name is walked anew) and its sibling
  names. Unreadable and other-volume nodes join `rewalk` without their parent unless a marker interest has sibling
  names. `RESCAN_LIMIT_PERCENT` (20) compares the folders read again plus every folder below a node walked again
  with the tree's folders.
- `TreeState` already reported `Collapsed` (06); a test now holds it on a decoded collapsed tree.
- **Cache numbers on `LARGE`** (release, the developer's machine, `cargo bench -- cache`). With the interests of the
  scan bench (node folders, the tag, `Library/Caches/*`, `*.tmp`): 587 637 nodes, a body of 19.5 MB (33 B a node)
  and 11.0 MB compressed — over the 8 MiB cap, and nothing can be collapsed, because one generated file in five is a
  matched `.tmp` file kept in nearly every folder: `fit` refuses it and nothing is saved (every scan is then full,
  never wrong). The cache benches therefore use the same interests without the extension: 114 478 nodes, a body of
  2 765 212 B (24 B a node, under the 32 B bar), 1 725 398 B compressed (−38 %; the generated sizes compress worse
  than a home's −60 %), well under the 8 MiB cap with nothing collapsed; `cache/encode` 15 ms, `cache/decode` 10 ms,
  `cache/fit` 51 ms (encode and one compression). Peak memory of the bench process 226 MB.
- For the user: an interest that matches a large share of all files keeps every one of them as a node and makes
  the tree uncollapsible; such a home is never cached. No module of today does that.
