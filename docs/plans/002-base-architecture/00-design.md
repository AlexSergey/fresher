# 002 — Base architecture: design

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** core, engine, store, modules, rules,
testing, app

## Intent

The user's task (translated): "Create a draft of plan 002 with the base architecture of the application. Strict
modularity is very important: cleaning Docker is a separate module, `node_modules` a separate one, the duplicate search a
separate one, and so on. Communication through events, and through interfaces where things are related. The core is
shared. Settings must be saved; decide right away whether to take SQLite or something simpler. The main thing is that no
extra dependencies have to be installed: the user starts the app and uses it. Text labels live in a separate dictionary,
in case of localization."

## Goals

1. Every feature is a **module**: its own crate, its own texts, its own settings, its own tests. Adding a module adds a
   crate and one line of registration; no other module changes.
2. Modules never know each other. They talk to the rest of the app through **ports** (traits in the core) and
   **events**.
3. One shared core: domain types, ports, events, text keys.
4. Settings are saved and survive a restart, with nothing for the user to install.
5. No user-facing text in the code: every label comes from a dictionary by key.
6. The whole path works end to end on a made-up module: UI → command → engine → module → events → UI.
7. The contracts later plans build on exist and are tested: the removal description of plan
   [008](../008-safe-removal/readme.md), the error shape of plan [011](../011-diagnostics/readme.md), the walk port of
   plan [004](../004-fast-scan/readme.md), the settings schema of plan [005](../005-settings/readme.md), the descriptor
   the generic module page of plan [003](../003-base-ui/readme.md) renders.

## Non-goals

- Real modules (Docker, `node_modules`, JetBrains, duplicates, …): each gets its own plan and its
  [category](../../categories/README.md) files.
- The fast scan (plan [004](../004-fast-scan/readme.md)): until it exists the app wires a walk that finds nothing.
- `fresher-macos`: reserved in the dependency table here, created by plan 004/02.
- The visual design (plan [003](../003-base-ui/readme.md)): the UI of this plan is a bare list.
- Deletion and the clean session (plan [008](../008-safe-removal/readme.md)): this plan fixes only the types a module
  uses to describe a removal and the `Clean` operation in the state machine.
- The guards of plan [007](../007-safe-development/readme.md) and the full error model of plan
  [011](../011-diagnostics/readme.md): they follow right after this plan.

## Design

### Crates

```
crates/
├── core/       fresher-core      domain types, ports (traits), events, commands, errors, text keys; no IO
├── macos/      fresher-macos     the only crate with `unsafe` (created by plan 004)
├── scanner/    fresher-scanner   the walk and the in-memory tree (plan 004)
├── rules/      fresher-rules     the shared matcher for declarative rule data; holds no rules itself
├── store/      fresher-store     the only crate that touches the app's data folder
├── cleaner/    fresher-cleaner   the only code that deletes (plan 008)
├── engine/     fresher-engine    module registry, event bus, scan session, settings in memory
├── testing/    fresher-testing   fixtures, fakes, the module test harness (dev only)
└── e2e/        fresher-e2e       end-to-end and golden tests (tests only)
modules/
├── demo/       fresher-module-demo   made-up findings, reads nothing from the disk
└── <id>/       fresher-module-<id>   one crate per feature: docker, node-modules, duplicates, jetbrains, …
app/tauri/      fresher-app       the composition root: real adapters, module registration, event forwarding
```

Modules sit in a top-level `modules/` folder, not under `crates/`: Cargo's `crates/*` glob fails on a folder without a
`Cargo.toml`, and excluding `crates/modules` excludes its members too (checked on Cargo 1.99). The workspace members
become `["app/tauri", "crates/*", "modules/*"]`.

Who may depend on whom (normal dependencies; `fresher-testing` is allowed as a dev-dependency anywhere, `fresher-e2e`
may depend on everything):

| Crate       | May depend on                                |
| ----------- | -------------------------------------------- |
| `core`      | —                                            |
| `macos`     | —                                            |
| `rules`     | core                                         |
| `scanner`   | core, macos                                  |
| `store`     | core                                         |
| `cleaner`   | core, macos                                  |
| `engine`    | core                                         |
| `modules/*` | core, rules — never another module           |
| `testing`   | core                                         |
| `app`       | everything (it wires the adapters together)  |

