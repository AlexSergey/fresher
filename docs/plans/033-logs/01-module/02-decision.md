# 033/01/02 — The decision

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** modules, docs

What the module does with each match: find its place, decide by the rows of the design's section 4 — offered, left
out with its reason, or not claimed — and report. Pure decisions on plain values, so every row is a unit test; the
module only reads the match and reports, as `modules/temp` does (read its `src/decide.rs`, `src/names.rs`,
`src/decide/tests.rs`, `src/tests.rs` first; write this module's own code, never import from `temp`). The design:
[`../00-design.md`](../00-design.md), sections 2 to 6. This step also writes the second exception of "Unknown is not
junk" into `docs/restrictions/deletion-safety.md`, in the same commit as the decision that relies on it.

## Files

- `modules/logs/src/names.rs` (new) — on bytes, never assuming UTF-8:
  - `pub(crate) enum Place { AppLogs, CrashReports }`;
  - `pub(crate) struct Located<'a> { place: Place, below: Vec<&'a OsStr> }`: the names of the path below the place's
    folder (below `Logs` for app logs, below `Logs/DiagnosticReports` for crash reports);
  - `pub(crate) fn locate<'a>(logs: &Path, path: &'a Path) -> Option<Located<'a>>`: `None` when `path` is not strictly
    inside `logs` by components, or when its first name below `logs` is another module's: `JetBrains` (ASCII case
    ignored) or, when more names follow it (a folder), `fresher_core::is_bundle_id_shaped(name)` — a top-level file
    shaped as a bundle id is an app log (fix round 1); `DiagnosticReports` (ASCII case ignored) as the first name
    followed by at least one more name gives `CrashReports` with the rest; `DiagnosticReports` alone (the folder, or a
    file of that name) is an app log named `DiagnosticReports` (a folder is refused by row 1 anyway);
  - `pub(crate) fn is_state_or_lock(name: &OsStr) -> bool`: the name ends with `.plist`, `.db`, `.sqlite`,
    `.sqlite-wal`, `.sqlite-shm`, `.lock` or `.pid`, ASCII case ignored, and is longer than the suffix;
  - `pub(crate) fn identity(names: &[&OsStr]) -> String`: each name as its UTF-8, a backslash doubled, any other byte
    `\xNN` (uppercase hex), joined by `/`;
  - `pub(crate) fn folder_text(names: &[&OsStr]) -> Option<String>`: the names but the last, lossy UTF-8, joined by
    `/`; `None` when there are none (a file at the top of its place).
