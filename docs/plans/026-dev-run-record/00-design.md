# 026 — The development run record: design

**Status:** ✅ approved (the user's decision of 2026-11-19) · **Version:** 0.1.0 · **Date:** 2026-11-19 ·
**Modules:** app, app/tauri, app/ui, core, cleaner, store, scripts, docs

Written on 2026-11-19 from the approved one-file draft (the user approved it with its six agent's calls: "work
autonomously with the recommended decisions") and from a read of the code on `feat/026-dev-run-record` (from `main`
b78d2cd). Where the draft was wrong about the code, the design is corrected and the correction is listed under
"Agent's calls".

## Intent

The user's idea, 2026-11-19 (translated): "We already have a development mode. Without a sandbox it works on the Mac;
in development mode we delete nothing, but we write to `NO_COMMIT` what was found and what could be cleaned (when the
clean button is pressed); also every user action in development mode and the list of settings — one big JSON
(`settings`, `actions`, `clear` by group). Add the build number to every run, and the build number and the date to the
file name, to know when and how it was run, and so that nothing is overwritten."

The runs by hand on the real Mac ([`../../manual-testing.md`](../../manual-testing.md), section 2) then become: start
the development build, scan, press clean, hand over the file. The agent checks every finding and every would-be
removal against the disk with a script instead of the user reading the window line by line.

## Goals

1. A development build (never a release build) records each run to its own file under `NO_COMMIT/dev-runs/`: the
   session (build, macOS, home, roots, Full Disk Access), the settings, the access, every action that reached the
   core, every scan's findings, left-out lines and coverage, and every cleanup, grouped by module group.
2. On the real Mac a confirmed cleanup is a **dry run**: the plan is built and every item goes through every check of
   the cleaner right before its act, as in a release build; the act itself is never made and the item ends "dry run"
   instead. In sandbox mode the cleanup is real, as today, and recorded too.
3. A record is never overwritten: its name carries the date and time, the build and the mode.
4. A checker script reads a record and checks each finding and each would-be removal against the disk, read-only, and
   writes the differences next to the record.

## Non-goals

- Anything in a release build: the opt-in session log of the deferred item
  [session-log-file](../../deferred/session-log-file.md) may reuse the writer later, with its own decisions (privacy
  text, location, rotation).
- Recording clicks that never reach the core (a checkbox, a scroll, a page): the record is of what the core was asked
  to do and what it answered.
- Sending a record anywhere: it stays in `NO_COMMIT/`, which git ignores.
- The debug panel and the developer command line of [dev-debug-panel](../../deferred/dev-debug-panel.md): the dry run
  answers its "print a dry run" on the real Mac; the rest stays deferred.
- The projects the walk found as their own list (agent's call 7) and the paths of left-out items (the core keeps only
  their counts).

## What already exists

| Need                                 | What exists                                                                                                         | Plan     |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------- | -------- |
| The build's kind                     | `fresher_core::Build` (`Release`, `Development`, `Sandbox { path, simulated }`), made by `compose::start` (`own_build`, `sandbox::build`), managed by the app; `AppInfo { version, build }` | 007      |
| The development reach                | `compose::reach()`: `Reach::RealDisk(RealDiskToken::new())` in a release build, `Reach::Sandbox` in a debug build or a test; `RealDiskToken::new` exists only under `cfg(not(debug_assertions))`; `crates/cleaner/tests/reach.rs` fails when the token is named outside `reach.rs`, `lib.rs` and `app/tauri/src/compose.rs` | 008      |
| What a debug build does on the Mac   | check 1 (`Reach::admits`) skips every item outside a marked folder with `SkipReason::NotInSandbox`, already in `prepare_cleanup`'s rehearsal: the plan of a real-Mac selection is empty (`commands.rs`'s `a_debug_build_cleans_nothing_outside_a_sandbox`) | 008      |
| Every act of the cleaner             | `pipeline::act_on`: `verify_tool` then `run_tool`; `verify(item, guard, Depth::Full)` then `empty` (Trash or permanent), `delete` (permanent, `std::fs` at the single deletion point) or `to_trash` (the `Trash` port) | 008      |
| Put back                             | `put_back::put_back`: `Reach::admits`, then the `Trash` port's `put_back`; the engine answers `PutBackOutcome::NotTrashed` for an item that never went to the Trash | 008      |
| The cleanup's detail                 | `Engine::cleanup(id) -> CleanupDetail { summary: CleanupSummary { method, ended, … }, items: Vec<CleanupItemView { index, path, category, danger, sizes, action, outcome: RecordedOutcome, … }> }`, read from the journal, which is ended before `Event::CleanupFinished` is published (`crates/engine/src/cleanup/operation.rs`) | 008      |
| The engine's events                  | `Engine::subscribe()` gives every subscriber its own receiver (`bus.rs`); `Event::{ScanStarted, FindingsAdded, ModuleFinished { left_out: Vec<LeftOutCount> }, ScanCoverage, ScanFinished { outcome, kind }, FindingsRemoved, SettingChanged, AccessChanged, CleanupStarted, CleanupItems, CleanupFinished, …}`, all `Serialize` | 003, 011 |
| The commands                         | `app/tauri/src/{commands,settings_commands,storage_commands}.rs`, listed in `tauri::generate_handler![…]` in `lib.rs`; the builder's `invoke_handler` takes any `Fn(tauri::ipc::Invoke<R>) -> bool`, and `InvokeMessage` gives `command()`, `payload()` (`InvokeBody::Json`) and `webview()` | 003      |
| Serialisation of what the UI sees    | every type the UI gets is `serde::Serialize` with `ts-rs` bindings: `Finding`, `LeftOutCount`, `Coverage`, `SettingsView { values }`, `AccessStatus`, `ModuleInfo { id, group, categories }`, `CleanupDetail`; the golden reports are `serde_json` of the same types | 002–021  |
| Texts                                | `Text { key, args }`, `TextArg::{Text, Count, Size, Date, Path}`; `Engine::dictionary()` (`Dictionary::get(key)`, `{name}` placeholders). No renderer outside tests (`crates/core/src/texts.rs`'s test helper `render`) | 002, 011 |
| The launcher                         | `npm run dev` = `tauri dev --config app/tauri/tauri.dev.conf.json`; `npm run dev:sandbox` = `scripts/sandbox-dev.ts`, which sets `FRESHER_SANDBOX` and runs `npm run dev` (tests in `scripts/sandbox-dev.test.ts`, `node --test`) | 007      |
| The macOS version                    | `fresher_system::macos_version(Path::new("/")) -> Option<MacosVersion { major, minor }>`                            | 019      |
| The ignored folder                   | `NO_COMMIT/` and `/sandbox/` are in `.gitignore`                                                                    | 001, 007 |

## Design

### 1. The dry run (plan 01)

#### 1.1 Why not only a Trash port

The draft gave the cleaner a recording `Trash` port on the real Mac. The code says it cannot be enough:

- a debug build's cleaner is `Reach::Sandbox`, so on the real Mac check 1 skips every item, already in the rehearsal
  that makes the plan: the plan is empty and the Trash is never reached;
- the `Trash` port is one act of four: a permanent delete calls `std::fs` at the single deletion point, an emptied
  folder deletes its entries when the method is permanent, and a tool item runs its program through `CommandRunner`.

So the dry run is a **reach** of the cleaner, and the act is stopped in the one place that makes every act.

#### 1.2 `Reach::DryRun`

In `crates/cleaner/src/reach.rs`, compiled in debug builds only:

```rust
pub enum Reach {
    Sandbox,
    SandboxFolder(SandboxFolder),
    /// Anywhere the other checks allow, acting on nothing: a development build on the real Mac.
    #[cfg(debug_assertions)]
    DryRun(DryRunToken),
    RealDisk(RealDiskToken),
}

/// The licence to check the real disk without acting on it. Only a debug build can make one, and only the app's
/// composition and the cleaner's tests name it (checked by `tests/reach.rs`).
#[cfg(debug_assertions)]
#[derive(Debug)]
pub struct DryRunToken(());

#[cfg(debug_assertions)]
impl DryRunToken {
    pub const fn new() -> Self { Self(()) }
}
```

- `Reach::admits`, `admits_tools` and `admits_unplaced` answer for `DryRun` as for `RealDisk`: every later check
  (roots, protection, whole folders, ignore, links, volume, identity, flags, access, in use, growth) runs on the real
  Mac exactly as a release build runs it.
- A new `pub(crate) const fn acts(&self) -> bool`: `false` for `DryRun`, `true` for the others.
- `pipeline::act_on`: after `verify_tool` passes, and after `verify(item, guard, Depth::Full)` passes, when
  `!guard.reach.acts()` the item ends `Removed::ended(ItemOutcome::Skipped { reason: SkipReason::DryRun })`;
  `run_tool`, `empty`, `delete` and `to_trash` are never called. The `Verified` is dropped, so nothing is held.
- `put_back::put_back`: when `!cleaning.reach.acts()`, `PutBackOutcome::Refused { reason: SkipCode::DryRun }` before
  any port is called.
- A release build has no variant, no token and no branch: `#[cfg(debug_assertions)]` on each; the `match` arms of
  `admits*` and `acts` for `DryRun` are cfg'd the same way.

The cancel token, the snapshots of open files and running apps every two seconds and the order of the items are the
pipeline's own and unchanged, so a dry run takes the time of the checks of a real cleanup without its acts.

#### 1.3 The outcome: `SkipReason::DryRun`

The core gains one skip reason (the only core change of this plan):

- `SkipReason::DryRun` and `SkipCode::DryRun` (`crates/core/src/cleanup/outcome.rs`): code `dry-run`, in
  `SkipCode::ALL` (20 codes), no `how`; text `core.cleanup.skip.dry-run` in `crates/core/locales/en.json`:
  "Development build: a dry run. It passed every check and was left where it is."
- The journal's mirror `SkipCodeForm` (`crates/store/src/journal/records.rs`) gains `DryRun`.
- The IPC bindings are regenerated (`app/ui/src/ipc/bindings.generated.ts`: `SkipCode` gains `"dry-run"`); the UI
  shows skip reasons by their core text and needs no other change.

So the window, the history and the journal tell the truth: every item of a dry run is "skipped: a dry run", nothing
was moved, the freed space is zero, and put back answers `NotTrashed` (the engine's answer for an item that never went
to the Trash) — the draft's "put back of a dry run answers nothing was moved".

#### 1.4 The composition

`app/tauri/src/compose.rs`:

| Build                          | `reach()`                                  | The cleaner's `Trash`                     |
| ------------------------------ | ------------------------------------------ | ----------------------------------------- |
| release (`not(debug_assertions)`, not a test) | `Reach::RealDisk(RealDiskToken::new())` | `MacTrash`                              |
| debug (the app outside a sandbox; `Shell::dry_run` in the shell's tests) | `Reach::DryRun(DryRunToken::new())` | `DryRunTrash` (below)                   |
| a release-mode test (`cargo test --release`) | `Reach::Sandbox`                     | `MacTrash` (no test cleans through it) |
| sandbox mode (debug, `FRESHER_SANDBOX`) | `Reach::SandboxFolder` (unchanged, `sandbox.rs`) | the sandbox's `home/.Trash` or `MacTrash` (unchanged) |

- `compose::cleanup(home, user_temp, folder, ports, volumes)` gains `reach: Reach` as its last argument; `normal`
  passes `reach()`; the shell's test helper passes its own.
- `app/tauri/src/dry_run.rs` (new, `#[cfg(debug_assertions)] mod dry_run;` in `lib.rs`): `pub(crate) struct
  DryRunTrash;` implementing `fresher_core::Trash`: `move_to_trash` and `put_back` log `tracing::error!` ("a dry run
  reached the Trash") and answer `Err(TrashError::NotAllowed)`; they touch nothing. It is a second wall, never reached
  while 1.2 holds: a test proves a cleanup with it moves nothing even when the reach would act.
- `normal` gives the cleaner `DryRunTrash` in a debug build and `MacTrash` in a release build (`#[cfg]` on the one
  expression).

The manual way of `docs/architecture/removal.md` ("To clean in a development build… make a folder of its own… write
the signature") is gone: a development build on the real Mac never moves anything, a marked folder included. Real
cleanups in a development build happen only in sandbox mode (`npm run dev:sandbox`), which is unchanged.

### 2. The record (plan 02)

#### 2.1 Where it lives

`app/tauri/src/dev_runs/` (new, `#[cfg(debug_assertions)] mod dev_runs;` in `lib.rs`): nothing of it is compiled in a
release build. The core, the engine and the modules gain nothing.

| File          | What                                                                                                              |
| ------------- | ----------------------------------------------------------------------------------------------------------------- |
| `mod.rs`      | `Launch` (what the environment asks for), `DevRunsRefusal`, `DevRuns` (the managed recorder), `start`             |
| `files.rs`    | the file name, creating the pair new, appending an event line, rewriting the snapshot atomically                  |
| `clock.rs`    | UTC date and time of a `SystemTime`, for the name and the record (no new dependency)                              |
| `english.rs`  | a `Text` in English from the engine's dictionary                                                                  |
| `run.rs`      | `Run`: begins the record, folds actions and events into it, rewrites the snapshot                                 |
| `record.rs`   | the record's types (`serde::Serialize`) and how an event or an action folds into them                             |
| `commands.rs` | which commands are actions, and the invoke hook                                                                   |
| `tests.rs`    | the shell-level tests                                                                                             |

#### 2.2 Turned on by the launcher only

The recorder starts only when the environment names its folder:

| Variable               | Set by           | Value                                                                    |
| ---------------------- | ---------------- | ------------------------------------------------------------------------ |
| `FRESHER_DEV_RUNS`     | `scripts/dev.ts` | `<repository>/NO_COMMIT/dev-runs`, absolute                              |
| `FRESHER_BUILD_COMMIT` | `scripts/dev.ts` | `git rev-parse --short HEAD`                                             |
| `FRESHER_BUILD_DIRTY`  | `scripts/dev.ts` | `1` when `git status --porcelain` prints anything, else `0`              |

- `Launch::from_env(var: impl Fn(&str) -> Option<OsString>) -> Result<Option<Launch>, DevRunsRefusal>`: `None` when
  `FRESHER_DEV_RUNS` is unset (a bare `tauri dev`, every test: nothing is written); refused when it is not absolute.
  The commit is kept when it is 4 to 40 lowercase hex digits, else `unknown`; dirty is `FRESHER_BUILD_DIRTY == "1"`.
- `compose::start` reads it in a debug build only (as it reads `FRESHER_SANDBOX`) and begins the run (2.4) before it
  returns; `Start` gains `#[cfg(debug_assertions)] pub(crate) run: Option<dev_runs::Run>`. A release build reads no
  variable (the test `a_release_build_ignores_the_dev_runs_variable`, run by `cargo nextest run --release`).
- A refused folder stops the start, as a refused sandbox does (`ComposeError::DevRuns(DevRunsRefusal)`, shown by
  `start_failure`): a run the user thinks is recorded is never silently lost. Refused: not absolute; cannot be
  created (`fs::create_dir_all`); after creation not a folder or a link (`symlink_metadata`); the pair cannot be
  created.
- After the start, a write that fails is logged (`tracing::warn!`) and the app runs on; the next write tries again.

#### 2.3 The file name

`<YYYY-MM-DD_HH-MM-SS>_<version>+<commit>[-dirty]_<mode>[-<n>].json` and beside it the same name with `.events.jsonl`
in place of `.json`:

- the date and time are the run's start in UTC (agent's call 8), from the composition's clock (`SystemClock`; a
  `FakeClock` in tests);
- `<version>` is `env!("CARGO_PKG_VERSION")`, `<commit>` and `-dirty` from 2.2;
- `<mode>` is `mac` (`Build::Development`) or `sandbox-<name>` (`Build::Sandbox { path }`: the folder's last component,
  every character other than ASCII letters, digits, `-` and `_` written `-`);
- both files are created with `OpenOptions::create_new(true)`; when either exists the pair is tried again with `-2`,
  `-3`, … up to `-99` before `.json` (then refused): a record is never opened for writing if it exists;
- example: `2026-11-20_10-04-31_0.1.0+26e4cb0-dirty_mac.json`.

`fn file_name(started: SystemTime, build: &BuildName, mode: &Mode, attempt: u32) -> String` is pure and tested.

#### 2.4 Writing

- **Events**: one JSON object per line, appended (`OpenOptions::append`) as each happens, flushed: a crash loses at
  most the line being written. Every action (2.6) and every engine event except `scan-progress` (throttled progress,
  thousands of lines), in the order they reached the recorder.
- **Snapshot**: the whole record (2.5), written to a `tempfile::NamedTempFile` in the same folder (prefix
  `.<name>.`, suffix `.tmp`), synced, then `persist`ed over this run's own `.json` — a reader sees the old snapshot or
  the new one. Rewritten when the run begins, after every `scan-finished` and after every `cleanup-finished`.
- Nothing else is ever written, and nothing outside the folder of 2.2.

#### 2.5 The shape

`serde_json` of the core's own types, as the UI and the golden reports get them, plus the English texts:

```jsonc
{
  "format": 1,
  "session": {
    "started": "2026-11-20T10:04:31Z",
    "build": { "version": "0.1.0", "commit": "26e4cb0", "dirty": true, "name": "0.1.0+26e4cb0-dirty" },
    "mode": "mac",                       // or "sandbox-<name>"
    "macos": "27.2",                     // null when unread
    "home": "/Users/…",
    "roots": ["~"],                      // core.scan.roots as the settings hold them
    "fullDiskAccess": "granted"          // the Access of Capability::AllFiles, as the core serialises it
  },
  "settings": { "core.scan.roots": ["~"], "stale-projects.unused-for-days": 365 },   // SettingsView.values
  "access": [ /* AccessStatus.capabilities, as the UI gets them */ ],
  "actions": [
    { "at": "2026-11-20T10:04:40Z", "command": "update_setting",
      "args": { "update": { "key": "stale-projects.unused-for-days", "value": { … } } } },
    { "at": "…", "command": "start_scan", "args": {} },
    { "at": "…", "command": "confirm_cleanup", "args": { "confirmation": { … } } }
  ],
  "scans": [
    {
      "id": 1, "kind": { "kind": "full" }, "fallback": "no-cache", "started": "…", "finished": "…", "durationMs": 41230,
      "outcome": { "kind": "completed" },
      "coverage": { /* Coverage */ },
      "modules": {
        "stale-projects": {
          "group": "development",
          "outcome": { "kind": "completed" },
          "findings": [ { /* Finding as the UI gets it */ "titleText": "…", "reasonText": "…" } ],
          "leftOut": [ { /* LeftOutCount */ "reasonText": "…" } ]
        }
      },
      "removed": [ 12, 13 ]              // finding ids of findings-removed
    }
  ],
  "clear": [
    {
      "id": 1, "scan": 1, "method": "trash", "dryRun": true, "ended": "completed",
      "groups": {
        "development": [
          { "module": "stale-projects", "category": "stale-projects.dependencies", "danger": "low",
            "path": "/Users/…/old-app/node_modules", "sizes": { … }, "action": "remove",
            "outcome": "would-move", "reason": null }
        ]
      }
    }
  ]
}
```

- `session.started` and every `at`, `started`, `finished` are UTC, `YYYY-MM-DDTHH:MM:SSZ`; `durationMs` is the
  clock's difference between `scan-started` and `scan-finished` reaching the recorder.
- `settings` and `access` are read again (`Engine::settings()`, `Engine::access_status()`) at every snapshot.
- A scan is opened by `scan-started` and filled by `findings-added` (per module: the finding as serialised, plus
  `titleText` and `reasonText`), `module-finished` (`outcome`, `leftOut` with `reasonText`), `scan-coverage`,
  `findings-removed` and closed by `scan-finished` (`kind`, `outcome`, and `fallback` from `Engine::fallback`: why the
  walk read every folder although a rescan was asked, `null` when it did not; a failed check gives
  `{ "check-failed": { "folder", "difference", "folders" } }` with the first differing folder inside its root). A module's group comes from
  `Engine::modules()` (`ModuleInfo.group`). A restored scan (`ScanKind::Restored`) is a scan like any other.
- A cleanup is opened by `cleanup-started` (`scan` = the id of the last scan in the record, `method`) and filled at
  `cleanup-finished` from `Engine::cleanup(id)`: each `CleanupItemView` becomes an item of its module's group (the
  module and the group from the `ModuleInfo` whose categories hold the item's category; `"unknown"` when none does);
  `dryRun` is true when at least one item ended `would-*` and none `moved`, `emptied`, `deleted` or
  `tool-done`.
- An item's `outcome`, from its `RecordedOutcome`, its `ActionKind` and the cleanup's `Method`:

  | `RecordedOutcome`                  | `action` | `method`    | `outcome`        | `reason`        |
  | ---------------------------------- | -------- | ----------- | ---------------- | --------------- |
  | `Skipped { reason: DryRun }`       | `remove` | `trash`     | `would-move`     | `null`          |
  | `Skipped { reason: DryRun }`       | `remove` | `permanent` | `would-delete`   | `null`          |
  | `Skipped { reason: DryRun }`       | `empty`  | any         | `would-empty`    | `null`          |
  | `Skipped { reason: DryRun }`       | `tool`   | any         | `would-run`      | `null`          |
  | `Skipped { reason }` (any other)   | any      | any         | `skipped`        | the skip code   |
  | `Moved`, `Emptied`, `Deleted`, `ToolDone`, `Failed`, `NotReached`, `Unknown` | any | any | `moved`, `emptied`, `deleted`, `tool-done`, `failed`, `not-reached`, `unknown` | `null` |

- `english(dictionary, text) -> Option<String>`: the template of `text.key()` with every `{name}` replaced: `Text` as
  it is, `Count` as its number, `Size` as bytes (`1234 bytes`), `Date` as the UTC date `YYYY-MM-DD`, `Path` as its
  `display()`; `None` for a key the dictionary lacks (the field is then `null`).

#### 2.6 Actions: at the command layer

Every call from the UI passes one point: the builder's `invoke_handler`. `lib.rs` wraps the generated handler:

```rust
let handler = tauri::generate_handler![ /* unchanged */ ];
… .invoke_handler(move |invoke| {
    #[cfg(debug_assertions)]
    dev_runs::observe(&invoke.message);
    handler(invoke)
})
```

- `observe` looks up the managed `DevRuns` (`message.webview().try_state::<DevRuns>()`); none (no
  `FRESHER_DEV_RUNS`): nothing. Otherwise, when `is_action(message.command())`, it appends `{ at, command, args }`
  with the JSON payload (`InvokeBody::Json`; a raw body is written `null`).
- The actions are the commands that change something or reach outside the app: `start_scan`, `cancel_scan`,
  `update_setting`, `reset_settings`, `ignore`, `restore_ignored`, `refresh_access`, `open_access_settings`,
  `reveal_finding`, `relaunch`, `prepare_cleanup`, `discard_cleanup`, `confirm_cleanup`, `cancel_cleanup`, `put_back`,
  `reveal_in_trash`, `quit_after_cleanup`, `pick_folder`. The rest only read (`app_info`, `subscribe`, `get_*`).
  `ACTIONS` and `READS` are two constants; a test reads `lib.rs` and fails when a command of the handler is in
  neither, so a new command is classified when it is added.
- The action is written before the command runs: it is what the user asked; what happened follows in the events.
- No command's signature changes, so no command's tests change.

#### 2.7 The recorder's thread

`lib.rs`'s `start`, in a debug build, when `Start.run` is `Some(run)`: subscribes to the engine **before**
`engine.restore()` (so the restored scan is recorded), manages `DevRuns`, and spawns the thread `fresher-dev-runs`,
which folds every event into the run (`Run::event(&mut self, event: &Event, engine: &Engine)`) with the engine taken
from the app (`app.state::<Engine>()`). `DevRuns` holds the run behind a `Mutex` (the lock helper of `events.rs`, which
survives poisoning), shared by the thread and `observe`. The thread ends when the engine's events end.

### 3. The launcher (plan 02)

- `package.json`: `"dev": "node scripts/dev.ts"`; the old command is not kept as a script: the launcher
  calls the tauri CLI directly with fixed arguments (`npm exec -- tauri dev --config app/tauri/tauri.dev.conf.json`). `scripts/sandbox-dev.ts` keeps running `npm run dev` with `FRESHER_SANDBOX`, so a
  sandbox run goes through the launcher too and is recorded as `sandbox-<name>`.
- `scripts/dev.ts`, in the shape of `sandbox-dev.ts` (pure functions exported, a `Spawner`, `import.meta.main`):
  `buildFacts(git: Git): { commit: string; dirty: boolean }` (a failing `git` gives `unknown` and clean),
  `devCommand(root: string, facts): Command` (`npm exec -- tauri dev --config ...` with the three variables, no argument), `run` as in
  `sandbox-dev.ts`. It creates nothing; the app creates the folder.

### 4. The checker (plan 03)

`npm run dev-runs:check -- <record.json>` (`scripts/dev-runs-check.ts`, tests in `scripts/dev-runs-check.test.ts`),
read-only on the disk: it never writes but its report.

- **Reads** the record, refuses one whose `format` is not `1` or that lacks `session`, `scans` or `clear` (exit 2).
- **Checks every finding of every scan whose subject is a path, and every `clear` item with a path**, against the disk
  through an injected `Disk` (`lstat`, `readdir`; the real one is `node:fs`, never following a link):

  | Check                | Holds when                                                                                                      |
  | -------------------- | --------------------------------------------------------------------------------------------------------------- |
  | `gone`               | a finding or a `would-*`/`skipped` item still exists; a `moved`/`deleted` item does not                         |
  | `link`               | it is no link (`lstat`)                                                                                         |
  | `kind`               | it is a file or a folder                                                                                        |
  | `size`               | its allocated size (`blocks × 512`, walked with `lstat`, a hard-linked file counted once by device and inode) is within 5 % or 1 MiB of the record's `allocated`, when the record's completeness is `complete` |
  | `outside`            | it is strictly inside a root of `session.roots` (`~` read as `session.home`) or inside `/private/var/folders` or `/private/tmp` |
  | `protected`          | it is not under `/System`, `/usr` (but `/usr/local`), `/bin`, `/sbin`, `/private/var/db`, and is not the home or one of its standard folders (`Desktop`, `Documents`, `Downloads`, `Movies`, `Music`, `Pictures`, `Public`, `Library`) itself |
  | `stale-projects`     | for its findings and items: the folder's name is one of `node_modules`, `target`, `build`, `.gradle`, `.build`, `.dart_tool`, `.venv`, `venv`, or it is a cache (`incremental` below a `target`, `target/llvm-cov-target`); no component is `.git`, `src` or `dist`; not under `<home>/Library`, a `*.app`, or a hidden folder directly under the home; for an artifact, its project (the folder's parent) has no folder outside the artifact names and `.git` (recursive, links not followed) and no folder of `.git/refs/heads` newer than the scan's `finished` less `settings["stale-projects.unused-for-days"]` days; for a cache, no folder of it newer than the scan less 7 days |
  | `downloads`          | its parent is `<home>/Downloads`                                                                               |
  | `temp`               | its parent is `/private/tmp` or a folder `/private/var/folders/<xx>/<hash>/T`, and nothing inside it changed (`mtime` or `ctime`) after the scan less `settings["temp.older-than-days"]` days |

  A time after the scan's `finished` is reported as `changed-after-scan`, not as a failed rule. The scan's time, never
  the wall clock, is "now", so the checker answers the same on the same disk.
- **Writes** `<record>.check.md` (created with `flag: 'wx'`; when it exists `<record>.check-2.md`, …): the record's
  name, build and mode; the counts (findings checked, items checked, differences); then one table row per difference
  (`path`, `module`, `category`, `check`, `expected`, `found`), or "No differences." Exit 0 with no difference, 1 with
  one or more.
- Pure functions exported (`parseRecord`, `checkRecord(record, disk): Difference[]`, `renderReport`), the main block
  under `import.meta.main`, as `sandbox-dev.ts`. The agent runs it on the user's record; the user does not need to.

### 5. Tests

Negatives first, everywhere. The safety tests carry the plan:

| What is proved                                                     | Where                                                                                         |
| ------------------------------------------------------------------ | --------------------------------------------------------------------------------------------- |
| a dry run moves, empties, deletes and runs nothing, on any method  | `crates/cleaner/src/pipeline.rs` (unit), `crates/cleaner/tests/never.rs` (property, random trees) |
| every check still runs in a dry run                                | `pipeline.rs`: a link, an ignored path, an item in use, a path outside the roots skip with their own reasons, never `dry-run` |
| put back in a dry run calls no port                                | `crates/cleaner/src/put_back.rs`                                                              |
| the token is made only by the composition and the cleaner's tests  | `crates/cleaner/tests/reach.rs`                                                               |
| the development engine on the real disk moves nothing             | `app/tauri/src/compose.rs`: `compose::engine` (real ports, `DryRunTrash`) on a temporary home with the trial module's marked folder, prepared, confirmed: every file stays, every item ends `dry-run` |
| the dry-run Trash refuses and touches nothing                      | `app/tauri/src/dry_run.rs`                                                                    |
| a release build has no recorder and reads no variable              | `compose.rs`'s `#[cfg(not(debug_assertions))]` test, run by `cargo nextest run --release -p fresher-app`; `cargo clippy --release` builds without the cfg'd code |
| nothing is written without `FRESHER_DEV_RUNS`                      | `compose.rs` and `dev_runs/tests.rs`                                                          |
| a record is never overwritten                                      | `dev_runs/files.rs`                                                                           |
| tests write only to temporary folders                              | every test passes a `var` closure and a `tempfile::tempdir()`; none reads the process environment |

### 6. Where the code lives

| Plan                  | Files                                                                                                     |
| --------------------- | --------------------------------------------------------------------------------------------------------- |
| 01 The dry run        | `crates/core/src/cleanup/outcome.rs`, `crates/core/locales/en.json`, `crates/store/src/journal/records.rs`, `crates/cleaner/src/{reach,pipeline,put_back,lib,test_support}.rs`, `crates/cleaner/tests/{reach,never}.rs`, `app/tauri/src/{compose,dry_run,lib,testing,commands}.rs`, `app/ui/src/ipc/bindings.generated.ts` (regenerated), `docs/architecture/{README,removal}.md`, `docs/restrictions/deletion-safety.md` |
| 02 The recorder       | `app/tauri/src/dev_runs/`, `app/tauri/src/{compose,lib}.rs`, `app/tauri/Cargo.toml`, `scripts/dev.ts`, `scripts/dev.test.ts`, `scripts/sandbox-dev.ts`, `package.json`, `docs/testing.md`, `docs/dependencies.md`, `docs/restrictions/privacy.md`, `docs/architecture/README.md`, `docs/deferred/` |
| 03 Checker and finish | `scripts/dev-runs-check.ts`, `scripts/dev-runs-check.test.ts`, `package.json`, `docs/manual-testing.md`, `docs/testing.md`, `docs/plans/` |

## Interfaces with other plans

- Plan 007 (sandbox mode) is unchanged; its runs are recorded as `sandbox-<name>`.
- Plan 008 (the cleaner): one reach and one skip reason more; the threat model gains the dry run.
- The deferred [session log file](../../deferred/session-log-file.md) may reuse `dev_runs/files.rs` (a new file per
  run, an append-only event file, an atomic snapshot) and `english.rs`; its privacy decisions stay its own.
- The deferred [debug panel and developer command line](../../deferred/dev-debug-panel.md): "print a dry run" is
  answered on the real Mac by the dry run and the record.

## Open questions

None.

## Decisions

The user's, of 2026-11-19 (translated):

1. The idea of the intent above: a development build deletes nothing on the real Mac and writes what was found and
   what could be cleaned, every action and the settings, to one JSON file under `NO_COMMIT`, with `clear` by group.
2. "Add the build number to every run, and include the build number and the date in the file name, so we know when
   and how it was run and nothing is overwritten."
3. "Work autonomously with the recommended decisions": the draft's six agent's calls became these decisions:
   1. **The build number is `<version>+<commit>[-dirty]`**: the version alone does not change while the app is built.
   2. **Two files per run**: an append-only `.events.jsonl` (safe against a crash) and the snapshot `.json` in the
      user's shape.
   3. **Only actions that reach the core are recorded**, not clicks.
   4. **The dry run happens only on the real Mac**; a sandbox cleans for real, as today, and is recorded.
   5. **Development builds only, and only with `FRESHER_DEV_RUNS` set by the launcher.**
   6. **The checker is the agent's tool**, read-only, writing a report next to the record.
4. **No version bump**: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Made while writing this design from the code; each with what it costs if wrong.

1. **The dry run is a reach of the cleaner, `Reach::DryRun`, not only a recording Trash** (the draft's mechanism
   corrected): a debug build's `Reach::Sandbox` skips every real-Mac item already when the plan is made, and the Trash
   is one act of four (permanent delete, emptying, tools). The act is stopped in `act_on`, the one place every act
   passes, after every check. Cost if wrong: none for safety; one enum variant more in the cleaner.
2. **A refusing Trash stays as a second wall** (`DryRunTrash`, the draft's "recording Trash" kept as a port that
   refuses and logs): it is never reached while call 1 holds. Cost if wrong: a few lines.
3. **The dry run's outcome is a skip reason, `SkipReason::DryRun`** (not in the draft, which said the core gains
   nothing but serialisation): the window, the history and the journal then say "a dry run" and count nothing as
   freed, and put back answers "never went to the Trash". Cost if wrong: one variant in the core, the journal's
   mirror and the bindings.
