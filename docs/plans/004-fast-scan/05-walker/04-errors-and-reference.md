# 004/05/04 — Errors, completeness and the reference

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner

## Files

- `crates/scanner/src/walk/errors.rs` — a folder that fails to read becomes a node flagged with its `NotScanned`
  reason (`Unreadable(NoPermission | Guarded | Failed)`); `Interrupted` is retried up to three times then
  counted `Failed`; `Vanished` (removed during the walk) is counted `Failed` but leaves no node; `Replaced` is
  `Failed` and its node is kept without children. A per-entry error marks that entry's parent `Partial`. The counts go
  into `WalkSummary::unreadable` and `not_scanned`; no path is logged.
- `crates/scanner/tests/reference.rs` — the scanner with `StdReader` against `NaiveWalk`:
  - every fixture home under `fixtures/homes/` with a fixed interest set covering every `Pattern` and both claims;
  - matches compared as sorted lists (the scanner's order is not the reference's): path, kind, interest, sizes,
    modified, siblings, marker;
  - `WalkSummary` entries and unreadable counts equal.
- `crates/scanner/tests/macos_reader.rs` — the same comparison with `MacFolderReader` (`fresher-macos`
  dev-dependency), plus a clone fixture (`clone_file`, `fixtures` feature) checked against the tree's clone
  accounting.

## Produces

The proof that the fast walk means the same as the reference walk; the error semantics 006 reads.

## Checks

- Negative: `an_unreadable_folder_is_reported_by_kind_and_makes_its_parents_partial` (mode `000` in a fixture),
  `a_folder_removed_during_the_walk_is_not_an_error_of_the_walk` (a reader wrapper that removes nothing but answers
  `Vanished` for one path), `a_link_to_a_folder_is_listed_and_not_followed`, `a_link_out_of_the_root_is_not_followed`.
- Positive: `the_scanner_and_the_reference_agree_on_every_fixture_home`,
  `the_macos_reader_and_the_std_reader_build_the_same_tree` (on `SMALL`).
- Property: `the_scanner_and_the_reference_agree_on_random_trees` (proptest generates a fixture description: folders,
  files with sizes, links, hard links, a few unreadable folders; and a random interest set).
- `cargo nextest run -p fresher-scanner`; coverage of the crate ≥ 90 %.

## Conclusions

- `walk/errors.rs` (written in 03, checked here): an interrupted read is tried again three times, then `Failed`;
  `EACCES` is `NoPermission`, `EPERM` `Guarded`, a vanished, replaced or still-interrupted folder `Failed`. The folder's
  node keeps its kind and makes its ancestors `Partial`; an entry that cannot be read makes its folder `Partial` and
  counts as `Failed`, as in the reference walk. No path is logged.
- **Deviation — a vanished folder keeps its node.** The arena cannot drop a node, and the reference walk keeps a folder
  that vanished as an unreadable one; so a folder removed during the walk is a `Failed` unreadable folder (counted in
  the summary), and the walk itself does not fail.
- What the reference walk does not count is not counted: an unreadable folder or an entry inside a whole claim, or a
  whole-claimed folder that cannot be read, makes the claim's sizes `Partial` but is not in `WalkSummary::unreadable`.
- **Known issue fixed in the reference walk** (found by 04): `NaiveWalk` looked marker files and siblings up through
  `symlink_metadata` on a joined path, so on a volume that ignores case `cachedir.tag` passed for `CACHEDIR.TAG` and
  `Package.json` for `package.json`; it now finds them among the names the folder lists, compared exactly, as the
  matcher does. A whole claim's size in the reference walk no longer enters an avoided folder inside it (it counts as
  not read, `Partial`), as the scanner never opens one. Both have tests in `fresher-testing`.
- `tests/reference.rs` (the portable reader) and `tests/macos_reader.rs` (the macOS reader) compare whole `Match`
  values, sorted by path and interest, and the summary's entries and unreadable counts, with an interest set of every
  pattern and both claims, on every fixture home (with an avoided `Documents`), on a tree of every kind of match (a
  valid tag, a hard link in a whole claim, siblings, an anchor), on links, names that look alike and case, and on
  unreadable folders. The property test builds random trees (folders, sparse files, links in and out, hard links, valid and
  invalid tags, unreadable folders) with a random subset of the interests; 48 cases a run.
- The property test found a bug, fixed in the walk before 03 was committed, with its seed kept in `tests/reference.proptest-regressions`: a folder
  inside a whole claim was read as if it were the claim's root, so a `CACHEDIR.TAG` deeper inside was taken as the
  root's own marker. The jobs inside an opaque subtree now carry no node.
- In the unit tests: the macOS and the portable readers build the same frozen tree of `SMALL`; a cloned file
  (`fresher_macos::fixtures::clone_file`, the `fixtures` feature as a dev-dependency) counts its shared blocks once in
  the tree and frees nothing when removed, while the portable reader, which cannot see clones, counts it twice.
- Checks: `cargo nextest run -p fresher-scanner -p fresher-testing`: 209 tests pass. Coverage: see the readme's
  conclusions (measured by the pre-push run).
