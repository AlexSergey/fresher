# 018/03/02 — The shared helpers and the core e2e flows

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** e2e, fixtures, docs

First the deferred shared helpers of the core e2e homes (due at this
harness, decision 14), then the module's flows on the real scanner and cleaner. Two commits: the move, then the flows.

## Files

The move (no test changes its assertions or its golden):

- `crates/e2e/tests/support/` — `until` (the events up to the last one wanted), `strip` (a key left out of a JSON
  report), `sort_by_path` (a report's lists in path order), `trash_unnumbered` (the Trash's order numbers left out of
  a tree report) and `contents` (a folder's files and their bytes), one copy each, beside `fixtures` and
  `check_golden`; split into small files if `mod.rs` grows past one concern.
- `crates/e2e/tests/jetbrains_home/mod.rs`, `temp_home/mod.rs`, `places.rs`, `cleaning/mod.rs` — their copies removed,
  the shared ones used.
- `docs/deferred/e2e-shared-helpers.md` removed, its row in `docs/deferred/README.md` removed (the README's rule).

The flows:

- `crates/e2e/tests/dev_caches.rs` (new) and its harness `crates/e2e/tests/dev_caches_home/mod.rs` (new) — the engine
  with the real scanner (its reader wrapped by the facts lens where a flow states a fact), the `dev-caches` module, the
  real settings, cache and journal files and the real `DiskCleaner` on a fixture home, with `FakeClock`, `FakeTrash`,
  `FakeRunningApps`, `FakeInstalledApps` (Xcode installed) and `FakeOpenFiles`; the shared helpers only. The eight
  flows of the design's section 6, "Core e2e", negatives first within each flow:
  1. `scan_confirm_trash_and_put_back` — npm, Cargo's `cache` and `src`, Gradle and the project offered under
     Development → Developer caches; the plan's highest danger Low; in the Trash and the journal; put back byte for
     byte; Cargo's `index`, `bin` and `credentials.toml` untouched (`contents` before and after).
  2. `a_cache_held_open_is_skipped_in_use` — a file of Gradle's caches held open: Gradle skipped `InUse`, the rest
     done.
  3. `derived_data_waits_while_xcode_is_open` — the project left out `app-open`; Xcode quits; a rescan offers it.
  4. `the_sign_is_read_again_by_a_rescan` — Gradle's tag invalid (43 zero bytes): not claimed; rewritten valid, a
     rescan offers Gradle's caches; removed, a rescan claims nothing.
  5. `a_cache_read_in_part_is_not_claimed` — a folder inside npm's cache set to mode 000 (restored at the end, even on
     failure): npm's cache not claimed, the folder counted as not read; every other answer unchanged.
  6. `a_restored_scan_offers_the_same` — after a scan, a new engine on the same data folder restores at launch: the
     same findings as the walk.
  7. `ignoring_a_category_then_the_module` — npm's category ignored: npm's finding in no total, the others stay; the
     module ignored: nothing; restore brings each back.
  8. `the_map_explains_the_caches` — the offered caches are explained; the project counts in Xcode's line.
- `fixtures/homes/dev-caches-flow.toml` (new) — the flows' home: npm's cache, Cargo's registry with its tag, `index`,
  `bin` and `credentials.toml`, Gradle's caches with its tag, one derived data project; valid tags with
  `fill = "cachedir-tag"`. `fixtures/golden/dev-caches-flow.json` (new, reviewed): flow 1's report.
- `crates/e2e/Cargo.toml` — `fresher-module-dev-caches` as a dev-dependency.

## Produces

The module's core flows and one copy of the e2e homes' helpers, for the acceptance and every later module's harness.

## Checks

- After the move, before the flows: `npm run test:e2e:core` passes with no golden changed (`git diff fixtures/golden`
  empty).
- `npm run test:e2e:core` passes; the dev-caches flows pass 150 runs in a row (`cargo nextest run -p fresher-e2e --test
  dev_caches --retries 0` in a loop), as plans 016's and 017's did.
- Flow 5 leaves no folder at mode 000 behind, whatever the outcome (a guard that restores it on drop).
- No flow writes outside its temporary folder (the e2e crate's guard).

## Conclusions

Built (3e37de7, 09d5b3f, f329b30): first the deferred shared helpers, one file per concern in
`crates/e2e/tests/support/` (`events.rs`: `until`; `report.rs`: `strip`, `sort_by_path`, `trash_unnumbered`; `tree.rs`:
`contents`), each pulled in with `#[path]` only by the binaries and harnesses that use it, the copies removed from the
JetBrains, temp and cleanup homes (and `places.rs`'s inline module renamed `home`); then the harness
`crates/e2e/tests/dev_caches_home/mod.rs`, the home `fixtures/homes/dev-caches-flow.toml`, the golden
`dev-caches-flow.json` and the eight flows of the design in `crates/e2e/tests/dev_caches.rs`, negatives first. The
deferred item `e2e-shared-helpers` and its row are removed.

Deviations and calls:

- One file per concern, not one `support/mod.rs` (F4): a module declared there would leave helpers unused in most of
  the ten binaries (`dead_code` under `-D warnings`); `trash_unnumbered` takes the Trash folder's name, the one thing
  its two copies differed in; `report.rs` carries an `#[expect(dead_code)]` where only `strip` is used.
- The harness scripts a `FakeChangeJournal` (a position and one answer per rescan) so flows 3 and 4 really run as
  rescans, asserted with `ScanKind::Rescan` (F5); `Flow::rescan()` does it.
- "The guard" of the brief is the real `DiskCleaner` with `Reach::SandboxFolder` on the flow's temporary folder (F16).
- Flow 4 adds Homebrew on disk, flow-local, so flow 1 keeps the brief's five caches. Its two rescans name the parents
  (the parent read again, the child copied) then the children (the child walked again, the parent copied): Cargo's
  two folders and Homebrew's downloads stay offered after each, so the scanner hands each sign place once.
- Flow 5 asserts the cache is not claimed and npm's node is `Partial`; it does not assert the folder "counted as not
  read": the scanner does not count an unreadable folder inside a whole-claimed match (known issue
  [`scanner-unreadable-folder-in-a-claimed-match.md`](../../../known-issues/scanner-unreadable-folder-in-a-claimed-match.md)).
  The test is named for what it proves, after a review.
- The Gradle tag is moved inside the sandbox rather than deleted: `fs::remove_file` is disallowed outside the cleaner.

Tests: `cargo clippy -p fresher-e2e --all-targets -- -D warnings` clean after the move and after the flows;
`npm run test:e2e:core`: 75 passed, 2 skipped (67 before the eight flows); no golden changed by the move; the new
golden reviewed. `cargo nextest run -p fresher-e2e --test dev_caches --retries 0` in a loop: 150 of 150 runs passed.
Parked minors: flow 1 asserts the group but not the module's name; the locked-folder guard relies on mode 000 and
fails confusingly as root; `dev_caches_home/mod.rs` repeats about 150 lines of `temp_home/mod.rs`
(done by plan 019/03/02).