A repository guard reads `cargo metadata` and fails on any edge outside this table (plan 07), so modularity is checked,
not promised. The engine depends on the core alone: everything it needs from outside — the walk, the settings file, the
clock, external commands, the access status — is a port in the core, implemented elsewhere and injected by the app.

### Dependencies

All third-party crates this plan adds, pinned in `[workspace.dependencies]` in step 01/01, so later plans running in
parallel never edit the root `Cargo.toml` for them:

| Crate                          | Version   | Licence          | Used by                     | Why                                                    |
| ------------------------------ | --------- | ---------------- | --------------------------- | ------------------------------------------------------ |
| `serde`, `serde_json`          | (pinned)  | MIT / Apache-2.0 | core, store, rules, testing | already in the workspace                               |
| `thiserror`                    | (pinned)  | MIT / Apache-2.0 | every library crate         | already in the workspace                               |
| `toml`                         | (pinned)  | MIT / Apache-2.0 | rules, testing              | already in the workspace                               |
| `tempfile`                     | (pinned)  | MIT / Apache-2.0 | store (atomic writes)       | already in the workspace (dev); now a normal dep too   |
| `tracing`                      | (pinned)  | MIT              | engine, store               | already in the workspace                               |
| `libc`                         | `=0.2.191` | MIT / Apache-2.0 | rules                       | `O_NOFOLLOW`, `O_NONBLOCK` (added after the review)   |
| `ts-rs`                        | `=12.0.1` | MIT              | core, app (tests)           | TypeScript types generated from the Rust types         |
| `tauri-plugin-single-instance` | `=2.5.5`  | MIT / Apache-2.0 | app only                    | one instance; pinned in `app/tauri/Cargo.toml`         |

No async runtime, no channel crate: the engine uses `std::thread` and `std::sync::mpsc`. Tauri's runtime stays in the
shell.

### Core types

All public, serializable (`serde`) and exported to TypeScript (`ts-rs`) unless marked internal. `u64` values (sizes,
timestamps) are exported as `number` (`#[ts(type = "number")]`): exact below 2^53 bytes, 9 PB.

```rust
// Identity
pub struct ModuleId(&'static str);         // kebab-case: "demo", "node-modules"
pub struct CategoryId(&'static str);       // "<module>.<name>": "demo.caches"
pub struct FindingId(u32);                 // session-scoped, assigned by the engine; the UI's only handle
pub struct FindingKey { module: ModuleId, identity: String }  // stable across scans (ignore list, history)
pub struct ScanId(u32);
pub struct Timestamp(u64);                 // seconds since the Unix epoch, from the `Clock` port

// What was found
pub enum DangerLevel { Safe, Low, Medium, High }               // Ord: Safe < High
pub enum Completeness { Complete, Partial, Unknown }           // "42 GB", "42 GB+", "?"
pub struct Sizes { logical: ByteSize, allocated: ByteSize, completeness: Completeness }
pub enum Subject { Path(PathBuf), External { tool: ToolId, id: String } }   // a folder, or a Docker image
pub enum Removal {                          // described by a module, executed only by the cleaner (plan 008)
    MoveToTrash(PathBuf),
    EmptyFolder(PathBuf),                   // for caches whose app expects the folder to exist
    ToolCommand { tool: ToolId, args: Vec<String> },   // a fixed argument list, never a shell string
}

pub struct Text { key: TextKey, args: BTreeMap<String, TextArg> }
pub enum TextArg { Text(String), Count(u64), Size(ByteSize), Date(Timestamp), Path(PathBuf) }

pub struct CategoryDescriptor { id: CategoryId, danger: DangerLevel }
// Its texts are found by convention: <category>.name, .what, .made-by, .if-removed, .comes-back.

pub struct NewFinding {                     // what a module produces
    key: FindingKey, category: CategoryId, subject: Subject, sizes: Sizes,
    danger: DangerLevel,                    // starts at the category's; `raise_danger` only goes up
    title: Text, reason: Text,              // the name of the item, why it matched
    removal: Removal,
}
pub struct Finding { id: FindingId, module: ModuleId, finding: NewFinding }   // what the engine publishes
```

