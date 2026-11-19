# 020/03/03 — Fixtures, golden and registration

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** modules, testing, app

The module run on real fixture trees with `run_module` (the real walk of the tree, the fakes of the system), its
golden report, and its registration in every build after `app-leftovers`. The design:
[`../00-design.md`](../00-design.md), section 4, "Fixture".

## Files

- `fixtures/homes/downloads-not-claimed.toml`, `downloads-kept.toml`, `downloads-offered.toml` (new) — the homes of
  the design's fixture table, every file under `Downloads/`; **every entry states `age` and `changed`** (the status
  change cannot be set back on disk, and an entry written at the test's real time is "changed lately"); big files are
  sparse (`size` with no `fill`), so 1 GB costs no disk; a negative that needs another entry at a path has its own
  name or its own home (no two entries at one path in one home).
  - not claimed: `report.pdf` (400 days), `notes-backup.dmg`, `install.dmg`, `Firefox (1).dmg` (with Firefox 133.0
    installed, `changed` 10 days ago), a link `movie.mp4.part` → `elsewhere/x`, `photos/big.mov` (2 GB, 400 days),
    `Example.app` (a bundle with `Contents/`, no row of `macos.toml`), `trip.download/` without `Info.plist`,
    `almost.mov` (999 999 999 bytes, 400 days), `Notes.dmg` with an installed `com.apple.Notes`;
  - kept: `a.crdownload` (40 days) with Chrome running → `browser-open`; `b.opdownload` (3 days, Opera not running) → `recent`;
    `Firefox 140.0.dmg` (5 days) beside Firefox 133.0 changed 10 days ago → `not-older`; `Docker.dmg` (5 days) with
    Docker changed 20 days ago → `not-older`; `Install macOS Tahoe.app/Contents/x` with the version 15.4 →
    `newer-macos`; `recent.mov` (2 GB, 3 days) → `recent`; `s.download/{Info.plist, x.sock (kind = "socket")}` (40
    days) → `special`;
  - offered: `movie.mp4.crdownload` and `song.flac.opdownload` and `doc.pdf.partial` (40 days), `video.mkv.part`
    (40 days) with an empty `video.mkv`, `trip.zip.download/{Info.plist, trip.zip}` (40 days), `Firefox 133.0.dmg`
    (20 days; Firefox 133.0 changed 10 days ago), `Example-2.0.pkg` (20 days; a receipt `Example-2.0.pkg` installed 10
    days ago; no app), `Install macOS Sonoma.app/Contents/x` (version 15.1), `video.mov` (2 GB, 400 days). Positives'
    small files `fill = "data"`.
- `modules/downloads/tests/support/mod.rs` (new) — the fixtures' shared setup, as `modules/app-leftovers/tests/support/
  mod.rs`: a `System` builder (installed apps with `changed_at`, running apps, receipts, the version, failing ports,
  restored, the order).
- `modules/downloads/tests/fixtures.rs` (new) — `mod negative`:
  `nothing_in_the_not_claimed_home_is_claimed_or_left_out`, `each_kept_item_is_left_out_with_its_reason` (a count per
  reason), `failing_running_apps_keep_the_unfinished_as_no_system`,
  `a_partial_app_list_keeps_installers_as_apps_unread`, `a_failing_app_list_keeps_installers_as_no_system`,
  `a_failing_version_keeps_macos_installers_as_no_system`, `a_restored_run_keeps_everything_to_scan_again`,
  `downloads_not_scanned_offers_nothing` (`Ports::avoid` holds `Downloads`); `mod positive`:
  `every_category_is_offered_at_its_danger` (each title, reason, size; the installer `for_app` Firefox), the golden
  `fixtures/golden/downloads.json` (`UPDATE_GOLDEN=1`, reviewed), `the_setting_moves_the_days`
  (`{"downloads.older-than-days": 60}` keeps the 40-day unfinished files `recent`).
- `modules/downloads/tests/contract.rs` — the contract over the fixture runs (every declared left-out reason used by
  a run, every removal a matched path), as `app-leftovers`' does.
- `app/tauri/Cargo.toml` — `fresher-module-downloads`; `app/tauri/src/compose.rs` — `modules()` registers
  `DownloadsModule::new()?` after `AppLeftoversModule`; `ComposeError` takes its error as the others do.

## Produces

The registered module and its golden, for 04.

## Checks

- `cargo nextest run -p fresher-module-downloads -p fresher-app` (the compose tests list the registered modules); the
  golden reviewed by hand: no path outside `Downloads/` in it, every category present.
- Registration makes `npm run sandbox:test` require `fixtures/sandboxes/downloads.toml` (a registered module without
  a case file fails the autotest, rule 8): 04/01 follows this step before any push, and this step's conclusions say
  that the autotest is red until then.
- Coverage of `modules/downloads` at or above 90 % (`npm run test:cov` or `cargo llvm-cov nextest -p
  fresher-module-downloads`); clippy; fmt.

## Conclusions

- Built as written, with the pre-flight rulings: `f.download` (a file of the system's, `not-yours`) in the kept home,
  `Tool.dmg` (an installed `com.example.Tool` with no version, `installer-no-version`) in the offered home, and
  `Firefox 140.0.dmg` 20 days old, so only the version check keeps it `not-older`. Every declared left-out reason and
  every text is produced by a fixture run (the contract checks both).
- The offers come through the real walk (11 findings in the offered home, among them the placeholder), so the path
  form the module compares with the system's Downloads is proven by the fixtures, not only by the unit tests.
- The golden `fixtures/golden/downloads.json` holds the three homes by name; it was reviewed: every path is under
  `Downloads/`, every category is present.
- Registered after `app-leftovers` in `compose.rs`, with `ComposeError::Downloads`; the module lists of both lists in
  `compose.rs` and of `commands.rs` are updated.
- `npm run sandbox:test` (the app's `every_registered_module_has_a_case_file`) is red until 04/01 adds
  `fixtures/sandboxes/downloads.toml` and its two siblings. Nothing else is red.
- Coverage of `modules/downloads`: 95 % or more of the lines in every file.