- `modules/logs/src/decide.rs` (new) — on plain values, never a port:
  - `pub(crate) struct Item { kind: EntryKind, facts: EntryFacts, modified: Option<Timestamp>, changed:
    Option<Timestamp>, state_or_lock: bool }`; `pub(crate) struct Scan { restored: bool, now: Timestamp, days: u32 }`;
  - `pub(crate) enum Reason { ScanAgain, Special, NotYours, Locked, Recent }` with `ALL` (that order) and
    `key(self) -> &'static str` (`logs.left-out.<kebab name>`);
  - `pub(crate) enum Verdict { NotClaimed, LeftOut(Reason), Offered { age_days: u64 } }`;
  - `pub(crate) fn decide(item: &Item, scan: &Scan) -> Verdict`: rows 1 and 3 to 9 of the design's section 4 in
    order (row 2, another module's place, is `locate` returning `None`, checked by the caller before `decide`):
    a folder or a link → `NotClaimed`; `state_or_lock` → `NotClaimed`; `restored` → `ScanAgain`; `EntryKind::Other`
    → `Special`; not `owned_by_user`, `system_immutable` or `restricted` → `NotYours`; `user_immutable` → `Locked`;
    the newest of `modified` and `changed` unknown (either one `None`), later than `now`, or `now - newest <= days *
    86_400` → `Recent`; otherwise `Offered { age_days: (now - newest) / 86_400 }`.
- `modules/logs/src/lib.rs` — `on_match`: locate (`cx.system().locate(KnownFolder::Logs)`; `None` → not claimed),
  build `Item` and `Scan` (`cx.is_restored()`, `Timestamp::from_system_time(cx.clock().now())`,
  `OLDER_THAN.days(cx.settings())`), decide, then report:
  - `LeftOut(reason)` → `out.left_out(LeftOutItem { category: <the place's category id>, reason: <key>, sizes })`;
  - `Offered { age_days }` → `NewFinding::new(&category, identity, Subject::Path(path), sizes, title, reason,
    Removal::MoveToTrash(path))`: identity `app-logs/<identity(below)>` or `crash-reports/<identity(below)>`; title
    `logs.title` with `name` (`TextArg::Text`, the file name lossy); reason `logs.reason.report` with `days`
    (`TextArg::Count`) for a crash report, else `logs.reason.folder` with `folder` (`TextArg::Text`) and `days` when
    `folder_text` is `Some`, else `logs.reason.top` with `days`. No `for_app`.
  - `finish` stays `Ok(())`. The module never asks the installed or the running apps.
- `modules/logs/src/decide/tests.rs`, `modules/logs/src/tests.rs` (new) — below. `modules/logs/Cargo.toml` —
  `proptest.workspace = true` as a dev-dependency; its row in `docs/dependencies.md`.
- `docs/restrictions/deletion-safety.md` — "Unknown is not junk": "The one exception" becomes "**Two exceptions**":
  the first as it is (`temp`); the second (plan [033](../plans/033-logs/readme.md), agent's call 1): a regular file
  under the user's `~/Library/Logs`, up to six names deep, is claimed by the place's contract whatever its name — it is
  the place macOS gives apps for their logs — and offered, to the Trash, only after every row of the `logs` module's
  decision: not another module's place (`JetBrains/`, a bundle-id-shaped top folder), not a state or lock name, not
  restored, no socket, pipe or device, the user's and unflagged, not locked, and unchanged for more than the setting's
  days ([`../categories/logs-app-logs.md`](../categories/logs-app-logs.md),
  [`../categories/logs-crash-reports.md`](../categories/logs-crash-reports.md)). No folder is claimed. "Nowhere else
  does a name the app cannot read count." stays. "Enforced by" adds the `logs` module's decision tests
  (`modules/logs/src/decide.rs`, `names.rs`), its fixtures and its case file (`fixtures/sandboxes/logs.toml`).

## Produces

The module's answers, for 01/03 (fixtures and golden) and 02.

## Checks

- `names.rs` tests, `mod negative` first: `a_path_outside_the_logs_has_no_place` (a sibling `/home/Library/Logsx/a`,
  the folder itself, `/home/Documents/Logs/a.log`), `a_jetbrains_path_has_no_place` (also `jetbrains/…`),
  `a_file_in_a_bundle_id_top_folder_has_no_place` (`com.example.App/app.log`; fix round 1: a top file
  `com.example.App.log` is an app log, `a_top_level_file_shaped_as_a_bundle_id_is_an_app_log`),
  `a_name_like_but_not_a_state_or_lock_is_not_one` (`plist`, `.plist` alone, `x.plist.log`, `x.dbx`, `lockfile`),
  `two_paths_differing_only_in_bytes_that_are_not_utf8_never_share_an_identity`; `mod positive`
  `a_file_at_the_top_is_an_app_log_with_no_folder`, `a_deeper_file_is_an_app_log_with_its_folder`
  (`Vendor/Updater/1.0/updater.log` → folder `Vendor/Updater/1.0`), `a_diagnostic_report_is_a_crash_report`
  (also `diagnosticreports/…` and `DiagnosticReports/Retired/x.ips`), `every_state_and_lock_suffix_is_one` (each
  suffix, uppercase too), `a_utf8_path_is_its_own_identity_and_other_bytes_are_escaped`.
- `decide/tests.rs`, negatives first, one test per row, keep before offer: `a_folder_is_not_claimed`,
  `a_link_is_not_claimed`, `a_state_or_lock_file_is_not_claimed`, `a_restored_scan_keeps_everything_even_what_would_be_offered`,
  `a_socket_pipe_or_device_is_special`, `a_file_of_the_system_or_another_user_is_not_yours` (each of the three facts),
  `a_file_the_user_locked_is_locked`, `exactly_the_days_old_is_recent`, `a_future_time_is_recent`,
  `an_unknown_time_is_recent`, `an_old_modification_with_a_fresh_status_change_is_recent`,
  `a_higher_setting_keeps_what_the_default_offers`, and the order tests `the_structural_rows_come_before_restored`,
  `restored_comes_before_special`, `special_comes_before_not_yours`, `not_yours_comes_before_locked`,
  `every_other_row_comes_before_age`; `mod positive` `the_days_and_one_second_is_offered`,
  `an_old_file_is_offered_with_its_whole_days`. `mod property` (proptest):
  `an_offered_file_is_always_older_than_the_days` (any times and days in 7..=90).
- `src/tests.rs` (the module through `on_match` on hand-built `Match`es with a recording sink and the fakes, as
  `modules/temp/src/tests.rs`): `mod negative` `a_match_outside_the_logs_is_not_claimed`,
  `nothing_left_out_is_also_offered`, `a_restored_scan_reports_only_left_outs`; `mod positive`
  `an_app_log_is_offered_in_app_logs_with_its_folder`, `a_top_level_log_reads_your_logs_folder`,
  `a_crash_report_is_offered_in_crash_reports`, `a_finding_moves_the_file_to_the_trash`,
  `a_left_out_is_in_its_place_s_category`, `the_setting_moves_the_threshold`; `mod property`
  `the_verdicts_do_not_depend_on_the_order_of_matches`.
- `deletion-safety.md`: `npm run lint` (Markdown and links) passes.
- `CARGO_TARGET_DIR=/Users/serhiialeksandrov/Desktop/Work/Projects/my/github/fresher/target cargo nextest run -p
  fresher-module-logs`; clippy; fmt; `cargo machete`.
- Commit: `feat(modules): plan 033 — the logs module decides each file`.

## Conclusions

Built (39e3a731, fix c5eab408): `names.rs` (the place, the identity, the state and lock suffixes), `decide.rs` (a pure
function of the rows, in order) and `on_match`. `serde_json` came in as a dev-dependency (the setting test needs
`StoredSettings`). One change from the design, found by the review: refusing every bundle-id-shaped first name also
refused almost every top-level log (`system.log`, `install.log`), since the shape needs only one dot. Row 2 now refuses
such a name only when more names follow, a folder, which is what `app-leftovers` claims. The design (section 3, call 3)
and `deletion-safety.md` say so. 52 tests; `decide.rs` and `names.rs` 100 % of lines.
