# 030/02/02 — The core e2e

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** e2e, fixtures

The engine with the real scanner, the module and the real `DiskCleaner` on a sandbox home, beside the other modules of
`Library/Caches` (`jetbrains`, `dev-caches`, `app-leftovers`), as the flows of `crates/e2e/tests/app_leftovers.rs`.
The first real module on `EmptyFolder`, so the emptying, the journal and put back are proven here. The design:
[`../00-design.md`](../00-design.md), sections 3 and 7.

## Files

- `fixtures/homes/app-caches-flow.toml` (new) — a home with an installed app's cache (`com.example.Editor`: a file
  and a folder of files at its top), its helper's (`com.example.Editor.Helper`), a removed app's leftovers
  (`com.example.Gone`: caches and a saved state, unchanged for 40 days, so `app-leftovers` offers them), Yarn's cache
  and a JetBrains IDE's caches (the other modules' answers under `Library/Caches`), and Apple's
  (`com.apple.Safari`).
- `crates/e2e/tests/app_caches.rs` (new) and `crates/e2e/tests/app_caches_home/` (the flow's home and helpers,
  through the shared harness of the sandbox flow homes; the four modules, scripted installed and running apps, the
  clock) — the flows, negatives first:
  - `mod negative`: `no_folder_is_offered_by_both_app_caches_and_app_leftovers` (the findings' paths are unique;
    Editor's two folders are `app-caches.library-caches`, Gone's cache is `app-leftovers.caches`, Yarn's and the IDE's
    are their modules', Apple's is no one's); `an_open_app_s_cache_is_left_out_and_offered_once_it_quits` (Editor
    running: left out `app-open`, a rescan after it quits offers it);
    `an_entry_held_open_skips_the_item_and_moves_nothing_of_it` (the open files fake names `Cache.db`: the item ends
    `InUse`, nothing moved, the folder unchanged, the helper's folder emptied);
    `a_restored_scan_offers_the_same_caches_and_judges_the_open_app_again` (a restored scan offers the same folders;
    a restart with Editor open restores them left out `app-open`, since the open apps are asked again at launch);
  - `mod positive`: `the_map_counts_both_folders_in_the_installed_apps_line`;
    `a_confirmed_cleanup_empties_the_folders_keeps_them_and_put_back_restores_every_entry` (the golden flow: the folders
    are still there and empty, their top-level entries in the sandbox's Trash, the outcomes `Emptied { moved: 2,
    skipped: 0 }` and `{ moved: 1, … }`, the journal names each entry, put back restores every entry byte for byte);
    `put_back_restores_every_entry_byte_for_byte_beside_a_file_the_app_wrote`.
- `fixtures/golden/app-caches-flow.json` (new) — the flow's first scan's report, reviewed.
- `crates/e2e/Cargo.toml` — the module as a dev-dependency.
- `docs/testing.md` — `app_caches.rs` in the list of core e2e files.

`crates/e2e/tests/app_leftovers.rs` is not changed: it registers its own list of modules, without `app-caches`
(agent's call 12).

## Produces

The module proven through the engine and the cleaner, for 03.

## Checks

- `cargo nextest run -p fresher-e2e --test app_caches --test app_leftovers`; `npm run test:e2e:core`.

## Conclusions

Built as written, eight tests over seven flows: no folder is offered by two modules (four modules on one home); an open app's cache is left out and offered after it quits; an entry held open skips the item and moves nothing of it; a restored scan; the golden flow (scan, confirm, emptied folders stay, entries in the Trash, journal, put back byte for byte); put back beside a file the app wrote; the map counts both folders in the app's line. A restored scan judges the open apps again, so the design's "cached matches at launch are offered" holds while the app is closed (the final review judged this right). Final wave: the restore flow was split by what it asserts, the overlap check uses a set.