4. **A development build on the real Mac never moves anything, a marked folder included**: the manual way of cleaning
   a marked folder in `npm run dev` is removed; real cleanups in development happen only in sandbox mode. Cost if
   wrong: a developer who used a marked folder by hand uses `npm run dev:sandbox -- <path>` instead.
5. **The plan order is dry run, recorder, checker** (the draft had the recorder first): the record's `clear` reads
   the dry run's outcome, and the safety comes before the file that relies on it. Cost if wrong: none.
6. **Actions are recorded at the IPC dispatcher**, by wrapping the generated invoke handler, as `{ at, command, args }`
   with the arguments the UI sent, before the command runs (the draft's `kind`/`from`/`to` per action): no command's
   signature or test changes, and every command is classified as an action or a read by a test. The value a setting
   had before is in the previous snapshot or the previous `setting-changed` event. Cost if wrong: a refused command is
   recorded as asked; the events show it did nothing.
7. **The record has no `projects` list** (the draft had one): the engine keeps no projects after a scan (they live in
   the scan's `SystemFacts`), and exposing them is an engine change this plan does not need; the stale-project
   findings name their project in the path and the reason, and the checker reads the project from the disk. Cost if
   wrong: an engine accessor and a serialisation of `Project` later.
8. **Every time is UTC**, in the file name too (`…_10-04-31_…` is UTC, the record says `Z`): no time-zone data without
   a new dependency, and a name that sorts the same on any Mac. Cost if wrong: the name's hour differs from the wall
   clock by the time zone.
9. **A bad `FRESHER_DEV_RUNS` stops the start**, as a bad `FRESHER_SANDBOX` does; a write that fails after the start
   is logged and the app runs on. Cost if wrong: a broken folder needs fixing before a run.
10. **The snapshot is rewritten at the start, after every scan and after every cleanup**; the events file gets every
    action and every engine event but `scan-progress`. Cost if wrong: a snapshot older than the last setting change
    until the next scan (the events file has it).
11. **Left-outs are recorded as the core's counts per category and reason** (`LeftOutCount`, with the reason's text):
    the core keeps no path of a left-out item. Cost if wrong: an engine change to keep them.
12. **The checker's size tolerance is 5 % or 1 MiB, on the allocated size, for complete sizes only**: the disk moves
    between the scan and the check. Cost if wrong: a few false differences, or a real one missed under 1 MiB.
13. **The checker knows the rules of three modules** (stale projects, downloads, temporary files) besides the
    checks of every item; the other modules' rules need the installed apps, Spotlight or a tool's signature and stay
    with the user's eyes (the finish lists them). Cost if wrong: more rows by hand.
14. **The launcher is `scripts/dev.ts` behind `npm run dev`**, there is no `dev:app` (the launcher calls the tauri CLI directly), and
    `dev:sandbox` keeps calling `npm run dev`, so both are recorded. Cost if wrong: one script name.
15. **The commit is 4 to 40 lowercase hex digits or `unknown`; dirty is `FRESHER_BUILD_DIRTY=1`**: a name never holds
    what the environment slipped in. Cost if wrong: none.
16. **The development banner keeps its text** ("nothing can be removed outside a sandbox" stays true); the dry run
    shows in each item's skip reason. Cost if wrong: one UI string later.
17. **The recorder writes its snapshot with `tempfile`'s `persist`, not the store's atomic write**: the store touches
    only the app's data folder, and `tempfile` is already a normal dependency of the app through `fresher-testing`.
    Cost if wrong: none.
18. **A checker report is never overwritten either** (`.check.md`, then `.check-2.md`, …). Cost if wrong: none.

## Conclusions

Not written yet.