`NewFinding::new(&CategoryDescriptor, …)` is the only constructor: the danger level starts at the category's and
`raise_danger(level)` keeps the maximum, so no finding is ever less dangerous than its category. "What happens if
removed" and "how it comes back" are the category's texts; a finding states only what is specific to it.

### A module

```rust
pub trait Module: Send + Sync {
    fn descriptor(&self) -> &ModuleDescriptor;
    fn interests(&self) -> &[Interest];
    fn on_match(&self, matched: &Match, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError>;
    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError>;
}

pub struct ModuleDescriptor {
    id: ModuleId,
    categories: Vec<CategoryDescriptor>,
    settings: Vec<SettingDescriptor>,       // keys "<module>.<name>"; "<module>.enabled" is added by the engine
    dictionary: Locales,                    // { en: include_str!("../locales/en.json") }
}

pub struct Interest {                       // what the module wants to see during the one shared walk
    pattern: Pattern,
    claim: Claim,                           // Whole: summed, not indexed inside (plan 004); Item: just this entry
    siblings: Vec<String>,                  // names whose presence next to the item the walk reports ("package.json")
}
pub enum Pattern {
    FolderName(String),                     // any folder named "node_modules"
    UnderHome(String),                      // "Library/Caches/*"
    MarkerFile(String),                     // a folder holding "CACHEDIR.TAG"
    Extension(String),                      // files ending ".dmg"
}

pub struct Match {                          // what the walk delivers, internal (not exported)
    interest: usize, path: PathBuf, kind: EntryKind, sizes: Sizes, modified: Option<Timestamp>,
    siblings: Vec<String>,                  // which of the interest's sibling names exist next to it
}

#[non_exhaustive]
pub struct ModuleContext<'a> {              // the ports a module may use; built by the engine
    clock: &'a dyn Clock, commands: &'a dyn CommandRunner, settings: &'a ModuleSettings, cancel: &'a CancelToken,
}

pub trait FindingSink { fn push(&mut self, finding: NewFinding); }
```

- A module never walks the disk: the walk is a port (`DiskWalk`, below), run once for all modules; the engine sends
  each match to the module whose interest it was. A module with no interests (Docker) does its work in `finish`,
  through `CommandRunner`.
- A module describes a removal as data, never performs it.
- Shared knowledge two modules need (installed apps, detected projects) becomes a port in the core with one
  implementation (plan [009](../009-system-knowledge/readme.md)); `ModuleContext` is `#[non_exhaustive]` so it grows
  without breaking modules.

### Ports

| Port            | In the core since | Implemented by                         | Fake for tests                        |
| --------------- | ----------------- | -------------------------------------- | ------------------------------------- |
| `Clock`         | 001/04            | the app (system time)                  | `FakeClock`                           |
| `Trash`         | 001/04            | plan 008                               | `FakeTrash`                           |
| `CommandRunner` | 001/04            | the app (`std::process`, plan 008)     | `FakeCommandRunner`                   |
| `DiskWalk`      | this plan         | `fresher-scanner` (plan 004)           | `NaiveWalk` on a fixture tree (06/03) |
| `SettingsStore` | this plan         | `fresher-store`                        | in-memory, in the engine's tests      |
| `AccessProbe`   | this plan         | plan 006 (`fresher-macos`)             | a fixed answer                        |

```rust
pub trait DiskWalk: Send + Sync {
    fn walk(&self, roots: &[PathBuf], interests: &InterestSet, cancel: &CancelToken,
            on_match: &mut dyn FnMut(Match), on_progress: &mut dyn FnMut(WalkProgress)) -> Result<WalkSummary, WalkError>;
}
pub trait SettingsStore: Send + Sync {
    fn load(&self) -> Result<StoredSettings, StoreError>;      // raw values, not yet checked against the schema
    fn save(&self, settings: &StoredSettings) -> Result<(), StoreError>;
}
pub trait AccessProbe: Send + Sync { fn status(&self) -> AccessStatus; }
pub struct CancelToken(Arc<AtomicBool>);   // internal; cloned into every thread of an operation
```

### Events and commands

**Commands** go down (UI → shell → engine) and return at once; long work reports through events. The UI names things
by id, never by path.

