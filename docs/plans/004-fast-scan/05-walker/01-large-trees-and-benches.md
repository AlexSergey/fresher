# 004/05/01 — Large trees and the bench harness

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** testing, scanner, repo

## Files

- `crates/testing/src/large_tree.rs` — `LargeTree::build(spec: &TreeSpec, under: &Path) -> Result<LargeTree, FixtureError>`:
  - `TreeSpec { seed, entries, depth, fan_out, large_file_share, hard_links, deep_dependency_folders, cache_folders }`
    with two named specs, `TreeSpec::SMALL` (100 000 entries) and `TreeSpec::LARGE` (3 000 000 entries);
  - a seeded generator of our own (xorshift64*, ten lines; no `rand`), so a spec always gives the same tree;
  - sparse files (`set_len`), so declared sizes cost no data blocks; hard links with `std::fs::hard_link`; names
    drawn from a fixed list of realistic shapes (versions, hashes, `node_modules`-like trees, `CACHEDIR.TAG` folders);
  - built once under `under/<spec name>-<spec hash>/` with a `.complete` marker written last; a second `build` with
    the same spec reuses it; nothing is ever removed by the generator (`cargo clean` removes the folder under
    `target/`).
  - `LargeTree::expected()` — the entry count and total logical size the spec promises, for the checks.
- `crates/testing/src/lib.rs` — `pub use large_tree::{LargeTree, TreeSpec}`.
- `crates/scanner/Cargo.toml` — dev-dependencies `criterion` (exact version, `default-features = false`; MIT OR
  Apache-2.0, checked with `cargo deny check`) and `fresher-macos` (for the real reader and `peak_memory`);
  `[[bench]] name = "scan" harness = false`.
- `crates/scanner/benches/scan.rs` — the harness: builds `SMALL` or `LARGE` under `target/bench-trees/` (chosen by
  `FRESHER_BENCH_TREE`, default `SMALL`), groups `scan/*` and `matcher/*`, and after each group prints the peak memory
  from `fresher_macos::peak_memory()`. The groups are filled by the steps that build what they measure.
- `package.json` — `"bench": "cargo bench -p fresher-scanner --locked"`; not part of any hook.
- `docs/testing.md` — "Performance": how to run the benches, the two specs, where the trees live, that timing bars
  are held by hand and written in the plan's notes, and the deterministic proxies held by tests.

## Produces

`LargeTree` for every later bench and for the deterministic bars (06/03, 07/03, 08/03); the `scan` bench file the
next steps add groups to.

## Checks

- Negative: `a_spec_with_zero_entries_is_refused`, `an_unfinished_tree_is_rebuilt` (no `.complete` marker → built
  again into a fresh folder name, never over the old one).
- Positive: `the_same_seed_builds_the_same_tree` (two builds of a tiny spec, same `TreeReport`),
  `a_built_tree_matches_its_expected_counts`, `sparse_files_take_no_data_blocks` (allocated far below logical).
- `cargo deny check` passes with criterion; `cargo machete` finds no unused dependency.
- `npm run bench` runs (empty groups print nothing) and builds `SMALL` in under a minute on the developer's machine;
  the time to build `LARGE` and its size on disk are written in the notes.

## Conclusions

- `LargeTree::build` / `TreeSpec::{SMALL, LARGE}` / `Expected` in `crates/testing/src/large_tree.rs`; the generator is a
  ten-line xorshift64\*; the tree is `Library/Caches/<app>` folders with a valid `CACHEDIR.TAG`, `Projects/app<n>` with
  three levels of `node_modules` packages, and a breadth-first `Data/` tree of `depth` × `fan_out`; then sparse files in
  random folders (names of versions, hashes, logs, sources) and hard links of some of them. Exactly `entries` entries.
- Not spelled out, taken the simplest way: `TreeSpec` also has a `name` (the folder's name) and `large_file_share` is
  per thousand (no floats); the spec hash covers a generator version, so a changed generator never reuses an old tree.
  `expected()` runs the generator again without writing (it also gives the folder count, for the "reads = folders"
  checks). A refused spec is `FixtureError::InvalidSpec`. The test that removes `.complete` renames it instead (the
  workspace forbids deleting functions outside the cleaner).
- Files are sparse, so their allocated size is 0 and no generated file is kept as a large record (the threshold is on
  allocated bytes): the tree's nodes are the folders, as on the measured home (556 k nodes, 525 k folders).
- The bench `crates/scanner/benches/scan.rs` reaches the crate-private matcher through `fresher_scanner::bench`, a
  `#[doc(hidden)]` module for the crate's own benches. `criterion` 0.8.2 is a workspace dependency (default features
  off, `cargo_bench_support` on; Apache-2.0 OR MIT); `cargo deny check` and `cargo machete` pass. `npm run bench` runs
  `cargo bench -p fresher-scanner --bench scan --locked`.
- **Measured** (the developer's machine, other builds running): `matcher/entry/1000-names` 37–66 µs (median ≈ 50 ns an
  entry; 04 measured 28 ns on an idle machine); `SMALL` built in **7.9 s**; `LARGE` built in **529.5 s** (8.8 min),
  about 2.6 million inodes and ≈ 300 MB of disk (sparse files, metadata only). Peak memory after the groups: 17 MB
  (matcher), 18 MB (scan, `SMALL` built in the process), 49 MB (`LARGE` built in the process).
