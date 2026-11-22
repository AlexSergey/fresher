# 033/02/01 — The case file and the registration

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, fixtures, app, docs

Rule 8 of [`../../../rules.md`](../../../rules.md): the module ships its case file, negatives first, with its golden
report and left-out expectations. The module is registered in the app here: `every_registered_module_has_a_case_file`
in `app/tauri/tests/sandboxes.rs` fails for a registered module without a case file. Model:
`fixtures/sandboxes/temp.toml` (stated ages and facts) and `fixtures/sandboxes/app-caches.toml`; the format is in
[`../../../testing.md`](../../../testing.md), "Sandboxes". The design: [`../00-design.md`](../00-design.md), section 9.

## Files

- `fixtures/sandboxes/logs.toml` (new) — `feature = "logs"`, `module = "logs"`; a head comment: what the file covers
  (old files under `home/Library/Logs`, crash reports apart; files go to the Trash and folders stay), that every old
  entry states `age` and `changed` (30 days unless said; a status change cannot be set back on disk), that positives'
  files use `fill = "data"`, and that the other modules' folders here are not this module's (in the full app
  `jetbrains` and `app-leftovers` decide them). Every path is used by one case only, and every name is apart from every
  other case file's (so `npm run sandbox -- all` leaves out no file it did not leave out before). These exact cases, in
  this order:
  - negative:
    - "a fresh log is kept" — `home/Library/Logs/Notes/notes.log` (`age = "1d"`, `changed = "1d"`); left out
      `logs.left-out.recent` 1 in `logs.app-logs`; `[case.map]` `part = "your-folders"` at `home/Library/Logs/Notes`;
    - "six days old is kept" — `home/Library/Logs/Notes2/six.log` (6 days); left out `recent` 1;
    - "a removed app's log folder is not this module's" — `home/Library/Logs/com.example.Departed/app.log`;
    - "a JetBrains IDE's log folder is not this module's" — `home/Library/Logs/JetBrains/GoLand2024.2/idea.log`;
    - "a link in the logs is never claimed and its target stays" — `[[case.symlink]]` `home/Library/Logs/linked.log` →
      `../../Documents/logs-kept/kept.log`, a file `home/Documents/logs-kept/kept.log`; after the cleanup both
      unchanged;
    - "a socket in a log folder is kept" — `home/Library/Logs/Server/s.sock` (`kind = "socket"`); left out
      `logs.left-out.special` 1;
    - "an app's state and lock files in the logs are never claimed" — `home/Library/Logs/Agent/heartbeat_last.plist`,
      `home/Library/Logs/Agent/run.pid`;
    - "a log of the system is kept" — `home/Library/Logs/Daemon/root.log` (`owner = "system"`); left out
      `logs.left-out.not-yours` 1;
    - "a log you locked is kept" — `home/Library/Logs/Diary/locked.log` (`flags = ["user-lock"]`); left out
      `logs.left-out.locked` 1;
    - "a folder named Logs outside the Library is never claimed" — `home/Documents/Logs/old.log`;
  - positive:
    - "an old log of an installed app goes to the Trash and its folder stays" — `[[case.app]]`
      `com.example.Scribbler` (`Applications/Scribbler.app`, name `Scribbler`); `home/Library/Logs/Scribbler/scribbler.1.log`
      (`size = 65536`, `fill = "data"`); `[case.expect]` item that file, `category = "logs.app-logs"`,
      `danger = "safe"`, `after-cleanup = "in-trash"`; `[case.map]` `part = "explained"` at that file;
    - "an old log with no app behind it goes to the Trash" — `home/Library/Logs/old-tool.log`; `logs.app-logs`, safe,
      `in-trash`;
    - "an old crash report goes to the Trash" — `home/Library/Logs/DiagnosticReports/Scribbler-2026-10-01-101010.ips`;
      `logs.crash-reports`, safe, `in-trash`.
- `fixtures/golden/sandbox-logs.json` (new) — `UPDATE_GOLDEN=1`, reviewed: the findings, the left-out counts, the map,
  the cleanup's outcomes, the tree after (every log folder still there; the three files in the sandbox's Trash; the
  fresh log, the link and its target, the other modules' files unchanged).
- `app/tauri/tests/sandboxes.rs` — `logs => "logs"` in `case_files!`, sorted by file name.
- `app/tauri/Cargo.toml` — `fresher-module-logs = { path = "../../modules/logs" }`.
- `app/tauri/src/compose.rs` — `#[error("the logs module cannot be built: {0}")] Logs(#[from]
  fresher_module_logs::LogsError)` in `ComposeError`; `Box::new(fresher_module_logs::LogsModule::new()?)` in
  `modules()` right after `app-caches`.
- `app/tauri/src/commands.rs` — `"logs"` after `"app-caches"` in `answers_the_modules_and_the_access_status`, and in
  every other expected-id list of the composition tests (`rg '"app-caches"' app/tauri` finds them).
- `docs/testing.md` — `logs.toml` in the list of case files, in the list's order, one line saying what it covers.

What the code does for each case is checked when the file is made; a case that differs from the design is fixed in the
module, not in the case file.

## Produces

The module in every build and its case file, for 02/03's screenshots and 03/01's walk list.

## Checks

- `npm run sandbox:test` passes for `logs.toml` and every other case file (their goldens unchanged).
- `npm run sandbox -- all` leaves out no file it did not already leave out.
- Once per kind, by hand, written in this step's conclusions, each reverted: a positive's `danger` changed `safe` →
  `low` on the crash report case, and the socket case's left-out `items` changed `1` → `2`: each makes the autotest
  fail.
- `npm run sandbox -- logs` writes the sandbox; `EXPECTED.md` lists every case, negatives first, each negative "never
  offered by `logs`".
- `CARGO_TARGET_DIR=/Users/serhiialeksandrov/Desktop/Work/Projects/my/github/fresher/target cargo nextest run -p
  fresher-sandbox -p fresher-app -p fresher-module-logs`.
- Commit: `feat(app): plan 033 — register the logs module with its case file`.

## Conclusions

- `fixtures/sandboxes/logs.toml` has the ten negatives and three positives as specified; `sandbox-logs.json` is
  reviewed (three files moved to the sandbox's Trash, the log folders and every kept file still there). No case
  differed from the design, so the module is unchanged.
- `npm run sandbox:test` passes for every case file, goldens unchanged; `npm run sandbox -- all` leaves out the same
  five files as before and none of `logs`; `npm run sandbox -- logs` writes 13 cases, each negative "never offered by
  `logs`".
- By hand, each reverted: the crash report case's `danger` changed `safe` to `low`, and the socket case's left-out
  `items` changed `1` to `2`: each made the autotest of `logs` fail.