| Command             | Takes                                        | Returns                                      |
| ------------------- | -------------------------------------------- | -------------------------------------------- |
| `subscribe`         | a Tauri channel for `Event`                  | — (the shell forwards the bus to it)         |
| `start_scan`        | — (the roots come from the settings)         | `ScanId`                                     |
| `cancel_scan`       | —                                            | —                                            |
| `get_overview`      | —                                            | `Overview`: state, last scan, category totals |
| `get_modules`       | —                                            | `Vec<ModuleInfo>`: descriptors for the UI    |
| `get_findings`      | `FindingsQuery { module?, category?, offset, limit }` | `FindingsPage`                      |
| `get_settings`      | —                                            | `SettingsView`: schema, values, problems     |
| `update_setting`    | `SettingUpdate { key, value }`               | —                                            |
| `get_dictionary`    | `Locale`                                     | `Dictionary` (merged: core + every module)   |
| `get_access_status` | —                                            | `AccessStatus`                               |

Every command returns `Result<T, CommandError>`; the shell passes input through and decides nothing.

**Events** go up: one enum, `#[serde(tag = "type")]`, a discriminated union in TypeScript.

```rust
pub enum Event {
    ScanStarted { scan: ScanId, modules: Vec<ModuleId> },
    ScanProgress { scan: ScanId, entries: u64, bytes: ByteSize },
    FindingsAdded { scan: ScanId, findings: Vec<Finding> },
    ModuleFinished { scan: ScanId, module: ModuleId, outcome: ModuleOutcome },   // Completed | Failed(Text) | Cancelled
    ScanFinished { scan: ScanId, outcome: ScanOutcome },                          // Completed | Partial | Failed(Text)
    SettingChanged { key: SettingKey, value: SettingValue },
    AccessChanged { status: AccessStatus },
}
```

- The engine's bus fans events out to subscribers (`EventBus::subscribe() -> Receiver<Event>`): the shell, later the
  store's history and other listeners. A dropped receiver is removed; it never blocks the others.
- Findings are sent in batches: a batch is flushed at 256 findings, when 50 ms have passed on the `Clock` port, or when
  a module finishes — so a scan never floods the UI and tests stay deterministic.
- The TypeScript types for commands and events are **generated from the Rust types** into
  `app/ui/src/ipc/bindings.generated.ts`; a test fails when the file is stale (`UPDATE_BINDINGS=1` rewrites it).

### Errors

Typed errors per crate with `thiserror`: `ModuleError` and `WalkError` (core), `StoreError` (store, also used by the
`SettingsStore` port, so it lives in the core), `RuleError` (rules), `EngineError` (engine, internal). At the engine's
edge every error becomes the one shape the UI sees:

```rust
#[derive(thiserror::Error, Serialize, TS)]
#[serde(tag = "code")]
pub enum CommandError {
    Busy { running: Operation },               // one operation at a time
    NotRunning,                                // cancel with nothing to cancel
    UnknownFinding { id: FindingId },
    UnknownModule { id: String },
    UnknownSetting { key: String },
    InvalidSettingValue { key: SettingKey },
    UnknownLocale { locale: String },
    SettingsReadOnly,                          // the file is from a newer version of the app
    Storage,                                   // the settings could not be written
    Internal,                                  // a bug: the operation stopped safely
}
impl CommandError { pub fn text(&self) -> Text; }   // key "core.error.<code>", with the variant's fields as args
```

Plan 011 adds the severity and the suggested action to this shape; it does not replace it. A module's failure is not a
command error: it is a `ModuleFinished { outcome: Failed(..) }` and the other modules finish.

### Sessions

- The engine is an explicit state machine: `Idle → Scanning → Idle`; `Operation { Scan, Clean }` and the `Cleaning`
  state are reserved for plan 008. **One operation at a time**: a command that does not fit the state is refused with
  `Busy`.
- A scan runs on its own thread; every operation can be cancelled (`CancelToken`) and ends in a defined state; a
  cancelled scan keeps its findings, marked `Partial`.
- The last scan's findings stay in the engine by `FindingId`; an id from another session is unknown. `Engine::finding`
  is the lookup plan 008 builds its plan from.
- **Closing the window quits the app**: the shell calls `Engine::shutdown` (cancel, then join within 5 s;
  a thread still running past that is detached and ends with the app). During a cleanup (plan
  008) the app asks first.
