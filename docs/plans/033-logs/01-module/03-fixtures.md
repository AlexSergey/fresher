# 033/01/03 — Fixtures, golden and the contract

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** modules, testing, fixtures

The module on fixture trees through `run_module` (the reference walk), its golden report, and the contract test over
the fixture runs. Model: `modules/temp/tests/{fixtures.rs,contract.rs,support/mod.rs}` and
`fixtures/homes/temp-*.toml` (read them first: the TOML format of `[[file]]`, `[[dir]]`, `[[symlink]]`, `age`,
`changed`, `owner`, `flags`, `kind`). The app's registration is made in 02/01, not here. The design:
[`../00-design.md`](../00-design.md), section 9.

Every old file states `changed` as well as `age` (a file written now has a status change of now). Times count back
from `FIXTURE_NOW`. Paths are relative to the fixture home; the fixture known folders put `Logs` at `Library/Logs`.

## Files

- `fixtures/homes/logs-not-claimed.toml` (new) — all 30 days old (`age = "30d"`, `changed = "30d"`), nothing
  claimed and nothing left out:
  `Library/Logs/com.example.Gone/app.log` (a removed app's folder: `app-leftovers`'),
  `Library/Logs/com.apple.Safari/x.log` (a bundle-id name, Apple's),
  `Library/Logs/JetBrains/WebStorm2024.3/idea.log` and `Library/Logs/JetBrains/stray.log` (`jetbrains`' place),
  a symlink `Library/Logs/linked.log` → `../../Documents/kept.log` with the file `Documents/kept.log`,
  an empty folder `Library/Logs/Empty`,
  `Library/Logs/Agent/heartbeat_last.plist`, `Agent/state.db`, `Agent/index.sqlite`, `Agent/index.sqlite-wal`,
  `Agent/index.sqlite-shm`, `Agent/run.lock`, `Agent/server.pid` (state and lock names),
  `Library/Logs/a/b/c/d/e/f/deep.log` (seven names deep),
  `Documents/Logs/old.log` and `Logs/old.log` (folders named alike outside the Library).
- `fixtures/homes/logs-kept.toml` (new) — under `Library/Logs/Keeper/`: `fresh.log` (`age = "1d"`, `changed = "1d"`),
  `six-days.log` (6 days), `future.log` (`age = "-2d"`, `changed = "30d"`), `rotated.log` (`age = "30d"`,
  `changed = "1d"`), `s.sock` (`kind = "socket"`, 30 days), `root.log` (`owner = "system"`, 30 days), `theirs.log`
  (`owner = "other-user"`, 30 days), `locked.log` (`flags = ["user-lock"]`, 30 days). Left out: `recent` ×4,
  `special` ×1, `not-yours` ×2, `locked` ×1, all in `logs.app-logs`.
- `fixtures/homes/logs-offered.toml` (new) — small files with real sizes: `Library/Logs/old-tool.log` (30 days, the
  top), `Library/Logs/com.example.Tool.log` (30 days, a top-level file shaped as a bundle id: offered, only a folder
  of that shape is another module's), `Library/Logs/Writer/writer.1.log` (30 days), `Library/Logs/Vendor/Updater/1.0/updater.log` (20 days, depth
  4), `Library/Logs/a/b/c/d/e/six.log` (30 days, six names deep), `Library/Logs/Writer/writer.10d.log` (10 days:
  offered at 7, kept `recent` at 14), `Library/Logs/DiagnosticReports/Writer-2026-10-01-101010.ips` (30 days),
  `Library/Logs/DiagnosticReports/Retired/Old-2026-09-01-000000.ips` (60 days); a fresh `Library/Logs/Writer/writer.log`
  (1 day) beside the old ones, kept `recent`.
- `fixtures/golden/logs.json` (new) — `UPDATE_GOLDEN=1`, reviewed: the offered home's findings (identities,
  categories, titles, reasons with their arguments, sizes, dangers, removals) and left-outs.
- `modules/logs/tests/support/mod.rs` (new) — the homes, the system they are scanned with (stored settings as JSON,
  restored or walked, the match order), one run of the module per home, as `modules/temp/tests/support/mod.rs`
  (no installed or running apps are needed; pass the defaults).
- `modules/logs/tests/fixtures.rs` (new) — below.
- `modules/logs/tests/contract.rs` — the fixture runs added: the three homes, the offered home restored (`scan-again`)
  and with `{"logs.older-than-days": 14}`; tests `every_logs_text_is_used` (`assert_texts_used`),
  `every_declared_left_out_reason_is_given_by_a_fixture_run` (the five keys exactly),
  `all_three_finding_reasons_are_given_by_a_fixture_run` (`logs.reason.top`, `.folder`, `.report`),
  `every_path_removal_names_a_matched_path` (`check_removals_matched` empty), and
  `every_removal_moves_a_file_to_the_trash` (each finding's removal is `Removal::MoveToTrash` of its subject's path and
  that path was matched as `EntryKind::File`).
- `modules/logs/Cargo.toml` — `serde_json.workspace = true` as a dev-dependency (the golden, the stored settings); its
  row in `docs/dependencies.md` (both already added by 01/02).

## Produces

The module's fixtures, golden and contract, for 02.

## Checks

- `tests/fixtures.rs`, `mod negative` first: `nothing_in_the_not_claimed_home_is_offered_or_left_out`,
  `the_kept_home_offers_nothing_and_leaves_out_each_with_its_reason`,
  `a_restored_scan_of_the_offered_home_only_leaves_out`, `a_longer_setting_keeps_the_ten_day_old_log`,
  `no_folder_is_ever_offered` (over every run: every finding's subject was matched as a file); `mod positive`
  `the_offered_home_offers_every_old_file_once`, `crash_reports_and_app_logs_are_apart`,
  `every_offer_moves_its_file_to_the_trash`, `the_golden_report_matches`.
- `tests/contract.rs` passes with the fixture runs.
- `CARGO_TARGET_DIR=/Users/serhiialeksandrov/Desktop/Work/Projects/my/github/fresher/target cargo nextest run -p
  fresher-module-logs`; coverage of the crate at or above 90 % of lines
  (`cargo llvm-cov nextest -p fresher-module-logs` or the repository's coverage script).
- Commit: `test(modules): plan 033 — the logs module's fixtures, golden and contract`.

## Conclusions

Built (b2f7f456): three fixture homes (not claimed, kept, offered), the golden of the offered home, the fixture tests
and the contract test with `Catalogue::Required`. `ModuleRun` exposes matched paths only, not kinds, so "matched as a
file" is checked as the subject being a regular file in the tree. 66 tests in the crate; `modules/logs/src` at 99.64 %
of lines (280 of 281).
