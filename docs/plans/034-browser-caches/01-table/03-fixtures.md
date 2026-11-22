# 034/01/03 — The text, the fixtures, the golden and the contract

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** modules, testing, fixtures

The table rows on fixture trees through `run_module` (the reference walk), with the installed and running apps as
fakes; their golden report; the contract test over the new runs; the one new text. The design:
[`../00-design.md`](../00-design.md), sections 4, 5 and 7.

## Context for the implementer

- Read `modules/app-caches/tests/{fixtures.rs,contract.rs,support/mod.rs}` and `fixtures/homes/app-caches-*.toml`
  first: the new homes, systems and tests follow them (a head comment in each home saying what each path is for; the
  facts the disk cannot show, a socket and the user's lock, are stated with `kind = "socket"` and
  `flags = ["user-lock"]`; links with `[[symlink]]`).
- Real bundle ids and folder names are used, since the module ships its table; the apps' shown names are `Google
  Chrome`, `Chromium`, `Firefox`, `Redis Insight`. Fixture homes are injected roots, never the real home.
- Build with `CARGO_TARGET_DIR=/Users/serhiialeksandrov/Desktop/Work/Projects/my/github/fresher/target`.

## Files

- `modules/app-caches/locales/en.json` — `"app-caches.title.profile": "{app}, {profile}: {folder}"`.
- `fixtures/homes/app-caches-tables-not-claimed.toml` (new), scanned with Google Chrome (`com.google.Chrome`, helper
  `com.google.Chrome.helper`) and an app `OtherChat` (`com.example.OtherChat`) installed; nothing is claimed or left
  out:
  - `Library/Caches/Chromium/Default/Cache/data_0` (Chromium not installed);
  - `Library/Caches/Google/Chrome/Default/image_cache/x`, `Library/Caches/Google/Chrome/Default/Storage/ext/x` (names
    not in the row);
  - `Library/Caches/Google/Chrome/Profile 2/cache/data_0` (a case variant: names are exact);
  - `[[symlink]] Library/Caches/Google/Chrome/Profile 3/Cache` → `../../../../../elsewhere/kept` with
    `elsewhere/kept/x` (a link named like a cache);
  - `Library/Caches/Google/Chrome/Profile 4/Code Cache` as a file (not a folder);
  - `[[dir]] Library/Caches/Google/Chrome/Profile 5/Cache` (empty);
  - `Library/Caches/Google/Chrome/Cache/x` (a cache name with no profile above it: `Google/Chrome/*/Cache` needs
    one name more);
  - `Library/Application Support/OtherChat/Cache/x` (a cache-named folder of an app not in the table);
  - `Library/Application Support/RedisInsight/Cookies`, `Library/Application Support/RedisInsight/Local
    Storage/leveldb/000003.log` (an app's data, its app not installed here either).
- `fixtures/homes/app-caches-tables-kept.toml` (new), every app installed, each cache kept for its own reason:
  - `Library/Caches/Google/Chrome/Default/Cache/data_0` and a socket `.../Default/Cache/s` (`special`);
  - `Library/Application Support/RedisInsight/GPUCache/data_0` with `flags = ["user-lock"]` (`not-yours`);
  - `Library/Caches/Firefox/Profiles/abcd1234.default-release/cache2/entries/x`, Firefox running (`app-open`);
  - `Library/Caches/Chromium/Default/Code Cache/js/index`, only Chromium's helper `org.chromium.Chromium.helper`
    running (`app-open`; the installed Chromium declares that helper);
  - and once with the running apps failing (`no-system` for the two folders that reach row 9).
- `fixtures/homes/app-caches-tables-offered.toml` (new), every app installed and closed:
  - `Library/Caches/Google/Chrome/Default/Cache/Cache_Data/data_0`, `.../Default/Code Cache/js/index`,
    `.../Profile 1/Cache/Cache_Data/data_0` (two profiles);
  - `Library/Caches/Firefox/Profiles/abcd1234.default-release/cache2/entries/x`, `.../startupCache/startupCache.8.little`;
  - `Library/Application Support/RedisInsight/{Cache/Cache_Data/data_0,Code Cache/js/index,GPUCache/data_0,DawnGraphiteCache/data_0,DawnWebGPUCache/data_0}`;
  - beside them, never claimed: `Library/Caches/Google/Chrome/Default/image_cache/x`,
    `Library/Application Support/RedisInsight/Cookies`, `.../Local Storage/leveldb/000003.log`, `.../config.json`.
  Small files, real sizes (`size = 4096`).
- `fixtures/golden/app-caches-tables.json` (new) — `UPDATE_GOLDEN=1`, reviewed: the findings (identities, titles with
  their args, reasons, sizes, apps, dangers, removals) and the left-outs of the three homes.
- `modules/app-caches/tests/support/mod.rs` — the three homes and their systems (`tables_not_claimed`,
  `tables_kept`, `tables_kept_with`, `tables_offered`, `tables_offered_with`, `tables_system`), as the existing ones.
- `modules/app-caches/tests/tables.rs` (new) — the tests below and the golden comparison (`fixtures.rs` is plan 030's
  and stays as it is).
- `modules/app-caches/tests/contract.rs` — `every_run()` gains the table runs: not-claimed, kept, kept with the
  running apps failing, kept and offered with the installed apps failing, offered, offered handed in reverse.

## Produces

The table's fixtures, golden and contract, for 02.

## Checks

- `tests/tables.rs`, `mod negative`: `nothing_in_the_tables_not_claimed_home_is_offered_or_left_out`,
  `the_tables_kept_home_leaves_out_each_cache_with_its_reason` (`app-open` 2, `not-yours` 1, `special` 1),
  `running_apps_that_cannot_be_listed_keep_the_table_caches_that_reach_the_question` (`no-system` 2, `not-yours` 1,
  `special` 1), `installed_apps_that_cannot_be_listed_claim_no_table_cache_and_say_nothing`,
  `a_profile_s_and_an_app_s_data_are_never_in_a_finding` (no finding's subject is or holds `image_cache`, `Cookies`,
  `Local Storage`, `config.json`); `mod positive`: `the_tables_offered_home_offers_each_cache_folder_once`
  (ten: three Chrome, two Firefox, five Redis Insight), `browser_findings_name_the_browser_and_the_profile`,
  `web_engine_findings_name_the_app_and_the_folder`, `every_table_offer_empties_its_folder`,
  `table_findings_are_counted_in_their_app_s_line`, `the_tables_golden_report_matches`.
- `tests/contract.rs` passes: `every_app_caches_text_is_used` (`app-caches.title.profile` included),
  `every_declared_left_out_reason_is_given_by_a_fixture_run`, `every_path_removal_names_a_matched_path`.
- `cargo nextest run -p fresher-module-app-caches`; `cargo llvm-cov` (or `npm run test:cov`'s Rust part) shows the
  crate at or above 90 % of lines; clippy; fmt.

## Conclusions

Built as the step says: `app-caches.title.profile` ("{app}, {profile}: {folder}"), the three fixture homes
(`app-caches-tables-not-claimed`, `-kept`, `-offered`), the golden `app-caches-tables.json` (reviewed by eye: 0 findings
and 0 left-out; 0 findings and 4 left-out; 10 findings), the tests `tables.rs`, the contract test with
`Catalogue::Required` over the three categories, and a test that reversed matches give the same report. `cargo nextest
run -p fresher-module-app-caches`: 139 tests; 92 to 100 % of lines per file (`lib.rs` 92.07, `report.rs` 94.01).

Changes from the design: the test support was split in three (`support/mod.rs` holds what both families share,
`plain.rs` the plan 030 homes, `tables.rs` the new ones, each pulled in by the test files that use it with `#[path]`),
because a shared file made every test target fail `dead_code` under `-D warnings`.