- One instance: starting the app again brings the open window forward (`tauri-plugin-single-instance`).

### IPC safety

- Commands take ids, never paths; a command that leads to a removal (plan 008) takes `FindingId`s the core itself
  produced in this session. Even a compromised webview cannot name an arbitrary path for removal. A folder in a setting
  (the scan roots) only changes what is scanned, never what is removed; the core accepts only `~`, `~/…` or a plain
  absolute path outside the protected system folders.
- Every command validates its input in the core; the shell passes it through.
- The app's commands are allowlisted through `tauri-build`'s app manifest, and the capability file names exactly those;
  no file system, shell, network or dialog plugin is exposed. The content security policy keeps no remote origin.
- The webview runs incognito (a non-persistent data store), so it leaves no cache of its own.

### Settings and the app's own data

**Plain files, no SQLite** (the user's decision, below).

- `settings.json` in the data folder `~/Library/Application Support/dev.macfresher.app/` (the app passes Tauri's
  `app_data_dir` to the store; the store never looks the folder up itself):

  ```json
  { "format": "settings", "version": 1, "values": { "core.scan.roots": ["~"], "demo.enabled": true } }
  ```

- Written atomically: a temporary file in the same folder, `fsync`, then a rename.
- **Typed schema in the core**:

  ```rust
  pub struct SettingKey(String);                         // "core.scan.roots", "<module>.<name>"
  pub enum SettingKind { Switch, Number { min: i64, max: i64, unit: Option<Unit> }, Choice { options: Vec<&'static str> }, Folders }
  pub enum SettingValue { Switch(bool), Number(i64), Choice(String), Folders(Vec<PathBuf>) }
  pub struct SettingDescriptor { key: SettingKey, kind: SettingKind, default: SettingValue }
  // label and description by convention: <key>.label, <key>.description
  pub struct SettingsSchema { .. }                       // core + every module; built by the engine
  impl SettingsSchema { pub fn resolve(&self, stored: &StoredSettings) -> ResolvedSettings; }  // values + problems
  ```

  An unknown key or a broken value falls back to its default and is reported as a `SettingProblem`, never a crash. The
  check is pure and lives in the core; the store only reads and writes the file.
- No setting can switch off the confirmation before deletion or lower a danger level: such keys do not exist, and a
  test of the core schema says so.
- **File versions.** Every file the app writes carries `format` and `version`. What holds the user's decisions or
  history (settings, the cleanup journal, history points) is **migrated** forward, one version at a time; what can be
  rebuilt (the scan cache) is **discarded** on a version change. A file from a newer version of the app is never
  overwritten: the app says so (`SettingsReadOnly`) and works on defaults in memory. A broken settings file is left as
  it is and moved aside to `settings.broken.json` on the first save.
- The app's own footprint: one data folder, a 50 MB budget, `DataFolder::footprint()` measured without following links.
  No log files by default (plan 011). The store deletes nothing in this plan; clearing the app's own data is plan 005's,
  with its own decision on the deletion lint.
- History and the cleanup journal choose their files in their own plans, under the same version policy.
- **Nothing to install**: the app is one self-contained bundle.

### The dictionary

- Every crate with user-facing text owns `locales/en.json`, embedded with `include_str!`; keys are namespaced
  `<module>.<group>.<name>` (`core.` for the core). A module's texts ship with it; the engine refuses a module whose
  keys leave its namespace.
- The core never produces English prose: it emits `Text { key, args }`. The UI asks for the merged dictionary once
  (`get_dictionary("en")`) and resolves keys with a small `t()` of our own: `{name}` placeholders, plural forms as
  `.one` / `.other` key suffixes chosen by `Intl.PluralRules`; sizes, counts and dates in `args` are formatted with
  `Intl`. No i18n library.
- The UI's own chrome lives in `app/ui/src/locales/en.json`; its keys are a TypeScript type, so a missing chrome key is
  a compile error.
- Checks: every key the code can emit exists (the core's keys by an exhaustive list, a module's by its descriptor and
  its golden findings); every category has all five texts; no key is unused.
- Only `en` ships now; a second language is one more file per crate.

### The module kit

- **Rules as data.** A module keeps its rules in `rules.toml` next to its code; `fresher-rules` parses, validates and
  matches them. A rule states the category it produces, where to look (it compiles to `Interest`s), how the item is
  recognized (a marker file, a sibling, a name or version pattern, a minimum age), **what must not match** (required),
  a danger level no lower than the category's, and the removal. `CACHEDIR.TAG` is honoured. Matching is pure: it
  works on the `Match`es the walk delivered and the `Clock`, with no IO.
- **The category contract.** A category is complete only with its five texts and its danger level (rule 3 of
  [`rules.md`](../../rules.md)); category `<module>.<name>` is documented in `docs/categories/<module>-<name>.md`. A
  test fails for a missing text or a missing file. The `demo` module is dev-only and checked without the catalogue.
- **The test harness** in `fresher-testing`: `NaiveWalk` (the reference `DiskWalk` over a fixture tree, links listed,
  never followed), `run_module` (one module against a fixture home → its findings) and `assert_module_contract`. A
  module's tests are fixture → expected findings, negative cases first, one golden report per module.
- **A template and a checklist** in [`docs/architecture/`](../../architecture/README.md).

### Parallel work

After plan 01, plans 02, 03, 04, 06 (steps 01–03) and step 07/01 touch separate crates and folders and can run at the
same time, each in its own worktree branched from `feat/002-base-architecture` and merged back. The core is frozen
for them: a missing contract stops the plan and goes into `crates/core` in the main session, never edited in parallel.
`Cargo.lock` is never merged by hand: on a conflict take either side and regenerate it (`cargo metadata
--format-version 1 > /dev/null`), then run the checks. The shared files of each step are listed in its file.

## Open questions

None: the user approved the draft with its recommended decisions and asked the agent to work autonomously.

## Decisions

- 2026-11-15 — The plan is approved: "accept the recommended decisions, work autonomously". All 14 agent's calls of the
  draft are accepted as decisions:
  1. **Files instead of SQLite** for settings and the scan cache; `goal.md` §5 and
     [`architecture/README.md`](../../architecture/README.md) are updated by this plan (07/02).
  2. **One crate per module**, with a guard on the dependency table.
  3. **`fresher-rules` is the shared matcher**; each module owns its rule data.
  4. **A new `fresher-engine`** holds orchestration, so the shell stays thin; **`fresher-macos`** is the only crate
     allowed `unsafe`.
  5. **A `demo` module** proves the contract and feeds plan 003 before any real scan exists.
  6. **Module-owned dictionaries**, merged at start; our own `t()` instead of an i18n library.
  7. **TypeScript types generated from Rust.**
  8. **No async runtime in the core.**
  9. **The webview keeps no persistent data.**
  10. **Rules are TOML data owned by each module**, with a required "must not match" part; the catalogue in
      `docs/categories/` is checked against the code.
  11. **One operation at a time**, and closing the window quits the app.
  12. **Commands take ids, never paths.**
  13. **Migrate what holds decisions, discard what can be rebuilt**; never overwrite a file from a newer version.
  14. **Target version 0.1.0.**
- 2026-11-15 — The main session, after the code review of the plan: **modules are isolated from panics**. The release
  profile unwinds (`panic = "unwind"`, no longer `"abort"`); the engine wraps every call into a module in
  `catch_unwind`, so a panicking module ends `ModuleFinished { Failed }` with the core's text `core.module.panicked`,
  the other modules finish and the scan ends `Partial`; any other panic on the scan thread still ends the scan
  `Partial` and the engine goes `Idle`. This overrules the step note of 02/03 ("a panic is not caught, the release
  profile aborts").

## Agent's calls (to check)

Made while turning the draft into steps:

1. **Modules in a top-level `modules/` folder**, not `crates/modules/` (Cargo's glob limit, checked); crates named
   `fresher-module-<id>`; members `["app/tauri", "crates/*", "modules/*"]`.
2. **Every port is in the core**, `SettingsStore` included (the draft put it in `fresher-store`), so the engine depends
   on the core alone; `fresher-store` implements it.
3. **`ts-rs` 12.0.1 over specta** (specta 2 is still a release candidate); the derives sit directly on the core types,
   no feature flag; `u64` exported as `number`.
4. **`std::thread` and `std::sync::mpsc`**, no channel or async crate in the core and the engine.
5. **A session-scoped `FindingId` (u32)** for the UI, separate from the stable `FindingKey` used by the ignore list and
   history: an id from an old scan is refused by construction.
6. **Two commands added to the draft's set**: `get_modules` (the generic module page of plan 003) and `subscribe` (the
   Tauri channel for events).
7. **Settings values are checked in the core** (pure); the store only does file IO, versions and migration.
8. **The store deletes nothing in this plan**; clearing the app's own data is decided in plan 005.
9. **The `demo` module is registered only in debug builds**, until plan 007's dev-tools feature takes it over.
10. **No disk walk until plan 004**: the app wires a walk that finds nothing, recorded in
    [`docs/hacks/`](../../hacks/README.md).
11. **Category ids `<module>.<name>`**, catalogue files `docs/categories/<module>-<name>.md`; text keys of categories
    and settings derived by convention, not listed.
12. **Plurals as `.one` / `.other` key suffixes** with `Intl.PluralRules`; placeholders `{name}`.
13. **The commitlint scopes `engine`, `macos`, `modules`, `testing`** are added in step 01/01, because the first commits
    of plan 02 need them.
14. **Every new workspace dependency is pinned in step 01/01**, so parallel plans do not edit the root `Cargo.toml`
    (except 02/05, which adds `modules/*` to the members).
15. **The app's commands are allowlisted** through `tauri-build`'s app manifest; `tauri-plugin-single-instance` =2.5.5;
    the window runs incognito.
16. **Findings batched by count (256), time (50 ms on the `Clock` port) and module end.**
17. **The version moves to 0.1.0** in `Cargo.toml`, `tauri.conf.json` and both `package.json` files in the last step.

Made while implementing plan 01 (details in each step file, "Notes from the implementation"):

18. **The port's `CommandError` is renamed `CommandRunError`**, so `CommandError` is the error the UI sees; the
    fakes of `fresher-testing` keep their state in a `Mutex`, because the ports are `Send + Sync`.
19. **Enum tags are kebab-case** (`scan-started`, `not-running`, `medium`), so an error code is a text-key segment;
    enums with newtype variants (`Subject`, `Removal`, `TextArg`, `SettingValue`) are adjacently tagged
    (`{ "kind", "value" }`); `Finding` serializes flat.
20. **Ids held as `&'static str` serialize only**: the commands' inputs name modules, categories and settings by
    string, and the engine looks them up (`UnknownModule`, `UnknownSetting`).
21. **`core` is a reserved module id**; a module may not declare `<module>.enabled` (the schema adds it).
22. **Folder settings keep `~` as written**: the app fills in the home folder, the core does no IO.
23. **The settings file's format, version and policy are constants on `StoredSettings`**
    (`"settings"`, 1, `Migrate`), with no separate type.
24. **A dictionary refuses a key written twice in one file** and a key that is not a text key (`InvalidKey`).

Made while implementing plan 05 (the main session's decisions; details in the step files):

25. **The settings file's state is in the IPC contract**: `SettingsView.file: SettingsFile` (`loaded`, `newer-version`,
    `broken`, `unreadable`), moved from the engine to the core; `Engine::settings_file()` is gone.
26. **A settings file that could not be read is read-only** (`SettingsFile::Unreadable`), like one from a newer
    version: a passing read error followed by a save must not replace the user's values with the defaults.
27. **The core dictionary is merged in the registry**: `Registry::new()` starts with `core_dictionary()` and returns a
    `Result`; `get_dictionary("en")` answers the core's and every module's texts.

Made while finishing the plan (the two calls the conclusions of plan 05 left to check):

28. **A refused command carries its text**: the commands return the core's `Refusal { error, text }`, `text` built by
    `CommandError::text()`; the UI shows it as it is and no longer rebuilds it. A failure that is not a refusal (the
    IPC itself) shows the UI's own `ui.core.unreachable`.
29. **The IPC types serialize multi-word fields in camelCase** (`Overview.lastScan`, `CategoryTotal.highestDanger`,
    `AccessStatus.fullDiskAccess`); the bindings test refuses a snake_case field, so ESLint's `camelcase` has no
    exceptions. No file on disk changed: `settings.json` keys are setting keys, not field names.
