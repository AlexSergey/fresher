# 034/02/02 — The core e2e

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** e2e, fixtures, docs

The engine with the real scanner, the module and the real `DiskCleaner` on a sandbox home, beside the other modules of
`Library` (`jetbrains`, `dev-caches`, `app-leftovers`), for the table's rows: one claim per folder, the data beside the
caches untouched, the open browser, the emptying and put back. The design: [`../00-design.md`](../00-design.md),
section 7.

## Context for the implementer

- Read `crates/e2e/tests/app_caches.rs` and `crates/e2e/tests/app_caches_home/mod.rs` first and follow them: the same
  shared harness (`support/home*.rs`, `support/tree.rs`), the same four modules in the app's order, scripted installed
  and running apps, the files held open, the clock. `app_caches.rs` stays as it is.
- The e2e crate already depends on `fresher-module-app-caches` (plan 030): no `Cargo.toml` change.
- Build with `CARGO_TARGET_DIR=/Users/serhiialeksandrov/Desktop/Work/Projects/my/github/fresher/target`.

## Files

- `fixtures/homes/app-caches-browsers-flow.toml` (new) — a home with:
  - Google Chrome's caches of two profiles: `Library/Caches/Google/Chrome/Default/Cache/Cache_Data/{index,data_0,data_1}`
    and `.../Default/Code Cache/js/index`, `.../Profile 1/Cache/Cache_Data/data_0`, and beside them
    `Library/Caches/Google/Chrome/Default/image_cache/x`;
  - Chrome's profile data in its support folder, never touched: `Library/Application Support/Google/Chrome/Default/
    {History,Cookies,Login Data,Preferences}`;
  - Redis Insight's support folder: `Library/Application Support/RedisInsight/{Cache/Cache_Data/data_0,GPUCache/data_0}`
    beside `Cookies` and `Local Storage/leveldb/000003.log`;
  - the other modules' answers: a removed app's leftovers (`Library/Caches/com.example.Gone/Cache.db` and its saved
    state, unchanged for 40 days, as in `app-caches-flow.toml`), Yarn's cache (`Library/Caches/Yarn/v6/x`), an older
    JetBrains IDE's caches, and an installed app's id-named cache (`Library/Caches/com.example.Editor/Cache.db`).
- `crates/e2e/tests/app_caches_browsers.rs` (new) and `crates/e2e/tests/app_caches_browsers_home/mod.rs` (new) —
  Google Chrome (`com.google.Chrome`), Redis Insight (`org.RedisLabs.RedisInsight-V2`), `com.example.Editor` and the
  IDE installed. The flows, negatives first:
  - `mod negative`:
    - `no_folder_is_offered_by_two_modules` (the findings' paths are unique across the four modules; Chrome's three
      caches are `app-caches.browsers`, Redis Insight's two `app-caches.web-engine`, Editor's `app-caches.library-caches`,
      Gone's `app-leftovers.caches`, Yarn's and the IDE's their modules');
    - `a_profile_s_and_an_app_s_data_are_never_offered` (no finding's subject is, holds or lies inside `History`,
      `Cookies`, `Login Data`, `Preferences`, `image_cache`, `Local Storage`; after a cleanup of every finding they are
      byte for byte unchanged);
    - `an_open_browser_s_caches_are_left_out_and_offered_once_it_quits` (Chrome running: its three caches left out
      `app-open`, Redis Insight's still offered; a rescan after it quits offers them);
    - `an_entry_held_open_skips_the_cache_and_moves_nothing_of_it` (the open files fake names
      `Default/Cache/Cache_Data/index`: that item ends `InUse`, nothing of it moved, the folder unchanged; the other
      caches emptied);
  - `mod positive`:
    - `a_confirmed_cleanup_empties_each_cache_keeps_it_and_put_back_restores_every_entry` (the golden flow: each cache
      folder still there and empty, its top-level entries in the sandbox's Trash, `Emptied` outcomes, the journal
      names each entry, put back restores every entry byte for byte);
    - `the_map_counts_the_caches_in_their_app_s_line` (Chrome's three in Chrome's line, Redis Insight's two in its
      line).
- `fixtures/golden/app-caches-browsers-flow.json` (new) — the first scan's report, `UPDATE_GOLDEN=1`, reviewed.
- `docs/testing.md` — `app_caches_browsers.rs` in the list of core e2e files, one paragraph in the style of the
  `app_caches.rs` one.

## Produces

The table proven through the engine and the cleaner, for 03.

## Checks

- `cargo nextest run -p fresher-e2e --test app_caches_browsers --test app_caches`; `npm run test:e2e:core`.

## Conclusions

Built as the step says: `fixtures/homes/app-caches-browsers-flow.toml`, the harness `app_caches_browsers_home`, the
tests of `app_caches_browsers.rs` (negatives: no folder is offered by two modules, a profile's and an app's data are
never offered, an open browser's caches are left out and offered once it quits, an entry held open skips the cache and
moves nothing of it; positives: a confirmed cleanup empties each cache, keeps it and put back restores every entry
(golden `app-caches-browsers-flow.json`, 10 findings, 6 `Emptied`), the map counts the caches in their app's line).
`cargo nextest run -p fresher-e2e --test app_caches_browsers --test app_caches`: 14 tests; `npm run test:e2e:core`:
144 passed, 2 skipped. No production code was touched, so no coverage was measured.

By-hand mutations, each reverted: the Chrome row's path `Google/Chrome/*` widened to `Google/*` fails 5 of the 6 flows
(the data test rightly passes); adding `image_cache` to the Chrome and Chromium caches fails all 6.

The Trash question of the case files' review is not a defect: the sandbox's fake Trash keeps same-named entries apart
with an order prefix (`<n>-Cache_Data`), and the sandbox harness and the e2e report drop the prefix so a golden does
not depend on the order of the moves, hence three `home/.Trash/Cache_Data` lines. The new e2e asserts it on disk: six
top-level Trash entries after three `Cache_Data` folders moved, every byte present, put back restores each to its own
cache. The golden and held-open flows clean the module's three categories, so Editor's cache is emptied beside the
table's caches.
