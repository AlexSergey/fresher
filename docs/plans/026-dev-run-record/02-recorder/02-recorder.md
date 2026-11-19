# 026/02/02 — The recorder in the shell

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** app/tauri

The record itself: its types in the user's shape, a run that begins at the start, folds every engine event into its
scans and cleanups and every action the UI sends into its actions, and the wiring — the composition begins the run,
the app subscribes it to the engine before the last scan is restored, and one wrapper around the generated invoke
handler records the actions. Debug builds only; a release build reads no variable. The design:
[`../00-design.md`](../00-design.md), sections 2.2 and 2.5 to 2.7 (agent's calls 6, 7, 9 to 11).

## Files

- `app/tauri/src/dev_runs/record.rs` (new) — the record's types, `#[derive(Debug, Serialize)]`,
  `#[serde(rename_all = "camelCase")]`, in the shape of the design's section 2.5:
  - `Record { format: u32 (1), session: Session, settings: BTreeMap<SettingKey, SettingValue>, access:
    Vec<CapabilityState>, actions: Vec<ActionRecord>, scans: Vec<ScanRecord>, clear: Vec<ClearRecord> }`;
  - `Session { started: String, build: BuildRecord, mode: String, macos: Option<String>, home: PathBuf, roots:
    Vec<PathBuf>, full_disk_access: Access }`;
  - `ActionRecord { at: String, command: String, args: serde_json::Value }`;
  - `ScanRecord { id: ScanId, kind: Option<ScanKind>, started: String, finished: Option<String>, duration_ms:
    Option<u64>, outcome: Option<ScanOutcome>, coverage: Option<Coverage>, modules: BTreeMap<ModuleId, ModuleRecord>,
    removed: Vec<FindingId> }` (the `Option`s are `null` while the scan runs);
  - `ModuleRecord { group: Option<ModuleGroup>, outcome: Option<ModuleOutcome>, findings: Vec<FindingRecord>, left_out:
    Vec<LeftOutRecord> }`; `FindingRecord { #[serde(flatten)] finding: Finding, title_text: Option<String>,
    reason_text: Option<String> }`; `LeftOutRecord { #[serde(flatten)] count: LeftOutCount, reason_text:
    Option<String> }`;
  - `ClearRecord { id: CleanupId, scan: Option<ScanId>, method: Method, dry_run: bool, ended: Option<Ended>, groups:
    BTreeMap<String, Vec<ClearItem>> }` (the group's kebab-case name, `unknown` when no module claims the category);
  - `ClearItem { module: String, category: String, danger: DangerLevel, path: Option<PathBuf>, sizes: Sizes, action:
    ActionKind, outcome: &'static str, reason: Option<SkipCode> }` and `fn clear_outcome(outcome: &RecordedOutcome,
    action: ActionKind, method: Method) -> (&'static str, Option<SkipCode>)`, the design's table.
  Check each core type used is `Serialize` (`ModuleOutcome`, `ScanOutcome`, `ScanKind`, `Coverage`, `Ended`, `Method`,
  `ActionKind`, `SkipCode`, `CapabilityState`, `Access`); `RecordedOutcome` itself is not written, only its mapping.
- `app/tauri/src/dev_runs/run.rs` (new) — `pub(crate) struct Run { files: RunFiles, record: Record, clock: Arc<dyn
  Clock>, scan_started: Option<SystemTime> }`:
  - `Run::begin(launch: &Launch, build: &Build, home: &Path, macos: Option<String>, clock: Arc<dyn Clock>, engine:
    &Engine) -> Result<Option<Run>, DevRunsRefusal>`: `None` for a release build (no `Mode`); otherwise
    `RunFiles::create` with `clock.now()`, the session (the roots from `engine.settings()`'s `core.scan.roots`,
    `full_disk_access` = `engine.access_status().access(&Capability::AllFiles)`), the settings and access, the first
    snapshot (`rewrite`; its failure here is `DevRunsRefusal::Write`);
  - `Run::action(&mut self, command: &str, args: serde_json::Value)`: appends `{ "at", "command", "args" }` to
    `actions` and as an event line `{ "at", "action": command, "args" }`;
  - `Run::event(&mut self, event: &Event, engine: &Engine)`: `ScanProgress` is ignored; every other event is appended
    as a line `{ "at", "event": <the event as serialised> }`, then folded: `ScanStarted` opens a `ScanRecord` (the
    modules it names, each with its group from `engine.modules()`); `FindingsAdded` adds each finding to its module with
    `english(engine.dictionary(), …)` of its title and reason; `ModuleFinished` sets the outcome and the left-outs (with
    `reason_text`); `ScanCoverage`, `FindingsRemoved` fill the scan; `ScanFinished` closes it (`kind`, `outcome`,
    `finished`, `duration_ms` from `scan_started`) and rewrites the snapshot; `CleanupStarted` opens a `ClearRecord`
    (`scan` = the last scan's id); `CleanupFinished` fills it from `engine.cleanup(id)` (each `CleanupItemView` into its
    group; `dry_run` true when at least one item ends `would-*` and none `moved`, `emptied`, `deleted` or `tool-done`;
    `ended` from the summary) and rewrites the snapshot; `SettingChanged` and `AccessChanged` only refresh `settings`
    and `access` from the engine at the next rewrite. A write error is logged with `tracing::warn!` and dropped.
  - A finding's module and a category's module and group come from `engine.modules()` (`ModuleInfo { id, group,
    categories }`), read once per scan and per cleanup.
- `app/tauri/src/dev_runs/commands.rs` (new) —
  - `pub(crate) const ACTIONS: [&str; 18]` and `pub(crate) const READS: [&str; …]` as the design's section 2.6 lists
    them; `pub(crate) fn is_action(command: &str) -> bool`;
  - `pub(crate) fn observe<R: Runtime>(message: &InvokeMessage<R>)`: `message.webview().try_state::<DevRuns>()`, then
    `runs.call(message.command(), message.payload())`; nothing else.
- `app/tauri/src/dev_runs/mod.rs` — `pub(crate) struct DevRuns(Mutex<Run>)` with `DevRuns::new(run)`,
  `DevRuns::call(&self, command: &str, payload: &InvokeBody)` (only for `is_action`; `InvokeBody::Json(value)` as is,
  `InvokeBody::Raw(_)` as `null`), `DevRuns::event(&self, event: &Event, engine: &Engine)`, and `pub(crate) fn
  follow<R: Runtime>(app: AppHandle<R>, events: Receiver<Event>)`: spawns the thread `fresher-dev-runs` that takes
  `app.try_state::<DevRuns>()` and `app.state::<Engine>()` and folds each event until the receiver ends (a spawn that
  fails is logged; the app runs on). The lock is taken with a poisoning-proof helper, as `events.rs`'s `lock` (moved
  to a shared `pub(crate) fn lock` only if both need it; otherwise a copy of three lines). The module's `expect(dead_code)`
  of 02/01 is removed from `lib.rs`.
- `app/tauri/src/compose.rs` —
  - `ComposeError::DevRuns(#[from] crate::dev_runs::DevRunsRefusal)` under `#[cfg(debug_assertions)]`, message "the
    development run record is refused: {0}";
  - `Start` gains `#[cfg(debug_assertions)] pub(crate) run: Option<crate::dev_runs::Run>`;
  - `start`: in a debug build, `let launch = crate::dev_runs::Launch::from_env(&var)?;` before anything else (a bad
    variable stops the start before any engine is made); after the engine and the build are made (on both paths, the
    sandbox's and the Mac's), `run = launch.map(|launch| Run::begin(&launch, &build, &home, macos, Arc::new(SystemClock),
    &engine)).transpose()?.flatten()` with `macos` from `macos_version(Path::new("/"))` as `"<major>.<minor>"`;
    `var` is read by reference so both readers share it. A release build reads no variable (the existing comment
    holds).
- `app/tauri/src/lib.rs` —
  - `start`: after `compose::start`, in a debug build, when `start.run` is `Some(run)`: `let events =
    engine.subscribe();` **before** `engine.restore()`, `app.manage(dev_runs::DevRuns::new(run))`, and after
    `app.manage(engine)`, `dev_runs::follow(app.handle().clone(), events)`;
  - `run`: `let handler = tauri::generate_handler![…unchanged…];` and `.invoke_handler(move |invoke| {
    #[cfg(debug_assertions)] dev_runs::observe(&invoke.message); handler(invoke) })`.
- `app/tauri/src/dev_runs/tests.rs` (new, `#[cfg(test)] mod tests;` in `mod.rs`) — the shell-level tests below, on
  `crate::testing::Shell` (`Shell::dry_run`, `Shell::cleaning`) and a `tempfile::tempdir()` as the record's folder,
  a `FakeClock` for the run.

## Produces

The working recorder: `npm run dev` with the variables of 02/03 writes a record; 03/01's checker reads the shape this
step writes (`format` 1).

## Checks

Negatives first; every record folder is a temporary folder; no test reads the process environment.

- `dev_runs/tests.rs`:
  - `mod negative`:
    - `a_read_command_is_not_recorded` (`DevRuns::call("get_findings", …)`: no action, no event line);
    - `a_raw_payload_is_recorded_as_null`;
    - `scan_progress_is_never_written` (after a scan, no line of the events file has `"type":"scan-progress"`);
    - `a_snapshot_that_cannot_be_written_is_logged_and_the_run_goes_on` (the folder made read-only after `begin`, a
      scan's events folded: no panic, the events file still grows, the old snapshot is intact; permissions restored at
      the end);
    - `a_release_build_begins_no_run` (`Run::begin` with `Build::Release`: `Ok(None)`, the folder stays empty);
  - `mod positive`:
    - `every_command_of_the_handler_is_an_action_or_a_read` (the names inside `generate_handler![…]` of
      `include_str!("../lib.rs")` equal `ACTIONS` ∪ `READS`, and the two lists share none);
    - `a_run_begins_with_its_session_settings_and_access` (`FakeClock::at(1_763_633_071)`, `Build::Development`, the
      commit `26e4cb0`: the pair `2025-11-20_10-04-31_0.1.0+26e4cb0_mac.{json,events.jsonl}`; the snapshot has `format`
      1, `session.started` `2025-11-20T10:04:31Z`, `build.name`, `mode` `mac`, the home, the roots, `fullDiskAccess`,
      `settings["core.scan.roots"]`, a non-empty `access`) — the version read from `env!("CARGO_PKG_VERSION")`;
    - `actions_are_recorded_with_their_arguments` (`call("update_setting", Json(…))`: `actions[0]` holds the command
      and the args as sent; the events file's last line is the same action);
    - `a_scan_is_recorded_with_its_findings_texts_and_left_outs` (`Shell::dry_run(trial_home(true))`; a scan; every
      event of a subscriber folded: `scans[0]` is `full`, `completed`, with a `durationMs`; its `trial` module's group
      is `debug`, its findings have the paths of `fresher-trial-cache` and `fresher-trial-kept` and non-empty
      `titleText` and `reasonText`; the `demo` module's `leftOut` has its line with `reasonText`);
    - `a_dry_run_cleanup_is_recorded_by_group_as_would_move_and_would_empty` (the same shell, then both trial
      categories prepared and confirmed to the Trash: `clear[0]` has `scan` 1, `method` `trash`, `dryRun` true, `ended`
      `completed`, `groups.debug` with the cache folder `would-move` and the emptied folder `would-empty`; the snapshot
      was rewritten after the cleanup);
    - `a_permanent_dry_run_is_recorded_as_would_delete` (the cache confirmed with `"method": "permanent"`:
      `would-delete`; the folder is still there);
    - `a_sandbox_cleanup_is_recorded_as_moved` (`Shell::cleaning(trial_home(true))`, `Reach::Sandbox`: `moved`,
      `dryRun` false).
- `app/tauri/src/compose.rs`:
  - `mod negative`: `#[cfg(debug_assertions)] a_development_build_without_the_dev_runs_variable_begins_no_run`
    (`start` with a `var` that answers nothing: `run` is `None`); `#[cfg(debug_assertions)]
    a_relative_dev_runs_folder_stops_the_start` (`Err(ComposeError::DevRuns(DevRunsRefusal::NotAbsolute { .. }))`);
    `#[cfg(not(debug_assertions))] a_release_build_ignores_the_dev_runs_variable` (the variables point at a temporary
    folder: `start` succeeds with `Build::Release` and the folder is still empty);
  - `mod positive`: `#[cfg(debug_assertions)] a_development_build_with_the_variable_begins_one_record` (a temporary
    folder: after `start`, exactly one `*_mac.json` and one `*_mac.events.jsonl` in it).
- `cargo nextest run -p fresher-app --lib`; `cargo nextest run --release -p fresher-app --lib` (the release test);
  `cargo clippy --workspace --all-targets -- -D warnings`; `cargo clippy -p fresher-app --release --all-targets -- -D
  warnings`; `cargo fmt --check`. `export CARGO_TARGET_DIR=…/fresher/target`.
- By hand, once: `FRESHER_DEV_RUNS="$PWD/NO_COMMIT/dev-runs" npx tauri dev --config app/tauri/tauri.dev.conf.json`
  (the launcher of 02/03 may not be in yet): scan, quit;
  one pair in `NO_COMMIT/dev-runs/`, its `actions` hold `start_scan`; written in the conclusions with the pair's
  sizes.

## Conclusions

Done 2026-11-19. 151 `fresher-app` lib tests pass (20 new: 13 shell-level, 4 in `dev_runs/record.rs` and
`dev_runs/run.rs`, 3 in `compose`);
clippy (workspace, and `fresher-app --release`) and fmt are clean.
`cargo nextest run --release -p fresher-app --lib` passes `a_release_build_ignores_the_dev_runs_variable`; its 13
other failures are the release-mode failures of `commands` and `shell` tests that existed before this step.

- The shell-level tests are `dev_runs/shell_tests.rs`, not `tests.rs`: `dev_runs/mod.rs` already has the inline
  `mod tests` of 02/01.
- `Record.settings` is `BTreeMap<SettingKey, serde_json::Value>` from `SettingValue::to_json()` (ruling C4); the
  begin test checks `settings["core.scan.roots"]` is a plain array.
- `Session.build` holds the owned `BuildName`; `Run` has a hand-written `Debug` (C8). `Start.run` is
  `Option<dev_runs::run::Run>` (no re-export from `dev_runs`).
- `observe` is `dev_runs::commands::observe`; it uses `InvokeMessage::webview_ref()` (no clone).
- `generate_handler!` cannot be bound with `let` (its closure's argument has no type there): the list moved to
  `fn handler() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static` in `lib.rs`, which `run` wraps.
- `compose::start` begins the run through a small `begin_run` (the `launch.map(…).transpose()?.flatten()` of the
  step, named), on both the sandbox's and the Mac's path; `var` is passed by reference to both readers.
- `FindingsRemoved` fills its scan by id, looked for from the last, so the post-cleanup snapshot holds `removed`.
  A `CleanupFinished` whose start was missed opens its entry then.
- The snapshot is serialised under the run's lock and written (temporary file, sync, rename) after it is released,
  under a writer's lock of its own that skips a snapshot older than the last one given (a sequence number): an
  action on the UI's call path never waits on the disk (`an_action_does_not_wait_for_a_snapshot_being_written`,
  `an_earlier_snapshot_never_replaces_a_later_one`). The event and action lines are typed `Serialize` structs.
- Every `at`, `started`, `finished` and `durationMs` is when the recorder handled the event, not when the engine
  published it: the events of a restored scan are stamped when the recorder's thread reads them, a little later.
- The run by hand (`FRESHER_DEV_RUNS=… npx tauri dev`, scan, quit) moved to step 03/03, the user's run (the
  controller's ruling); this step is done without it.
