# 019/03/02 — The shared home harness and the core e2e flows

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** e2e, fixtures, docs

First the deferred shared home harness of the core e2e
(a deferred item due at this harness, decision 16), then the module's flows on the
real scanner and cleaner. Two commits: the move (no assertion and no golden changes), then the flows. The move creates
`crates/e2e/tests/support/home.rs`; the flows' harness uses it.

## Files

The move:

- `crates/e2e/tests/support/home.rs` (new) — the part `temp_home/mod.rs` and `dev_caches_home/mod.rs` repeat: the
  sandbox layout (`LAYOUT`), the restore and the walk, the findings, the clean, the outcomes, the journal, the relative
  paths, the user, and the engine's assembly from the parts a home supplies (its modules, its fakes, its stated facts,
  its avoid list). Pulled in with `#[path = "../support/home.rs"]` only by the homes that use it (never from
  `support/mod.rs`, which many test binaries include: an unused item there is `dead_code`); split into one file per
  concern if it holds more than one.
- `crates/e2e/tests/temp_home/mod.rs`, `dev_caches_home/mod.rs` — their copies removed; each keeps only its module's
  own (its fixture, its fakes, its scripted facts).
- `docs/deferred/e2e-module-harness-shared.md` removed, its row in `docs/deferred/README.md` removed.

The flows:

- `crates/e2e/tests/app_leftovers.rs` (new) and its harness `crates/e2e/tests/app_leftovers_home/mod.rs` (new) — the
  engine with the real scanner (its reader wrapped by the facts lens), the `app-leftovers` module registered beside
  `jetbrains`, `temp` and `dev-caches` (as the app registers them), the real settings, cache and journal files and the
  real `DiskCleaner` on a fixture home, with `FakeClock`, `FakeTrash`, `FakeRunningApps`, `FakeInstalledApps` (an
  installed `com.example.Editor`), `FakeOpenFiles` and `FakeChangeJournal`; the shared harness only. The eleven flows of
  the design's section 6, "Core e2e", negatives first within each flow:
  1. `scan_confirm_trash_and_put_back` — `com.example.Gone`'s ten leftovers offered under Apps → Removed apps'
     leftovers; the three categories selected; the plan's highest danger High; in the Trash and the journal; put back
     byte for byte; `com.example.Editor`'s and `com.apple.Safari`'s files untouched (`contents` before and after).
  2. `keeping_settings_cleans_only_the_caches` — the caches category alone: the plan names Safe; after the cleanup a
     rescan (the journal scripted with the cleaned folders, the scan asserted `ScanKind::Rescan`) offers the settings
     and the data, not the caches.
  3. `an_app_installed_again_keeps_its_files` — offered; `com.example.Gone` installed (`FakeInstalledApps` changed); a
     rescan (asserted) claims nothing of it.
  4. `an_app_open_from_a_disk_image_keeps_its_files` — running: left out `running`; it quits; a rescan (asserted)
     offers it.
  5. `files_changed_lately_wait_for_the_days` — every entry of the id stated 20 days old: left out `used-recently`;
     the clock 15 days on; a rescan (asserted, the journal naming no change) offers them.
  6. `spotlight_keeps_what_it_finds_or_cannot_see` — an app found elsewhere: `installed-elsewhere`; then the search
     incomplete (`failing_search`): `spotlight-unread`.
  7. `a_place_not_read_keeps_every_app` — the home's `Library/Logs` set to mode 000 (restored on drop, even on
     failure): every proven id left out `scan-again`; `dev-caches`' and `jetbrains`' answers unchanged.
  8. `containers_are_not_opened_until_asked` — the scanner's standing avoid holds `places_that_prompt(home)`, as the
     app composes it: a container-only id not claimed; a saved-state id offered.
  9. `a_restored_scan_keeps_everything` — a new engine on the same data folder restores at launch: every id left out
     `scan-again`; a scan then offers them.
  10. `the_map_names_a_removed_app_by_its_id` — the offered leftovers in the map's line of `com.example.Gone`,
      `installed: false`, named by the id.
  11. `ignoring_the_data_keeps_the_rest` — the data category ignored: its findings in no total, the caches and settings
      stay; restore brings it back.
