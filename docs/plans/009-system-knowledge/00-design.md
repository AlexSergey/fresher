# 009 — System knowledge: design

**Status:** ✅ approved · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, macos, system, scanner, rules,
engine, cleaner, testing, app

Expanded from the one-file draft ([`readme.md`](./readme.md)) on 2026-11-16. The user delegated every open question to
the agent with "take the recommended option and record it" (section "Decisions").

## Intent

Several modules need the same facts about the Mac: which apps are installed and in which version, where the projects
are, what is running, where the system keeps app caches and settings. Modules may not depend on each other, so these
facts are shared services behind ports of the core — gathered once, used by all (the user, 2026-11-15).

## Goals

1. One source of truth for each fact; no module gathers it on its own.
2. Gathered at most once per scan, in memory, and only when something asks.
3. Every service has a fake in `fresher-testing`, so a module is tested with made-up apps, processes and folders.
4. A module names a place by a known folder, never by a macOS path (the user's decision of 2026-11-15).
5. The cleaner skips an item that belongs to a running app, whether or not it can see the app's open files
   ([known issue](../../known-issues/cleaner-other-users-processes.md), first option).
6. An external tool can be stopped: a cancelled scan or cleanup kills the program it runs.

## Non-goals

- Per-process disk activity ([live storage attribution](../../ideas/live-storage-attribution.md)).
- Storing any of it: the facts are cheap to gather again and are never written to disk.
- Any module that uses the facts (leftovers, uninstaller, stale projects): each comes with its own plan.
- Volume kinds, free and purgeable space: they move to plan [012](../012-storage-map/readme.md), their first consumer
  (decision 3).
- Verifying code signatures: the team id is read, not validated (call 5).

## What already exists (plans 002, 004, 006, 008)

The draft predates the code; this plan extends and reuses it rather than duplicating it.

| Draft's service    | What exists today                                                                                                    | This plan                                                                                     |
| ------------------ | -------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `Clock`            | the core's port; `SystemClock` in the app (and a private one in the scanner); `FakeClock`                            | unchanged; the app's `SystemClock` moves to `fresher-system`                                  |
| `CommandRunner`    | the core's port without cancel; the app's `ProcessCommandRunner` (fixed arguments, a 60 s timeout); `FakeCommandRunner` | gains a cancel token (the gap plan 004 left, its call 33); the runner moves to `fresher-system` |
| `Volumes`          | the core's port (`identify`, `available`), `MacVolumes`, `FakeVolumes`                                                | unchanged (decision 3)                                                                        |
| `RunningProcesses` | `MacOpenFiles` walks the user's processes (`libproc`) and names each by its outermost app bundle; `OpenSet`           | new port `RunningApps`, built on the same process walk in `fresher-macos`                      |
| `KnownFolders`     | `Pattern::UnderHome(String)` and the scanner's `AnchorResolver` (the home is the only anchor)                          | new port; `Pattern::Under(KnownFolder, String)` replaces `UnderHome`                          |
| `InstalledApps`    | nothing                                                                                                              | new port and adapter                                                                          |
| `Projects`         | nothing; the walk already reports marker files (`Pattern::MarkerFile`) with sibling names                           | new, collected by the engine from the one shared walk                                         |

## Design

### 1. Where the code lives

- **`fresher-core`**: the types and ports — `KnownFolder` and `KnownFolders`; `BundleId`, `TeamId`, `InstalledApp`
  and `InstalledApps`; `RunningApp`, `RunningApps` and `RunningSet`; `Project`, `ProjectKind`; `SystemPorts` (the
  three ports together) and `SystemFacts` (one scan's lazy, cached view of them). No IO, as before.
- **`fresher-system`** (new, depends on core and macos): the safe adapters of system knowledge — `MacKnownFolders`,
  `MacInstalledApps` with its own `Info.plist` reader and code-signature reader, `MacRunningApps`, and the two app
  adapters that were waiting for this crate (plan 006's call 2): `SystemClock` and `ProcessCommandRunner`.
  Justification after the inspection: the readers are a few hundred lines of parsing in safe Rust that need no FFI;
  keeping them out of `fresher-macos` keeps the only unsafe crate small, and out of the app keeps them testable on
  fixture trees.
- **`fresher-macos`**: one new safe function over its existing `libproc` calls, `running_processes()` (every
  process whose executable the system gives, with its short name). `MacOpenFiles` keeps its own walk of descriptors;
  both share the listing of process ids. No new `unsafe`.
- **`fresher-engine`**: builds one `SystemFacts` per scan, hands it to every module's context, collects projects
  from the walk.
- **`fresher-cleaner`**: check 10 reads a `RunningSet` beside the `OpenSet`.

### 2. Known folders

```rust
pub enum KnownFolder {                 // serde: kebab-case, so rules can name it
    Home, Library, AppCaches, AppSupport, Preferences, Logs, Containers, GroupContainers,
    SavedState, Trash, Downloads, Applications, UserApplications, SystemApplications,
}
pub trait KnownFolders: Send + Sync {
    /// Where this system keeps `folder`; `None` when it has no such place.
    fn locate(&self, folder: KnownFolder) -> Option<PathBuf>;
}
pub enum Pattern { Under(KnownFolder, String), FolderName(String), MarkerFile(String), Extension(String) }
```

- `Pattern::UnderHome(path)` becomes `Pattern::Under(KnownFolder::Home, path)`; a module writes
  `Under(AppCaches, "JetBrains/*")`. The path rules stay: relative, plain names, `*` matches one name.
- The scanner's `AnchorResolver` becomes `Arc<dyn Fn(KnownFolder) -> Option<PathBuf>>`, built by
  `known_folders_anchor(&dyn KnownFolders)`, which canonicalizes every folder once. The home folder (for `~` in the
  roots and the scan's user) is `KnownFolder::Home`. The interest fingerprint includes the folder's name.
- `fresher-rules`: `[rule.look] under-home = "…"` becomes `under = { folder = "app-caches", path = "…" }`.
- `MacKnownFolders::new(home)` answers the macOS layout from the home folder it is given; pure, opens nothing.
- `FakeKnownFolders::home_layout(root)` answers the same layout under a fixture root (the table is repeated in the
  fake, so the testing crate stays on the core alone); `.with(folder, path)` overrides one.

### 3. Installed apps

```rust
pub struct BundleId(String);           // reverse-DNS: ASCII letters, digits, '.', '-', '_'; compared ignoring case
pub struct TeamId(String);             // ten ASCII capitals and digits
pub struct InstalledApp {
    pub bundle_id: BundleId, pub team_id: Option<TeamId>, pub name: String,
    pub version: Option<String>,       // CFBundleShortVersionString
    pub build: Option<String>,         // CFBundleVersion
    pub path: PathBuf, pub helpers: Vec<BundleId>,
}
pub trait InstalledApps: Send + Sync { fn list(&self) -> Result<Vec<InstalledApp>, SystemError>; }
```

- **Where apps are looked for**: the known folders `Applications`, `UserApplications` and `SystemApplications`; a
  bundle directly in one, or one folder below (`Utilities`, a vendor's folder). Never inside a bundle, never through a
  link, never on another volume than its folder.
- **Identity by bundle id and team id only** (the draft's rule): the bundle id from the bundle's own
  `Contents/Info.plist` (`CFBundleIdentifier`); a bundle without one is skipped. Never a loose name.
- **The `Info.plist` reader** (`fresher-system`, our own): reads the top-level dictionary's string values of an XML or
  a binary property list, at most 1 MiB; anything else (nested values, numbers, dates) is skipped, never a failure of
  the whole file. Our own because the app needs six string keys; a general library would bring five crates.
- **The team id** from the main executable's code signature: the Mach-O header (thin or universal, the first slice),
  its `LC_CODE_SIGNATURE` load command, the signature's super blob, its code directory, and the team identifier that
  directory names (version 0x20200 and later). Read with bounded reads (64 KiB of load commands, 1 MiB of signature);
  an unsigned, ad-hoc signed or unreadable executable has no team id. Read, not validated (call 5).
- **Helpers**: the bundle ids of the bundles in `Contents/Library/LoginItems`, `Contents/Helpers`,
  `Contents/XPCServices`, `Contents/PlugIns` and `Contents/Frameworks` (one level), so data named after a helper is
  known as the app's.
- A bundle that cannot be read is skipped and counted in the log; only a failure to read every apps folder is an
  error. The order is by path, so the answer is deterministic.

### 4. Running apps and who owns a folder

```rust
pub struct RunningApp { pub name: String, pub bundle_ids: Vec<BundleId> }
pub trait RunningApps: Send + Sync { fn list(&self) -> Result<Vec<RunningApp>, SystemError>; }
pub struct RunningSet { … }            // built from a list
impl RunningSet { pub fn owner_of(&self, path: &Path) -> Option<&str>; }
```

- `MacRunningApps` lists every process whose executable path the system gives — the user's own and, where the system
  allows it, other users' services — and maps each executable to the bundles around it (`.app`, `.appex`, `.xpc`):
  the name is the outermost app's (as `MacOpenFiles` names a holder today), the ids are every enclosing bundle's
  `CFBundleIdentifier`. An `Info.plist` is read once per bundle per call. A process outside any bundle is left out:
  it has no bundle id to own a folder by.
- **Ownership**, pure, in the core: a path belongs to a running app when one of its names equals one of the app's
  bundle ids, or starts with the id and a dot (`com.example.App.savedState`, `com.example.App.plist`), ignoring case.
  Too wide errs on the side of skipping, which is the safe side.
- **The cleaner's check 10** reads the `RunningSet` beside the `OpenSet`, both taken at the same moments (before the
  first item and every two seconds). A hit skips the item as `InUse { app }` — the same reason and text as an open
  file: "the app is using it". A failure to list the running apps refuses the cleanup, as a failure to list the open
  files does (fail closed).
- Modules read the same facts through `SystemFacts` to raise a finding's caution ("quit the app first"); no module
  does yet.

### 5. Projects from the shared walk

```rust
pub enum ProjectKind { Node, Rust, Gradle, Maven, Python, SwiftPackage, Go, Dart }
pub struct Project {
    pub root: PathBuf, pub kinds: Vec<ProjectKind>,
    pub artifacts: Vec<PathBuf>,       // the kinds' artifact folders that exist next to the markers
    pub last_activity: Option<Timestamp>,
}
```

- Each kind lists its marker files and its artifact folders as data in the core (`package.json` → `node_modules`;
  `Cargo.toml` → `target`; `build.gradle`, `build.gradle.kts`, `settings.gradle`, `settings.gradle.kts` → `build`,
  `.gradle`; `pom.xml` → `target`; `pyproject.toml`, `setup.py` → `.venv`, `venv`; `Package.swift` → `.build`;
  `go.mod` → none; `pubspec.yaml` → `.dart_tool`, `build`).
- **No second pass**: the engine adds one `MarkerFile` interest per marker and one `FolderName` interest per artifact
  name (all `Claim::Item`), owned by the reserved id `ModuleId::system()` (the core's namespace, which no module may
  take). Their matches go to the project collector, never to a module; an artifact folder counts when its parent is a
  project of a kind that makes it. (Built so in 05/01: a marker's sibling names are read beside the matched folder,
  not inside it, so they cannot carry the artifacts as first designed.)
- After the walk the collector merges the markers of one folder into one project, and drops a project inside another
  project's artifact folder (every package inside `node_modules`). The projects are set on the scan's `SystemFacts`
  before any module's `finish`; during the walk `projects()` answers `None`.
- **Last activity**, first version: the project folder's own modification time as the walk reports it (it changes
  when a file is added, removed or renamed at the top level, which atomic saves do). The newest change of the whole
  project outside its artifact folders needs a query of the walked tree before the commit; it is
  [deferred](../../deferred/project-last-activity.md) to the stale-project module's plan, its first consumer.

### 6. One scan's facts: `SystemFacts`

```rust
pub struct SystemPorts { pub known_folders: Arc<dyn KnownFolders>, pub apps: Arc<dyn InstalledApps>,
                         pub running: Arc<dyn RunningApps> }
pub struct SystemFacts { … }           // one per scan
impl SystemFacts {
    pub fn new(ports: &SystemPorts) -> Self;
    pub fn locate(&self, folder: KnownFolder) -> Option<PathBuf>;
    pub fn installed_apps(&self) -> Result<&[InstalledApp], SystemError>;   // gathered on first ask
    pub fn running_apps(&self) -> Result<&RunningSet, SystemError>;        // gathered on first ask
    pub fn projects(&self) -> Option<&[Project]>;                          // after the walk
}
```

- The engine makes a new `SystemFacts` for every scan, so a rescan refreshes everything; nothing is kept between
  scans and nothing is written.
- `ModuleContext::new` takes `&SystemFacts`; `cx.system()` gives it to the module. Lazy: a scan whose modules never
  ask reads no `Info.plist` and lists no process.
- The engine's `Ports` gain `system: SystemPorts`; the app composes the macOS adapters, the tests the fakes.

### 7. Commands that can be stopped

```rust
pub trait CommandRunner: Send + Sync {
    fn run(&self, program: &str, args: &[&str], cancel: &CancelToken) -> Result<CommandOutput, CommandRunError>;
}
pub enum CommandRunError { NotInstalled { .. }, Failed { .. }, Cancelled }
```

- `ProcessCommandRunner` checks the token every 10 ms beside its timeout and kills the program when it is set.
- Callers pass the token they already hold: a module `cx.cancel()`, the engine's tool checks the scan's token (a
  cancelled check changes no tool's state). The cleaner's tool removal runs to its end or its timeout: Stop ends a
  cleanup after the item in flight, and a removal killed halfway would leave its state unknown (found while
  building 02/03). `FakeCommandRunner` answers `Cancelled` when the token is set.

### 8. Testing

- Every port has a fake in `fresher-testing`: `FakeKnownFolders`, `FakeInstalledApps` (a list, or an error),
  `FakeRunningApps` (a script of lists, so a test can start an app between two snapshots); `run_module`'s `Ports` carry
  them, so a module test says "WebStorm 2026.3 is installed" in one line. The scenario files of plan
  [007](../007-safe-development/readme.md) do not exist yet; when they come, they fill these fakes.
- The readers are tested on bytes the tests build (XML and binary property lists, Mach-O headers with and without a
  signature, universal files, truncated and hostile inputs, with property tests that no input panics) and on fixture
  bundles written to a temporary folder. No test reads a real app or the real home folder.
- A read-only test of the real adapters runs on this Mac's own `/System/Applications` (it exists on every supported
  macOS) and on this test process itself for the running apps; each skips quietly where the place is missing.
- The engine's test: a scan over a fixture tree with projects reports them to `finish`, drops the nested ones, and a
  module that never asks makes the fakes see no call.
- The cleaner's tests: an item named after a running app's bundle id is skipped as in use; a failed listing refuses
  the cleanup.

### 9. Interfaces with other plans

- **005 (settings, built in parallel)**: this plan does not touch `crates/core/src/settings/` or the settings UI; it
  touches `ModuleContext::new` and the engine's `Ports`, which 005 may also touch — the later merge resolves that.
- **007**: its scenario files fill the fakes of this plan.
- **010**: nothing.
- **012**: the `Volumes` extension (decision 3).
- Plans of modules (leftovers, uninstaller, stale projects): read `cx.system()`.

## Open questions

None left: every question below was delegated and decided.

## Decisions

Delegated to the agent by the user on 2026-11-16, recommended option taken (the user was away and asked for every open
question to be answered with the recommended option and recorded):

1. **The plan is approved** as designed here, with the agent's calls below still to check.
2. **Target version 0.5.5** — the user's decision of 2026-11-16, replacing the 0.7.0 first taken here: many
   features are still to come. Plan 005, built in parallel, targets 0.5.5 too; whichever branch is merged into `main`
   first bumps the workspace to 0.5.5, the second keeps it.
3. **The `Volumes` extension (kinds, free and purgeable space) moves to plan 012**, its first consumer: nothing reads
   it before the storage map, and the purgeable space needs a new Foundation call whose answer cannot be tested
   deterministically ([deferred](../../deferred/volume-facts.md)).
4. **`KnownFolders` replaces `Pattern::UnderHome`** before the first real module (the draft's call 6, recommended):
   no module is ever written with a macOS path.
5. **A new crate `fresher-system`** (the draft's call 1), justified in section 1.
6. **Project last activity, first version**: the project folder's own modification time; the precise one is deferred
   to its first consumer (section 5).
7. **Running apps own folders by bundle id**, and the cleaner skips such an item as in use (section 4); a failed
   listing refuses the cleanup.

The user's decision of 2026-11-15 stays: macOS first, Windows not ruled out — modules name known folders through
`KnownFolders`, never macOS paths.

## Agent's calls (to check)

1. **Our own `Info.plist` reader** (XML and binary, string values of the top level only) instead of a property-list
   library, which would add five crates for six keys.
2. **Our own code-signature reader for the team id** instead of the Security framework: safe Rust over bounded reads
   of the executable, no new FFI.
3. **`SystemFacts` is made per scan** and handed to modules through `ModuleContext`; the facts are lazy, so a scan
   whose modules never ask costs nothing.
4. **Projects ride on the shared walk** under the reserved owner `ModuleId::system()`, and a project inside another
   project's artifact folder is dropped.
5. **The team id is read, not validated**: it identifies an app for matching, it is not a trust decision; a forged
   team id can only make the app keep more (a leftover looks owned), never remove more.
6. **Ownership by name**: a path's name equal to a running app's bundle id, or starting with it and a dot, ignoring
   case. It may skip too much (`com.example.App.Extra` of another app), never too little.
7. **The skip reuses `InUse { app }`** and its text, so no IPC type, text or screen changes.
8. **Processes of other users are included** in the running apps where the system names their executable: they are
   the services whose open files the cleaner cannot see (the known issue).
9. **`SystemClock` and `ProcessCommandRunner` move from the app to `fresher-system`** (plan 006's call 2); the
   scanner keeps its private clock.
10. **The fake known folders repeat the macOS layout** (fourteen lines) so `fresher-testing` keeps depending on the
    core alone.
11. **No module uses the facts in this plan**: the demo and trial modules are left as they are; the first real module
    is the first consumer.
