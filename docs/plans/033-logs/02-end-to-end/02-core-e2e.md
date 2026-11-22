# 033/02/02 — The core e2e

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** e2e, fixtures, docs

The engine with the real scanner, the module and the real `DiskCleaner` on a sandbox home, beside the other modules of
`Library/Logs` (`jetbrains`, `app-leftovers`). Model: `crates/e2e/tests/app_caches.rs` with
`crates/e2e/tests/app_caches_home/` (the shared harness of the sandbox flow homes, scripted installed and running apps,
the open-files fake, the clock), and `crates/e2e/tests/temp.rs` (a day setting through the real settings file, a
restored scan). Read them first. The design: [`../00-design.md`](../00-design.md), sections 3, 4 and 9.

## Files

- `fixtures/homes/logs-flow.toml` (new) — a sandbox flow home, old entries 30 days (`age` and `changed`):
  - `home/Library/Logs/Writer/writer.1.log` (old, 64 KiB, data) and `home/Library/Logs/Writer/writer.log` (1 day:
    fresh, beside it);
  - `home/Library/Logs/Vendor/Updater/updater.log` (old) and `home/Library/Logs/Vendor/Updater/state.plist` (old, a
    state file);
  - `home/Library/Logs/notes-10d.log` (10 days);
  - `home/Library/Logs/DiagnosticReports/Writer-2026-10-01-101010.ips` (old);
  - a removed app's leftovers as `crates/e2e/tests/app_leftovers.rs`' home lays them out (`com.example.Gone`: its
    `Library/Logs/com.example.Gone/app.log`, caches and saved state, unchanged for 40 days, not installed, so
    `app-leftovers` offers them);
  - an older JetBrains IDE's logs as `crates/e2e/tests/jetbrains.rs`' home lays them out
    (`home/Library/Logs/JetBrains/WebStorm2024.3/idea.log`, the product's newer version installed, so `jetbrains`
    offers the old folder).
- `crates/e2e/tests/logs.rs` (new) and `crates/e2e/tests/logs_home/` (the flow's home and helpers, through the shared
  harness; the three modules registered in the order of the app's `modules()`: `jetbrains`, `app-leftovers`, `logs`)
  — the flows, negatives first:
  - `mod negative`:
    - `no_path_is_offered_by_two_modules` (the findings' paths are unique; no `logs` finding lies inside
      `Logs/com.example.Gone` or `Logs/JetBrains`; Gone's folder is `app-leftovers.caches`, WebStorm 2024.3's is a
      `jetbrains` category; `state.plist` is no one's);
    - `a_fresh_log_is_left_out_and_stays_after_the_cleanup` (`writer.log` left out `recent`; after a confirmed cleanup
      of everything offered it is unchanged, byte for byte);
    - `a_log_held_open_is_skipped_and_moves_nothing` (the open-files fake names `updater.log`: the item ends `InUse`,
      the file stays, the other items move);
    - `a_restored_scan_leaves_every_log_out_until_a_rescan` (a restart restores the scan: every `logs` match left out
      `scan-again`, no `logs` finding; a rescan offers them again);
    - `a_longer_setting_keeps_the_ten_day_old_log` (`logs.older-than-days` = 14 in the settings file: `notes-10d.log`
      left out `recent`; at the default it is offered);
  - `mod positive`:
    - `a_confirmed_cleanup_moves_the_old_logs_to_the_trash_and_keeps_their_folders` (the golden flow: scan, confirm the
      `logs` findings, the files in the sandbox's Trash, `Logs/Writer`, `Logs/Vendor/Updater` and
      `Logs/DiagnosticReports` still there, the outcomes `Moved`, the journal names each file);
    - `put_back_restores_every_log_byte_for_byte`.
- `fixtures/golden/logs-flow.json` (new) — the flow's first scan's report (the three modules' findings and left-outs),
  reviewed.
- `crates/e2e/Cargo.toml` — `fresher-module-logs` as a dev-dependency (and `fresher-module-jetbrains`,
  `fresher-module-app-leftovers` if not already there).
- `docs/testing.md` — `logs.rs` in the list of core e2e files, one line saying what it proves.

`crates/e2e/tests/jetbrains.rs` and `app_leftovers.rs` are not changed: each registers its own modules; the split is
proven in `logs.rs`.

## Produces

The module proven through the engine and the cleaner, for 03.

## Checks

- `CARGO_TARGET_DIR=/Users/serhiialeksandrov/Desktop/Work/Projects/my/github/fresher/target cargo nextest run -p
  fresher-e2e --test logs --test jetbrains --test app_leftovers`; `npm run test:e2e:core`.
- Commit: `test(e2e): plan 033 — the logs module's core flows`.

## Conclusions

- `crates/e2e/tests/logs.rs` has the five negatives and two positives as specified, over
  `fixtures/homes/logs-flow.toml` with `jetbrains`, `app-leftovers` and `logs` registered in the app's order. Every
  flow passed as designed on the first run, so the module is unchanged.
- `fixtures/golden/logs-flow.json` is reviewed: the three modules' findings (Gone's three folders
  `app-leftovers.caches`, WebStorm 2024.3's logs `jetbrains.logs`, four files of `logs`), one `recent` left-out, and
  the module's findings with their titles, reasons (`top`, `folder` with `Writer` and `Vendor/Updater`, `report`) and
  sizes; the findings' ids are dropped and the list sorted by identity, since the walk's order is not the report's.
- The setting goes through `update_setting` into the real settings file, as `temp.rs` does (a fixture home cannot
  state settings).
- By hand, reverted: the fresh log made 8 days old (`age` and `changed`) failed six of the seven flows.