- `fixtures/homes/app-leftovers-flow.toml` (new) — the flows' home: `com.example.Gone` in the ten forms (no container:
  flow 8 adds `com.example.Boxed`, container only, and the saved-state id), `com.example.Editor`'s files,
  `com.apple.Safari`'s caches; **every entry of a leftover states `age` and `changed`**, its folder a `[[dir]]`, its
  files directly inside; files synced as the tree vocabulary writes data files; atime never compared. A flow that needs
  another entry at a path has its own id. `fixtures/golden/app-leftovers-flow.json` (new, reviewed): flow 1's report.
- `crates/e2e/Cargo.toml` — `fresher-module-app-leftovers` as a dev-dependency.
- `docs/testing.md` — the shared harness of `crates/e2e/tests/support/home.rs` and `app_leftovers.rs` in the e2e list.

## Produces

The module's core flows and one copy of the e2e homes' harness, for the acceptance and every later module's harness.

## Checks

- After the move, before the flows: `npm run test:e2e:core` passes with no golden changed (`git diff fixtures/golden`
  empty) and `cargo clippy -p fresher-e2e --all-targets -- -D warnings` clean (no `dead_code` in any binary).
- `npm run test:e2e:core` passes; the app-leftovers flows pass 150 runs in a row (`cargo nextest run -p fresher-e2e
  --test app_leftovers --retries 0` in a loop), as plans 016 to 018 did.
- Every rescan flow asserts the scan was a rescan (`ScanKind::Rescan`), never a full scan.
- Flow 7 leaves no folder at mode 000 behind, whatever the outcome (a guard that restores it on drop).
- No flow writes outside its temporary folder (the e2e crate's guard).

## Conclusions

Built (b89887d, 2f208d5): first the move (no assertion and no golden changed): the shared home harness split by
concern under `crates/e2e/tests/support/` (`home.rs`, `home_engine.rs`, `home_scan.rs`, `home_rescan.rs`,
`home_cleanup.rs`, each pulled in with `#[path]` only by the homes that use all of it, F4), `temp_home` and
`dev_caches_home` keeping only their own; the deferred item removed. Then `app_leftovers.rs`, `app_leftovers_home/mod.rs`,
the home `app-leftovers-flow.toml` and the golden `app-leftovers-flow.json`: the module's flows beside jetbrains, temp and
dev-caches, as the app registers them. `FakeInstalledApps::then(apps)` in `fresher-testing` (F10); `fresher-macos` and
`fresher-module-app-leftovers` added to the e2e dev-dependencies (F13).

Deviations and calls:

- **Flow 5 runs 12 days on, not 15**: the scanner refuses a rescan when the cache is older than its journal position
  (`MAX_POSITION_AGE_SECS`, 14 days), so the leftovers are 20 days old and the clock moves 12 days (32 over the days
  setting, still a rescan).
- Flow 2's change journal also names `home/.Trash`; without it the rescan after the cleanup was a full scan.
- Gone holds all ten forms, its container included; only flow 8 avoids the places that prompt (F3). Boxed lives in the
  same home and is offered in the other flows and the golden (12 items).
- The swallow check (F5): the home holds `Caches/Yarn/v6` and `Caches/JetBrains/WebStorm2024.3` (and `2025.3`, in use);
  flow 1 asserts both are still offered by their own modules beside app-leftovers; flow 7 compares dev-caches' findings
  with `Library/Logs` locked and does not claim jetbrains' unchanged.
- The golden's list is under the key `offered` (a `findings` key is stripped by `relative`).
- Temp's `FakeOpenFiles` is one per `Flow` now, not per engine; it is never held, so no behaviour differs.

Tests: every rescan flow scripts `FakeChangeJournal` and asserts a rescan (`rescan()`); flow 7's locked-folder guard
restores the mode on drop. `cargo nextest run -p fresher-e2e`: 86 passed, 2 skipped; the new binary 150 runs in a row
with no failure; `fresher-testing` 210 passed; clippy clean.

Parked minors: flows 3 and 4 rely on one listing per scan (a comment or an assertion on the calls); a long line in
`docs/testing.md`; flow 2's `.Trash` entry has no reason comment.
