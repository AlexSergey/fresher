# Architecture

The crates, the layers and the data, as plan [002](../plans/002-base-architecture/readme.md) built them, plan
[004](../plans/004-fast-scan/readme.md) added the disk walk to them and plan
[008](../plans/008-safe-removal/readme.md) the removal, and plan [009](../plans/009-system-knowledge/readme.md) the
system's facts; their designs argue every choice
([002](../plans/002-base-architecture/00-design.md), [004](../plans/004-fast-scan/00-design.md),
[008](../plans/008-safe-removal/00-design.md)). A crate a later plan creates keeps its row here and is described in
detail by that plan. How to write a module: [`modules.md`](./modules.md); how anything is removed, for a developer
about to touch it: [`removal.md`](./removal.md); what the app may see and do, and how to test it:
[`permissions.md`](./permissions.md).

## Layers

```
app/ui (React)  ──typed IPC (generated types)──▶  app/tauri: commands, the composition root
      ▲                                                  │
      └───────────── events (one channel) ───────────────┤
                                                         ▼
                                           fresher-engine: registry, event bus,
                                           one operation at a time, settings in memory,
                                           the plan builder, the cleanup and its history
                                                         │
          ┌──────────────────────────────┬───────────────┴─────────────┬───────────────────────────┐
          ▼                              ▼                             ▼                           ▼
 ports of the core: DiskWalk     modules/* (one crate per        Cleaner, from a confirmed   SettingsStore, ScanCache,
 (fresher-scanner) → DiskState   feature) over ONE shared        plan only: fresher-cleaner, CleanupJournal
          │                      walk → findings, each           the only code that removes  (fresher-store: files)
          │                      removal described as data                     │
          ▼                                                                    ▼
 OS ports: FolderReader, ChangeJournal, Volumes                  Trash, OpenFiles, RunningApps, CommandRunner, and
 (fresher-macos: the only unsafe code); KnownFolders,            the calls beneath held folders (fresher-macos,
 InstalledApps, RunningApps (fresher-system)                      fresher-system)
```

- **Strict modularity**: every feature is a module in its own crate under `modules/`; a module never knows another,
  never walks the disk and never deletes. It declares what it wants to see, the engine runs one walk for all modules
  and hands each module its matches; the module turns them into findings.
- **A finding's removal is data, and may be several paths**: one path moved or emptied, a tool's command, or two to 64
  paths that go together as one item the user confirms (`Removal::MoveTogether`, the first the finding's subject).
  The cleaner checks every path before the first moves, journals every path that went, and puts them back together
  ([`removal.md`](./removal.md)).
- **The engine depends on the core alone**: everything it needs from outside is a port (a trait) in `fresher-core`,
  implemented in another crate and injected by the app.
- **Events, not polling**: the engine streams typed events (`scan-started`, `findings-added`, …) to the UI through one
  channel; the commands take ids, never paths.
- **No async runtime in the core**: `std::thread` and `std::sync::mpsc`; Tauri's runtime stays in the shell.

## Crates

| Crate                 | Folder           | Responsibility                                                                                                                                                                          | May depend on |
| --------------------- | ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| `fresher-core`        | `crates/core`    | domain types, the module contract, ports (traits), settings schema, events, commands, errors, texts, the apps view's types (`apps/`, plan 036); no IO                                      | —             |
| `fresher-macos`       | `crates/macos`   | the only crate with `unsafe` (in `mod ffi`): the bulk read, volumes, the change journal, the IO policy, the Trash call, open files, the protection list, the calls beneath held folders | core          |
| `fresher-scanner`     | `crates/scanner` | the walk, the tree, the matcher, the cache codec, the rescan and the queries behind `DiskWalk`/`DiskState`                                                                              | core, macos   |
| `fresher-rules`       | `crates/rules`   | the shared matcher for declarative rule data (TOML); holds no rules, each module owns its own                                                                                           | core          |
| `fresher-system`      | `crates/system`  | the safe adapters of system knowledge (plan 009): `MacKnownFolders`, `MacInstalledApps` with its own `Info.plist` and code-signature readers, `MacRunningApps`, `SystemClock`, `StartedClock`, `ProcessCommandRunner`, the bundle reader (`bundle_facts/`: the measure, Spotlight's last-used date, Homebrew's casks; plan 036) | core, macos   |
| `fresher-store`       | `crates/store`   | the only crate that touches the app's data folder: atomic versioned files, `settings.json`, the scan cache, the cleanup journal                                                         | core          |
| `fresher-cleaner`     | `crates/cleaner` | the only code that removes: rehearses and executes a confirmed plan through its guard, Trash first, puts back ([`removal.md`](./removal.md))                                            | core, macos   |
| `fresher-engine`      | `crates/engine`  | module registry, event bus, one operation at a time, the scan session, settings, the disk queries, the plan builder, the cleanup and its history, the Apps view and the uninstall plan (`apps/`, plan 036)                                        | core          |
| `fresher-module-<id>` | `modules/<id>`   | one feature: descriptor, rules or code, texts; `jetbrains` (older IDE versions' folders); `temp` (old items of the temporary folders); `logs` (the old log files and crash reports of the user's `~/Library/Logs`, one finding per file, group System); `dev-caches` (the package managers', Go's and Xcode's caches); `app-leftovers` (the data of apps no longer installed, group Apps); `app-caches` (the `Caches/<bundle id>` folders of installed apps, emptied while the app is closed, and the cache folders a table ties to an installed browser or web-engine app, group System); `downloads` (unfinished downloads, installers of installed apps, old macOS installers and big files not opened in `~/Downloads`, group Your files, only once the folder is allowed); `stale-projects` (the artifact folders of projects nothing changed in for a year, and the Rust build caches unused for a week, group Development; decided from the projects in `finish`); `local-ai-models` (the models of Hugging Face, LM Studio and Ollama, group Development; one finding per model, decided in `finish`; an Ollama model is one finding of several paths, its manifests and the layers no other model names); `unused-toolchains` (the rustup toolchains and, behind a switch off by default, the nvm Node versions nothing points at, group Development; decided in `finish` from the pointer and pin files it reads); `xcode` (Xcode's device support, previews, simulator caches and archives, group Development; four categories, decided in `finish` while Xcode and the Simulator are closed); `demo` and `trial` are dev-only (debug builds)                                                | core, rules   |
| `fresher-testing`     | `crates/testing` | fixture trees, fakes of the ports, the module harness and the category contract (dev only)                                                                                              | core          |
| `fresher-sandbox`     | `crates/sandbox` | the case files of a feature, their checks, the sandbox layout and its generator (a binary); also the demo worlds and the scenarios of the recordings (plan 038, exported to the UI with `ts-rs`); dev only, a dependency of the app and the tests alone                                                                                     | core, testing, macos, store |
| `fresher-e2e`         | `crates/e2e`     | end-to-end and golden tests of the core (tests only)                                                                                                                                    | anything      |
| `fresher-app`         | `app/tauri`      | the composition root: real adapters, module registration, commands, the event channel                                                                                                   | anything      |

The "May depend on" column lists normal dependencies; `fresher-testing` is allowed as a dev-dependency of any crate,
and a module never depends on another module. The column is the constant `TABLE` (and `MODULE_ALLOWED`) of
[`scripts/check-crate-deps.ts`](../../scripts/check-crate-deps.ts), which reads `cargo metadata` and fails on any edge
outside it, and on a crate that is not in it (`npm run check:crate-deps`, in the pre-push hook and CI;
[`../quality.md`](../quality.md)). The scanner may depend on `fresher-macos` but does not: it reads folders through the
`FolderReader` port (its own portable `StdReader` in tests), and the app injects the macOS reader.

Rules of the layout:

- The core crates know nothing about Tauri; the shell knows nothing about rules' internals.
- **The core, the engine and the modules know nothing about the operating system** (the user's decision of
  2026-11-15: macOS first, Windows not ruled out). Every OS call is an adapter behind a port, and only `fresher-macos`
  and the adapters built on it are macOS-specific; a module names a known folder (plan
  [009](../plans/009-system-knowledge/readme.md), `KnownFolders`, `Pattern::Under`), never a macOS path such as
  `~/Library/Caches`.
- Only `fresher-cleaner` may call a deleting file system function (`std::fs`'s, or `fresher-macos`'s `remove_at`),
  at its single deletion point (enforced by Clippy's `disallowed-methods`; [`removal.md`](./removal.md)). The app
  gives the system's `Trash` to the cleaner and to nothing else; the engine reaches the cleaner only through the
  `Cleaner` port, with a confirmed plan.
- A test gets `Reach::Sandbox` (or a sandbox folder): the cleaner acts only below a folder that holds the sandbox
  marker. A development build outside a sandbox gets `Reach::DryRun` — every check on the real disk, no act, each item
  ends "a dry run" — and `DryRunTrash`, a Trash that refuses every call. Only a release build's composition makes the
  token for the real disk ([`removal.md`](./removal.md), "Reach and the sandbox").
- **Sandbox mode** (plan [007](../plans/007-safe-development/readme.md), a debug build only, `app/tauri/src/sandbox.rs`):
  `FRESHER_SANDBOX=<folder>` makes the composition (`sandbox_engine`) live entirely in a folder the sandbox generator
  made: its `home/` is `~` and the known folders (its `private/var/folders/xx/sandbox/T` and `private/tmp` the two
  temporary places), `.fresher-data/` the data folder, `home/.Trash` the Trash, the cleaner is
  `Reach::SandboxFolder`, and the protection is the real Mac's list joined with the sandbox home's and its temporary
  places' (`Protection::joined`). The walk's reader is the bulk reader seen through the sandbox's stated facts
  (`FactsLens` over `world.toml`'s `[[entry]]` lines, plan 017): an old status change, an owner, a flag or a socket
  that cannot be made on disk; the cleaner reads the disk. The scanner's avoid list is the places that prompt of both homes, so no scan raises a
  system prompt. Volumes, open files and commands stay real, read-only; the access is the stand-in `UnknownAccess` (the
  sandbox's world is not the Mac's, `docs/hacks/`); the running apps are never the Mac's
  but the sandbox's declared ones (`world.toml`, none without it: plan 016), Spotlight answers only the sandbox's
  listed apps and the apps `world.toml` declares elsewhere, and cannot be asked when it says `spotlight = "off"` (plan
  019), and the clock starts at the moment the
  generator wrote into `world.toml` and runs from there (`StartedClock`; the system's without one), so the case files'
  ages hold however long ago the sandbox was generated (plan 016); the process's IO policy is set
  only by the app's start, never by `sandbox_engine`, so the autotests run the same composition on fakes. The window
  shows a banner naming the sandbox and listing what its world simulates (`Build::Sandbox.simulated`, a list of `Simulated`: running apps always, Spotlight,
  access and installs when the world states them: "Simulated: open apps, Spotlight, access, installs."; a development build outside one: "nothing
  can be removed outside a sandbox"; nothing in a release build). A release build has no such branch.
- **Demo mode** (plan [038](../plans/038-demo-recordings/readme.md), a debug build only, `app/tauri/src/demo.rs`, beside
  sandbox mode): `FRESHER_DEMO=<absolute scenario path>` with `FRESHER_SANDBOX` (refused without it) sizes and places
  the window and registers two commands under `#[cfg(debug_assertions)]`: `demo_play` returns the scenario and the
  private texts (and writes the window's rectangle for the recorder), `demo_finished` writes the result and ends the
  app. Neither exists in a release build; the core is not involved.
- **The development run record** (plan [026](../plans/026-dev-run-record/readme.md), debug builds only,
  `app/tauri/src/dev_runs/`): when `FRESHER_DEV_RUNS` names a folder (set by `npm run dev`), the composition begins a
  run; the app subscribes it to the engine before the last scan is restored and wraps the generated invoke handler, so
  every action the UI sends and every event but the scan's progress go to an append-only `.events.jsonl`, and a
  snapshot `.json` is rewritten after each scan and cleanup. The core, the engine and the modules know nothing of it.
- External programs (the Docker CLI, `tmutil`) are called through the `CommandRunner` port, so tests use fakes.
- **The system's facts** (plan [009](../plans/009-system-knowledge/readme.md)): the engine's `Ports.system`
  (`SystemPorts`: known folders, installed apps, running apps, install records, small files) becomes one lazy `SystemFacts` per scan, handed to
  every module as `cx.system()`; the engine adds its own interests for the projects (`ProjectFinder::interests()`, in
  the core, so the module harness finds the same projects) under `ModuleId::system()` and, after the walk, asks the walk
  what it saw of them (`DiskWalk::subtrees`) and sets the projects before any `finish`. Nothing of it is written to disk.
- The UI gets everything it shows (names, descriptions, sizes, danger levels, the text of a refusal) from the core, as
  dictionary keys with arguments; the UI's own dictionary holds only its chrome. A technical detail (a tool's own
  message) travels as the arg `detail` (`DETAIL_ARG`), which no template names: the sentence stays plain and the UI
  shows the detail only on demand. An error's `Display` form is for logs and never reaches the UI.
- The IPC types are generated from the core's Rust types (`ts-rs`) into `app/ui/src/ipc/bindings.generated.ts`; a
  test keeps the file up to date.

## Ports

| Port             | Implemented by                                                                                                  | Fake for tests                                              |
| ---------------- | --------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| `Clock`          | `fresher-system`'s `SystemClock`; in sandbox mode its `StartedClock`, from the moment in `world.toml`           | `FakeClock`                                                 |
| `Trash`          | `fresher-macos`'s `MacTrash` (the system's Trash call; put back by `renamex_np`); in a development build outside a sandbox `DryRunTrash` (refuses every call) | `FakeTrash` (a Trash folder inside the fixture)             |
| `OpenFiles`      | `fresher-macos`'s `MacOpenFiles` (the user's own processes, `libproc`)                                          | `FakeOpenFiles`                                             |
| `CommandRunner`  | `fresher-system`'s `ProcessCommandRunner` (`std::process`, no shell; a 60 s timeout; the caller's cancel token kills it) | `FakeCommandRunner`                                         |
| `KnownFolders`   | `fresher-system`'s `MacKnownFolders` (the macOS layout of a home folder; opens nothing)                         | `FakeKnownFolders` (`home_layout` of a fixture root)        |
| `InstalledApps`  | `fresher-system`'s `MacInstalledApps` (the apps folders and one below, the older JetBrains Toolbox's folder four below; an app link read where it points; `Info.plist`, code signature; what it cannot read in `unread`; `find` by bundle id through Spotlight, `mdfind`, or a sandbox's declared apps; `bundle`: one bundle's facts (the measure by `lstat`, the entry's owner and flags, Spotlight's last-used date, the Homebrew cask that links it) through `BundleReader`, `BundleReader::stated` in sandbox mode; `is_as_listed`: the canonical-path check of the uninstall; `search_complete`: Spotlight knows the Finder and every mounted volume is indexed, `mdutil -s` through its own 10 s runner, a network volume never asked) | `FakeInstalledApps` (`partial`, `failing_search`), `installed_app` |
| `RunningApps`    | `fresher-system`'s `MacRunningApps` (`fresher-macos`'s process table, each executable's bundles)                | `FakeRunningApps` (a script), `running_app`                 |
| `InstallRecords` | `fresher-system`'s `MacInstallRecords` (the installer receipts in `/var/db/receipts`, read only; the version of macOS from its own version file) | `FakeInstallRecords`, `changed_at`                          |
| `Cleaner`        | `fresher-cleaner`'s `DiskCleaner`                                                                               | `FakeCleaner` (records the executed plan)                   |
| `CleanupJournal` | `fresher-store`'s `FileCleanupJournal`                                                                          | `MemoryCleanupJournal`                                      |
| `DiskWalk`       | `fresher-scanner`'s `Scanner`, with the macOS reader in the app; `subtrees` answers what the last walk saw below a folder (sizes, newest folder time) before the commit | `NaiveWalk` on a fixture tree                               |
| `FolderReader`   | `fresher-macos`'s `MacFolderReader` (bulk read); `fresher-scanner`'s portable `StdReader`; in sandbox mode `MacFolderReader` through `FactsLens` | `StdReader` on a fixture tree, through `FactsLens` for its stated facts |
| `SmallFiles`     | `fresher-core`'s `ReaderFiles` over the scanner's reader, as the app composes it (in sandbox mode over its stated lens); behind `SystemFacts::small_file`: a regular text file of at most 4 KiB, never a link | `DiskFiles` (the standard library, on a fixture tree), `NoFiles` (every read `Unreadable`) |
| `ChangeJournal`  | `fresher-macos`'s `MacChangeJournal` (FSEvents)                                                                 | `FakeChangeJournal` (a script)                              |
| `Volumes`        | `fresher-macos`'s `MacVolumes`: `identify`, `available` (what a cleanup measures), `space` (the map's volume)   | `FakeVolumes` (`with_space`, `answering_available`)         |
| `ScanCache`      | `fresher-store`'s `FileScanCache`                                                                               | `MemoryScanCache`                                           |
| `DiskState`      | `fresher-scanner`'s `TreeState`, returned by `DiskWalk::commit`                                                 | `FixedDiskState` (a tree from paths and sizes)              |
| `SettingsStore`  | `fresher-store` (`FileSettingsStore`)                                                                           | in memory, in the engine's tests                            |
| `AccessProbe`    | `fresher-system`'s `MacAccess` (`fresher-macos::privacy`: the guarded places, checks that never prompt, its own tries remembered); in sandbox mode the app's `UnknownAccess` | `FakeAccess` (any state, guarded places, tries)             |
| `SystemUi`       | `fresher-system`'s `MacSystemUi` (`/usr/bin/open` with a settings pane's address, `-R` and a path, or `open_folder`: a folder's absolute path, which Finder opens) | `FakeSystemUi` (records every call)                         |
| `FinderSettings` | `fresher-system`'s `MacFinderSettings` (`defaults read com.apple.finder FXRemoveOldTrashItems` through the `CommandRunner`: `1` is After30Days, `0` or a missing key Never, anything else Unknown; it never writes a setting) | `FakeFinderSettings` (a fixed answer)                       |

The stand-ins until a later plan are listed in [`../hacks/`](../hacks/README.md).

## Data

No database: the app keeps its own state in small plain files, with nothing to install.

- **The data folder** is `~/Library/Application Support/io.github.alexsergey.fresher/` — never inside a scanned root. The app
  passes it to `fresher-store` (`DataFolder::open`); the store never looks it up itself.
  A development build (`npm run dev`) runs with `app/tauri/tauri.dev.conf.json`: the identifier
  `io.github.alexsergey.fresher.dev`, so its data folder, single-instance lock and permission grants are separate; a
  debug build with the release identifier refuses to start, and so does a build without debug assertions with the
  development identifier (it would really remove and record nothing).
- **`settings.json`**: `{ "format": "settings", "version": 1, "values": { "<key>": <value>, … } }`. Values are checked
  against the settings schema in the core (pure); an unknown key or a broken value falls back to its default and is
  reported, never a crash. Each setting carries its **group** (where the settings screen shows it: `general`, `scan`,
  `ignored`, `permissions`, or `module` for every module's) and its **effect** (`now`, or `next-scan`: a change marks
  the last scan, and the UI offers "Scan again to apply it"; the app never rescans on its own). The kinds are `switch`,
  `number`, `choice`, `folders` (`~`, `~/…` or a plain absolute path, never a protected folder), `ids` (of
  categories, modules or groups, checked by what they hold: `IdOf`) and `paths` (the ignore lists). `reset_settings` puts every setting but the ignore lists back to its default in one save.
- **The ignore list** (`core.ignore.modules`, `core.ignore.groups`, `core.ignore.categories`, `core.ignore.paths`):
  the engine keeps a scan's ignored findings in memory but leaves them out of every answer and plan, so a restore
  shows them again without a rescan; the cleaner gets the ignored paths as protection. An ignored group hides every
  registered module of it (resolved through the registry when the lists are taken), whatever the module's own entry
  says; `ignore` refuses a module or a group nobody registered. Ignoring a module is not `<module>.enabled`: the
  module still runs, and its findings are only hidden. The overview says which modules are hidden and why
  (`Overview.hiddenModules`, each `{ module, by }`, `by` the module's own entry or its group), so the UI never
  works it out. `ModuleInfo.group` carries each module's group and `ModuleInfo.choices` its choices (`ChoiceInfo`: a name and the categories it selects, with its two texts); the engine's `hidden_modules` resolves the lists. `ignore` and `restore_ignored` change the lists through the same check and
  save as `update_setting`. The app's own commands beside the engine: `pick_folder` (the system's folder picker,
  `tauri-plugin-dialog`, called from Rust only) and `get_own_data` (the footprint below). Since plan 039 two more: `get_agreement` (the text, its version and whether
  the current one was accepted; a read) and `accept_agreement(version)` (an action; `AgreementChanged` for another
  version). The acceptance is two settings of a group the screen never shows (`core.agreement.version` and
  `core.agreement.accepted-at`), written only by `Engine::accept_agreement`; every prepare and every confirm checks it
  (`BlockedReason::Agreement`, after the settings file and before the journal; [`removal.md`](./removal.md), "Fail
  closed").
- **The High switch** (plan 042; `HighLevel::{Offered, Withheld}`, `crates/core/src/danger.rs`): a required argument
  of `Engine::new` and `Engine::with_cleanup`, set by the app (`app/tauri/src/compose.rs`, `sandbox.rs`): `Withheld`
  in every build of the first release, development and sandbox mode included; tests and the sandbox autotest pass
  `Offered` (a case file with `high = "withheld"` runs as the app does). `Withheld` (`crates/engine/src/withheld.rs`)
  leaves out of every answer a finding whose own level or category is High: pages, `finding`, totals, the found size,
  `explained_under`, "Can be freed" on the map, a module's categories and choices (an empty choice dropped), its
  left-outs, insight parts at High, an app page's footprint items of a High kind and an Apps row's parts of a High
  kind; and it refuses with `CommandError::HighWithheld` every prepare holding a High item (first in
  `rehearse_and_keep`, before the agreement and settings checks), `pick_own_item` and `prepare_uninstall(_,
  Everything)`. `AppInfo.high` reports it to the UI. The rule:
  [`../restrictions/deletion-safety.md`](../restrictions/deletion-safety.md), "High is withheld in the first release".
- **Atomic writes**: a temporary file in the same folder, `fsync`, then a rename.
- **The version policy**: every file the app writes carries `format` and `version`. What holds the user's decisions or
  history (settings, the cleanup journal, history points) is migrated forward one version at a time; what can be
  rebuilt (the scan cache) is discarded on a version change. A file from a newer version of the app, or one that could
  not be read, is never overwritten: the settings are read-only and the defaults are used in memory. A broken settings
  file is moved aside to `settings.broken.json` on the first save.
- **The footprint**: one folder, a budget of 50 MB (`FOOTPRINT_BUDGET`), measured by `DataFolder::footprint()` without
  following links.
- **The scan cache** (`FileScanCache`, behind the core's `ScanCache`): fixed slots, one per scan scope —
  `scan-1.bin`, `scan-2.bin` (current) and `scan-1.previous.bin`, `scan-2.previous.bin` (previous). A save of a
  scope renames its current file over its previous one and writes the new one atomically; a new scope takes a free
  slot or the one written longest ago. Each file is a binary envelope — `FRSHSCAN`, its format version, the scope
  key, the stored length and a CRC-32 — around the scanner's body compressed with deflate level 1. A file is capped at
  8 MiB on disk (`CachePolicy`), so the cache takes at most 32 MiB of the footprint budget. A broken, foreign, older
  or newer file is discarded (one full scan, never a wrong result) and never overwritten except by a save into its
  slot. The cache never deletes a file: rotation is a rename, so it never holds more than its four files.
- **The cache body** (the scanner's `codec`): its own tree format version, the time the state was taken, the roots with
  their volumes and journal positions, the avoid list, the interest fingerprint and the claims table; then every node
  of the frozen tree depth-first, children in name order — its name front-coded against the previous sibling, its
  flags, child count, own bytes, modification time, hits, for a node with hits its status change, access and inside,
  owner and access (one byte), all as variable-length integers — then the hard link and clone groups. Totals are not
  stored: the reader sums them again. A body that does not read whole is refused (one full scan). A body whose file
  would pass the cap (measured compressed, as the store writes it) has its smallest subtrees with no hit, claim or
  unreadable folder collapsed into their folder (totals and counts only) until it fits; one that does not fit even
  then is not saved.
- **Rescan and restore** (the scanner's `rescan`): every committed scan is saved to the cache with each root's journal
  position, taken before the walk read anything. A scan the user starts (`WalkMode::Auto`) loads the cache, checks it
  is of the same roots, volumes, avoid list and interests, asks the change journal what changed since (`ChangeJournal`,
  FSEvents), reads again only those folders with their parents and the unreadable ones, copies the rest of the tree,
  then reads a sample of the copied folders: one difference, a gap in the journal, a position older than two weeks or
  too many changes, and the scan is full (`WalkSummary::fallback` says why). At launch the engine restores the last
  state (`WalkMode::Restore`): the cache only, no folder read, the modules with interests run over its matches. The
  places walked outside the roots are never served from the cache: a rescan walks each again in full, whatever the
  journal says (it reports no read, and a status change through a hard link elsewhere moves no event in the folder),
  and a restore emits nothing inside one.
- **Places outside the scan roots** (`WalkRequest.places`, the engine's `places.rs`): the user's temporary folder and
  `/private/tmp` (`KnownFolder::is_walked_place`), when a registered module's interest is anchored in one, enabled or
  not, it lies outside the home, and no scan root holds it or lies in it, are walked as roots of their own after the
  scan roots: only the interests anchored in a place match there, a folder of another user is listed as itself and
  not entered (`NotScanned::OthersFolder`, which leaves the scan complete), and `<UserTemp>/TemporaryItems` is never
  opened (`NotScanned::SystemItems`, never a reason to ask for Full Disk Access). A place that is missing or cannot
  be read is not scanned, with why, and never stops the walk. The places are not in the cache's key (changed places:
  `OtherRoots`). The cleaner counts the last scan's places as roots; the engine skips a place the cleaner's
  protection (`Cleaner::protection`) refuses for ever; the storage map shows each place as a root of its own, named
  by its known folder (`VolumeMap.places`).
- **The scan in memory** (the scanner's `walk`, `tree` and `matcher`): one parallel pass, a worker per core (at most
  16) taking folders from one queue, each folder read in one bulk call per batch of entries, every entry matched
  against every module's interests with one lookup, and one aggregator building the tree. Files under 1 MiB are
  folded into their folder (a small hard-linked or cloned file keeps only its identity, in the link and clone tables);
  larger files and matched files are nodes of their own; hard links and clones are counted once. Each node keeps its
  entry's access for the user (the owner of the home folder) in one byte — owned, writable by the mode bits, the
  three locks — read in the same bulk call; a match's entry facts (removability) come from its node and its
  parent's. The committed tree is the `DiskState`.
- **Queries** (the engine's `disk.rs` over the last committed `DiskState`): `disk_roots`, `disk_node`,
  `disk_children` (by size or by name, paged), `largest_files`, `items` (plan [035](../plans/035-large-files/readme.md): the files and folders the Largest screen
  offers, in one pass), `search_names` (case ignored, in memory, a lowercase
  copy of the names built by the first search) and `unreadable_places`, every limit capped at 500. A query reads the
  state's `Arc` outside the engine's lock, so it never blocks a scan; `NoScan` before any commit, `StaleNode` for an
  id of another state. The IPC commands come with the screens that use them (plans 003 and 012).
- **The storage map** (the engine's `storage/`; the section "The storage map" below): `storage_map` in three phases
  — before any scan the configured roots' volumes, all "Not scanned yet"; during a scan the findings so far (ignored
  ones left out) as "Explained" on the volume of their root; after a commit the full map of the state — one map per
  volume holding a root, the home's first. `storage_meaning` (one volume's by-meaning groups, ending with its parts)
  and `storage_place` (a page of the by-place view, opened through the folders that are almost all one child, or at
  the folder asked for a step of the breadcrumb: `Descent::{Through, Stay}`) read the last state; a place is refused
  with `Busy` while a scan runs. The `Shares` of a state are built by the first query after a commit or a change of
  the ignore lists and kept beside the state; the volumes (`Volumes::space`, through the cleanup ports) are read at a
  scan's start and commit, or once before the first scan, never per query, and the committed volumes are kept beside
  the committed state.
- **The webview keeps nothing**: the window runs `incognito`.
- **The cleanup journal** (`FileCleanupJournal`, behind the core's `CleanupJournal`): an append-only JSON-lines file,
  `cleanups.jsonl`, private to the user (`0600`). A header line `{"format":"cleanup-journal","version":2}`, then one
  record per line tagged by `type` — `begin` (every item of the cleanup: its path, identity, the parts after the
  subject of an item of several paths (`more`, each with its identity; version 2), action, category, finding key,
  reason, danger level and sizes), `item` (its outcome, where it landed in the Trash and, for an item of several
  paths, how many parts went: `went`), `end` and `put-back` (with the parts of an item of several paths it brought
  back: `back`) —
  in forms of the file's own, not the IPC types. A path is a string when it is UTF-8 and `{"bytes":"<hex>"}`
  otherwise. Write-ahead: `begin` is synced before it returns (and a new file's folder before it), the engine syncs
  the items every 50 and every second, `end` and `put-back` are synced. A torn last line (a crash) is skipped and
  counted when reading, and the next record starts on a new line; a `begin` with no `end` reads as interrupted. Ids
  continue from the highest in either generation. A file from a newer version or one that cannot be read makes the
  journal read-only, and a cleanup is refused; a file that is not a journal is renamed to `cleanups.broken.jsonl` at
  the next write; an older version is read through the migrations line by line and becomes the previous generation
  at the next write. **Rotation**: when a `begin` finds `cleanups.jsonl` at 4 MiB or more, the file is renamed over
  `cleanups.previous.jsonl` and a new one starts, so a cleanup never spans two files and the journal keeps about
  8 MiB of the 50 MB budget (a hundred cleanups of a hundred items take 6.9 MB); the oldest generation drops off by
  that rename. History reads the previous generation, then the current one, line by line.
- **The Trash view** (plan [037](../plans/037-trash/readme.md); `TrashView`, one `TrashLine` per Trash folder that
  holds something, built by `Engine::trash`, read by the `get_trash` command). A line is `index`, `volume` (a name,
  none for the home's own), `waiting` (what is there, in bytes on disk, or none when it is not known),
  `fromCleanups` (what this app's cleanups put there) and `goneBy` (a moment). The folders are the home's Trash, the
  Trashes of the volumes (`<mount point>/.Trashes/<uid>`) and, in sandbox mode with `--real-trash`, the Mac's home
  Trash; an item that landed elsewhere is never counted. `fromCleanups` comes from the cleanup journal: every item of
  every cleanup still found in a known Trash, each item's allocated size once; an item put back, emptied from the
  Trash or replaced counts nothing. `waiting` is the last scan's measure of the folder (a scanned node of its state)
  plus what cleanups moved there since that scan, never below `fromCleanups`; with no scan of the folder it is none.
  There is no walk of the Trash of its own, so the first number stays the scan's measure until the next scan
  ([`../known-issues/app-trash-waiting-stale-after-emptying.md`](../known-issues/app-trash-waiting-stale-after-emptying.md)).
  `goneBy` is the latest end of a cleanup in that Trash plus 30 days, only when Finder's setting says After30Days and
  the date is still ahead. A line is left out when there is nothing waiting and no share. No path crosses the IPC:
  the UI names a Trash by its `index`, and `open_trash(index)` refuses an index that is not in the last answer
  (`unknown-trash`). The app never empties a Trash, and nothing in the core can: `open_trash` only asks the system to
  open the folder in Finder.
- History points (plan 010) choose their files in their own plan, under the same policy.

## The storage map

Plan [012](../plans/012-storage-map/readme.md): where the space of a volume went, in parts that add up to its
capacity. The core holds the types (`StorageMap`, `VolumeMap`, `MapPart`, `PartKind`, `SummaryPart`, `RemainderReason`,
`MeaningGroup`, `PlaceView`, `DiskSpace`); the engine builds them; the UI draws them and decides nothing.

- **The parts**, in this order, each with a core-given name, description and danger tone: Explained (the findings of
  the modules; the only part offered for cleaning), Your folders (scanned, claimed by no module), Not scanned yet
  (during a scan only), the remainder (used space the scan did not measure), the system's volumes, the virtual
  memory volume, the user's other volumes by their own names, the disk's own space (capacity minus free minus every
  mounted volume's used space), and Free. `VolumeMap::new` is the only way to build a map and refuses parts that do
  not add up to the capacity or come out of order.
- **The summary** (`VolumeMap.summary`, `SummaryKind`, plan 042): what the disk card draws, the parts merged in the
  core into five, always in this order, each with a core-given name and one-sentence description
  (`core.storage.summary.<kind>.*`): Can be freed (Explained), Your files and apps (Your folders, Not scanned yet, the
  remainder, the other volumes), macOS (the system's volumes), System space (the virtual memory volume and the disk's
  own space) and Free. The UI draws only the summary; the detailed parts stay in the core for the by-meaning and
  place queries.
- **The remainder always says why**: its reasons are folders the scan could not read (with the permission that would
  show them), local snapshots and the file system's bookkeeping, and files that changed while the scan ran. It is
  "Outside the scanned folders" when the roots do not cover the volume. A part with no reason is not drawn; there is
  no catch-all part ([`../rules.md`](../rules.md), rule 3).
- **The volume** (`Volumes::space`): capacity (the container's, `f_blocks`), this volume's own used space
  (`getattrlist`, `ATTR_VOL_SPACEUSED`, the only `unsafe` for it, in `fresher-macos`'s `ffi`), free, purgeable
  (shown beside Free, never counted in it), its kind, and the neighbours: the mounts of the same disk number with the
  same `f_blocks`, each with its role (system, virtual memory, other).
- **One claim per byte** (the engine's `shares.rs`, `attribution.rs`): every owned node of the state
  (`DiskState::owned`, and `find` by path) is counted by the claims' rule on the tree's own sizes. The deepest owner owns
  its subtree, an outer owner keeps its sizes minus the owners nearest inside it, a node inside a counted owner is wholly
  explained, and an owner whose category or path is ignored is unclaimed: its bytes fall to the user's folders, never to
  an outer owner.
  Apps are found by bundle id (`NewFinding::for_app`: a module that knows the app names it, see
  [`modules.md`](./modules.md)), so an app's data folders, its bundle and its findings are one group.
- **By meaning** (`meaning.rs`, scoped to one volume's roots): the apps largest first, the categories no app names, the
  named places (the Trash) and the largest of the user's folders, then Your folders; every byte of the roots is in
  exactly one leaf, and the volume's other parts are appended at the end (`append_volume_parts`). **By place**
  (`place.rs`): a folder's children largest first, each with its explained size; the UI dropped it with plan
  [027](../plans/027-scan-screen/readme.md) and it is back as Largest, Folders (plan
  [035](../plans/035-large-files/readme.md), the section "The largest files" below) with its way up as a breadcrumb.
- **The three phases** (`query.rs`): no scan, scanning and complete (the phase is in the map, `MapPhase`); a query
  takes what it needs under the results' lock and builds after letting it go, so none blocks a scan.
- **Budget**: the shares of a scan of the `LARGE` tree take under 50 ms and 5 MB
  (`crates/e2e/tests/storage_budget.rs`, run by hand; [`../testing.md`](../testing.md)).
- The commands are `get_storage_map` and `get_storage_meaning(volume)` (`app/tauri/src/storage_commands.rs`;
  `get_storage_place` was removed by plan 035, whose `get_largest_place` shows a folder's children). The storage map's sandbox case file (plan 012/05) is not built yet: it
  waits for plan [007](../plans/007-safe-development/readme.md) to be merged into `main`.

## Apps and uninstall

Plan [036](../plans/036-uninstall/readme.md): the user picks an installed app, sees everything tied to it by bundle id
and uninstalls it as one item. An engine view over the last committed scan, like the storage map; not a module (an app
is not junk, and a module's findings land on Scan). The core holds the types and the texts (`crates/core/src/apps/`:
`AppSummary`, `AppDetail`, `FootprintItem`, `PartKind`, `KeptReason`, `NotOffered`, `UninstallChoice`, the two
categories), the engine builds them (`crates/engine/src/apps/`), the UI renders them.

- **The list** (`list.rs`): the apps of the installed list over the last committed scan, never any of Apple's
  (`com.apple.`, the system's apps folder), never Fresher's own bundle, and only a bundle directly in `Applications` or
  `~/Applications` or one folder below that the protection passes. Each row is offered or carries one reason, the first
  that holds: not measured, installed by a package manager (a Homebrew cask link), needs an administrator, locked or
  protected (`Removability::of` the bundle's entry), open (the app or a helper is running, asked again at each listing
  and at prepare). When the installed list could not be read the view says so and lists nothing.
- **The footprint** (`footprint.rs`, `places.rs`): pure over the committed state and the installed list. A const table
  of places (caches, logs, saved state, launch agents, web data, settings, support, containers, group containers); a
  part is a first-level child named by the app's bundle id or a helper's (`BundleId::names`, the longest naming id
  wins), never by name, a vendor prefix or a look-alike. A part is kept, listed and never in a plan, when another
  installed bundle names the id, the group container's team is shared, the node is a link, a container needs other
  apps' data and it is not granted, the node was not read in full or the user ignored it. When the installed list is
  incomplete every part but the bundle is kept.
- **The measure** (`cache.rs`): the bundle's size, owner and last-use date are read once per committed state, on
  demand, on at most four threads, outside every engine lock, and cancelled when a scan starts; a gate keeps two
  listings from measuring twice. "Last used" is the later of Spotlight's date and the newest change of the app's own
  caches, saved state, settings and support, never the executable's access time.
- **The uninstall plan** (`uninstall.rs`): `prepare_uninstall(app, choice)` builds one engine-made finding of the
  category `core.uninstall-keep-data` (Medium, the default choice) or `core.uninstall-everything` (High): the bundle,
  then the parts the choice takes, as one item that moves together (`MoveTogether`; a bundle alone is a plain move to
  the Trash). At most 64 paths, else the uninstall is refused, never truncated. The finding is never in the results, on
  Scan or in a total; the ordinary prepare, rehearsal, confirmation, journal and put back run unchanged.
- **The context** (`cleanup/context.rs`): the plan's context holds the apps folder as a root only for an uninstall;
  every other plan (a cleanup, a Largest pick) refuses both apps folders (`Protection::with_refused`), and put back adds
  no refusal. The protection itself lets through only a whole `.app` bundle in an apps folder
  ([`../restrictions/deletion-safety.md`](../restrictions/deletion-safety.md)). The plan is made from the listing and
  refused unless `InstalledApps::is_as_listed` holds: the canonical path equals the listed path byte for byte, and the
  bundle is a real folder with `Contents/Info.plist`; it is asked at prepare and again at confirm, with the running
  apps asked anew.
- **The paths that went**: the paths of an uninstall item that moved (the first parts of a partly done one) leave the
  results and join the state's gone paths, so the list and Largest hide them until the next scan.
- **App Management**: a capability that is never probed. `NotAsked` until the first uninstall, then learned for the
  session from the outcome (`apps/management.rs`; [`permissions.md`](./permissions.md)).
- **The commands** are `get_apps(query)`, `get_app(app)`, `reveal_app(app)` and `prepare_uninstall(app, choice)`
  (`app/tauri/src/apps_commands.rs`); the engine reveals only a listed app's own path.
- **UI**: the screens `apps` (the list, filters, "not used for") and `app` (the footprint item by item, the two choices
  as radios, the notice, Uninstall...); the sidebar lists Scan, Largest, Apps, Review, Cleanups and Settings. The
  cleanup state carries an `origin` (`selection` or `uninstall`): a finished uninstall keeps the Scan selection, and a
  refusal of the confirmation shows on Review for either origin.
- **While High is withheld** (plan 042): an app's page offers only "Keep my settings" and lists no file of a High
  kind, an Apps row shows no part of a High kind nor counts its bytes, and an uninstall of everything is refused.

## The largest files

Plan [035](../plans/035-large-files/readme.md): the biggest things the user owns that no module explains, listed so
the user can choose one. The core judges, the UI renders.

- **Two queries** (the engine's `largest/`): `largest` (the Files list: items below the home, ranked by allocated size,
  ties in path order, filtered by kind, place and `Unchanged`, paged by `MAX_PAGE` = 100) and `largest_place` (the
  Folders view: a folder's children, the home first, a folder opened through the breadcrumb). The items come from
  `DiskState::items` (one depth-first pass over the committed tree, never entering owned nodes, links, hidden names,
  `Library`, `.Trash` or `.app` bundles), at the minimum of the setting `core.largest.min-size` (10, 50, 100, 500 MB or
  1 GB, default 100 MB), at most `MAX_ITEMS` = 10 000 kept, computed once per state and minimum and kept beside the
  state. Cloud-only files are not listed (they take no space).
- **The judgement** (`judge.rs`): every row is offered or withheld with one reason, in a fixed order (explained, library,
  trash, hidden, app, link, not seen, standard folder, protected, holds explained, read in part, ignored, offered). One
  function, `judge_reached`, answers for the Files rows, the Folders rows, the pick and the plan: it also asks every
  folder above the row whether it opens, so a row inside a package, a claimed folder or a protected place is never
  offered. A folder holding an ignored path is withheld but still opens; a folder holding any claimed node is
  withheld.
- **The pick**: `pick_own_item(node)` is refused (`NotOwnItem`) unless the judgement answers offered at that moment.
  The answer is a finding of the core's category `core.own-file` (danger High, `Removal::MoveToTrash`), kept in
  `Results.own` apart from the scan's findings: never in a total, a card, the map or a findings page. The plan builder
  adds the picks still offered (judged again at prepare and at confirm); a removed pick leaves the lists until a
  rescan.
- **The commands** are `get_largest`, `get_largest_place` and `pick_own_item` (`app/tauri/src/largest_commands.rs`).
- **UI**: the screen `screens/largest/` (`Screen` `largest`; Files and Folders as tabs, the filters, a selection bar) and
  the component `product/own-item-row/`; the sidebar lists Scan, Largest, Apps (plan 036), Review, Cleanups and Settings.
- **Hidden while High is withheld** (plan 042): a pick is High, so the sidebar leaves Largest out and the core refuses
  `pick_own_item` (`HighWithheld`). The two queries are still answered; nothing in the UI asks them.

## Problems and reasons

Plan [011](../plans/011-diagnostics/readme.md). No raw technical error reaches the user. A problem the user sees has
one of three forms, each with its carrier:

- **A refusal** (a command that could not run): `Refusal { text }`, built from a `CommandError` by the app; the UI
  renders `text`.
- **A reason** (why something is missing, skipped or blocked): a typed enum of the core that turns into a `Text`
  (a dictionary key with args) in the answer that holds it.
- **A module's problem** (its part failed, or it left an item out): `ModuleError::Failed { reason }` or
  `out.left_out(LeftOutItem)`, each with a text the module declared in `ModuleDescriptor::with_problems`.

The rules of the shape:

- A `Text` is the only thing the UI shows. The sentence says what happened and what to do next, in plain English.
- A technical detail (a tool's own message) is the arg `detail` (`DETAIL_ARG`); no template names it, and the UI shows
  it only behind "Details" (`product/problem-text`).
- An error's `Display` form is for the log and never reaches the UI; a test serializes every `CommandError` and
  checks that no refusal holds its `Display`.
- A module's declared key must have a text, name no `{detail}`, and be used by a run; an undeclared failure or
  left-out reason fails the module's contract test (`UndeclaredProblem`, `DetailInSentence`).

| Type                       | Made by                                   | Shown by                                                  |
| -------------------------- | ----------------------------------------- | --------------------------------------------------------- |
| `Coverage`                 | the engine, per scan (what was read)      | the overview and the storage map                          |
| `NotScanned`               | the engine's storage map before a scan    | the storage map's "Not scanned yet"                       |
| `RemainderReason`          | the storage map (a part that is no file)  | the storage map's remainder lines                         |
| `SkipReason`               | the cleanup plan builder                  | the review screen, per item                               |
| `Failure`                  | the cleaner, per item                     | the result summary, through `ProblemText`                 |
| `BlockedReason`            | the cleanup plan builder                  | the review screen, per item                               |
| `ModuleOutcome`            | the engine, per module (`ModuleFinished`) | the module section, through `ProblemText`                 |
| `LeftOutCount`             | the engine, summed from `out.left_out`    | `LeftOutLines` under the category, from `ModuleFinished`  |

### Left-outs

A module that recognizes an item and does not offer it reports `out.left_out(LeftOutItem { category, reason, sizes })`
(the name `LeftOut` is taken by a cleanup plan's view). `LeftOutItem` holds no path and no args. The engine sums them
per (category, reason) into `LeftOutCount` (saturating, the weakest completeness) in `crates/engine/src/left_outs.rs`
and sends them with `ModuleFinished.left_out`, sorted by category, then by reason key, for every outcome (completed,
failed, cancelled, panicked). A left-out whose category is not the module's own is dropped and logged once as a
warning, and the contract refuses it (`ForeignLeftOutCategory`); an undeclared reason is counted and logged once.
A left-out of a category in the scan's ignore list, or of an ignored module or a module of an ignored group, is not
counted, and a category's details drop a left-out-only category once it, its module or its group is ignored. Left-outs are event-only: they are not in the scan cache, and the restore at start runs before the
UI subscribes, so after a restart no left-out line shows until the next scan. The rules engine reports none yet.

### Insights

A module may also describe space it does not offer (`out.describe(Insight)`, plan 032: Docker). The engine collects
them per module and sends them with `ModuleFinished.insights`, whatever the outcome, dropping a repeated identity of one
module with a warning. An insight is text and numbers: never a finding, never in a total, the storage map, the scan
cache or the journal, and nothing in it can be selected; the UI keeps the last scan's insights per module from the
event, and a restore at launch describes again (the module runs over the restored matches), but its events come before
the UI subscribes, so a UI that connects after a restore shows none until the next scan (known gap). The shown commands
are text; no code path of the app runs one.

### A failed start

`app/tauri/src/start_failure.rs`: when setup fails, the app logs the error, destroys the half-started main window (a
hidden web view would still run script and call commands without their state), manages a `StartFailed` marker and
shows the native dialog with the plain sentence `core.start.failed` (a development build adds the error under it; a
release build never does). The dialog does not block the main thread: its callback ends the app with exit code 1.
`shell::on_run_event` holds an exit with no code while the marker is there, so closing the last window or Cmd-Q waits
for the dialog.

### Permissions in the shell

When the main window gains focus, `shell::on_focus` probes the permissions again on a blocking thread
(`tauri::async_runtime::spawn_blocking`), never on the event loop's; the engine publishes `access-changed` when
something changed. The commands are `refresh_access` (the same check, answering the status), `open_access_settings`
(takes a `Capability`, never an address or a path) and `relaunch` ("Reopen Fresher": refused while a cleanup runs,
otherwise the engine is shut down and the app restarts). `reveal_finding` ("Show in Finder" on an item the app may not
remove) takes a finding's id, never a path: the engine shows that finding's own path through `SystemUi::reveal`, and
refuses an id of another scan (`unknown-finding`) and a tool's object (`nothing-to-reveal`). `app/tauri/Info.plist`,
merged by the bundler, holds the five reason strings of plan 006's design and no other usage key
(`app/tauri/tests/info_plist.rs`).

## UI

The interface, as plan [003](../plans/003-base-ui/readme.md) built it; its design argues every choice
([`00-design.md`](../plans/003-base-ui/00-design.md)): the visual language "Graphite and Iris", the motion system, the
components and the screens. React, TypeScript and plain CSS Modules over one cascade of layers; no component, animation,
router, state or CSS library. The baseline is Safari 16.0 (macOS 13, the minimum).

### Folders and boundaries

```
app/ui/src/
├── design/    generic, knows nothing of the product; every text comes in by props
│   ├── tokens/  base/  motion/  icons/        tokens, the cascade, the motion hooks, the icon allowlist
│   └── primitives/  data/  feedback/  layout/  effects/      one folder per component
├── product/   the app's components: take the core's types, render texts through i18n
├── screens/   the frame and the screens (scan, category, review, cleanups, settings): wire state to product
├── state/  ipc/  i18n/  locales/  test/
└── gallery/   the dev-only gallery: every component in every state (`gallery.html`, never in `dist/`)
```

The import boundaries, enforced by ESLint (`@typescript-eslint/no-restricted-imports`, one config per folder in
`eslint.config.ts`):

- `design/` imports `design/` and React; only `design/icons/` imports `lucide-react`. Never `product/`, `screens/`,
  `state/`, `ipc/` or `i18n/`.
- The design layer's `primitives/menu/` (a button's menu of actions, with the keyboard and focus handled) stands beside
  `primitives/tooltip/`; the card's settings menu is built on it, and so is a finding row's (`Menu.tabIndex` keeps
  a trigger out of the tab order when its row is not the list's focused row). The virtual list has one tab stop on its
  rows and the focused row's controls follow it; the arrow, Home, End, Page and Space keys act only from the row
  itself, a key on a control inside a row is the control's.
- `product/` imports `design/`, `i18n/` and types from `ipc/bindings.generated` (`import type`). Never `ipc/index`,
  `state/` or `screens/`: it takes its data by props.
- `screens/` imports everything above, `state/` and `ipc/`.
- `gallery/` imports `design/`, `product/` and `test/`. Never `state/` or `ipc/index`.
- Only `ipc/` imports `@tauri-apps/api`.
- `screens/` and `product/` never read an error's `message` or `code` (`no-restricted-syntax` in `errorDetailConfig`
  of `eslint.config.ts`): they show the core's text. The lint matches names (`e`, `err`, `error`, `reason`, `failure`,
  `cause`, and `.error` of a refusal), so it is a tripwire, not a guarantee: an aliased or destructured read slips
  through, and review catches it.

### State

- **The store keeps totals, not findings**: the engine status, progress, module outcomes, the overview (totals per
  category and `Overview.found`), the settings, the access status, the dictionary, the last failure. A batch of
  findings only marks the overview stale; `state/refresh.ts` asks `get_overview` at most 4 times a second during a
  scan and once more at its end; progress reaches React at most 10 times a second.
- **The storage map** (`state/connect.ts`, `state/reduce.ts`): the store keeps the core's `StorageMap`, asked with the
  overview and again when a scan starts or ends; the UI draws only each volume's five-part summary
  (`VolumeMap.summary`). The detailed parts stay in the core for its by-meaning and place queries; no screen asks
  for them now.
- **Scan** (`screens/scan/`, plan 042): a toolbar ("Scan", when the last scan finished as "2 minutes ago",
  formatted by `i18n/relative-time.ts` from the core's time and an injected wall clock, `state/wall-clock.ts`; Scan
  again, or Stop while scanning), then **the disk card** (`product/disk-card/`): the ring of the home volume made of
  the summary's four used parts with Free as the track, "<size> can be freed" in its centre (Scan before the first
  scan), the disk's name and "<free> free of <capacity>", and a two-column legend with an "i" per part holding the
  core's description; hovering a legend row or its arc highlights the other; a part too small to see gets a minimum
  arc. Other volumes are compact disk cards under it. **While scanning** the disk card gives way to the scanning card
  (`product/scanning-card/`): the swoosh (`design/data/swoosh-progress/`) fills to `ScanProgress.expected`, the core's
  estimate of what the scan walks (`screens/scan/progress.ts`: bytes against it, clamped and held at 99 % until the
  scan ends, the share of modules finished without an estimate), with the percent, "Looking at <module>" (the modules
  still running take turns every 6 s on the injected clock, `screens/scan/use-looking-at.ts`, a new name fading in
  with full motion only) and the size found so far; nothing below it looks finished: the tiles wait for the scan's
  end. Once it has ended (completed, partial or stopped), **the separator** "What you can remove"
  (`design/layout/section-separator/`) and **the groups** (`screens/scan/scan-groups.tsx`), one per level, its
  heading the core's `core.danger.<level>.zone` and `.meaning` with the group's icon; Safe and Usually safe have a
  "Select all" (its description in a tooltip), Look before removing has none. **Tiles**
  (`product/category-tile/` on `design/layout/tile/`, two columns in `product/category-grid/`): the category's icon
  badge (`product/category-icon/`, the core's `CategoryIcon` mapped to one `lucide-react` shape), its name and its
  `.line`, a checkbox, the size and count, Details and the `⋯` menu; the border tinted by the group. "Found, but left
  out" and "Shown, never removed" (insights) follow the groups. The selection bar (`product/selection-bar/`) floats
  at the bottom on Scan and on a category's details; it counts what goes and, with nothing going, the categories to
  check in Review.
- **Lists are paged**: `state/findings.ts` keeps at most 10 pages of 100 findings per app, least recently used
  dropped, but never a page with rows on screen of a mounted list: with more lists on screen than that (Review with
  every category selected) it keeps their pages and no more. `useFindings(view)` gives a virtualised list its total
  and the rows it shows, and holds the view while the list is mounted; once it unmounts its pages are dropped like any
  other. A category's details ask the core
  for `FindingsQuery.order: largest`. The pages start again on an ignore change (`CoreState.ignoreChanges`, bumped by
  a `core.ignore.*` setting change) as on a cleanup's removal, so an ignored row leaves at once and a restore brings
  it back.
- **Selection** (`state/selection.ts`): findings, and whole categories minus the findings ticked off in them, and
  `toCheck`: the "Look before removing" categories (a `BY_HAND` level, Medium or High) the user brought to Review,
  none of whose findings goes until the user ticks it there or on its details, or ticks the Review tile's
  all-checkbox (`tick-all`); a `BY_HAND` category is never selected whole for the user, nor is one whose level the
  overview does not tell. The action `choose { categories, within }` drops the module's own selection and selects
  the categories of a module choice that hold a finding ("Remove everything", "Keep settings"), a `BY_HAND` one
  only to check, touching nothing outside the module; emptied when a scan starts; nothing is pre-selected; a group's
  Select all (Safe and Usually safe only) ticks that group only, and Review with its confirmation stays the gate. The
  core receives only what goes. Its total is a preview summed with the core's rule; the confirmation shows only the
  core's cleanup plan (plan 008).
- **Cleanup** (`state/cleanup.ts`): `idle`, `preparing`, `confirming { plan }`, `running`, `finished { report }`,
  `refused { refusal }`, from the review screen's button to the result; the commands are sent from
  `state/cleanup-actions.ts`, which is also where the agreement is met: before a prepare it asks the core
  (`get_agreement`) and opens the agreement sheet (`product/agreement-sheet/`) when the current version is not
  accepted; one acceptance per click, then the prepare goes on. The sheet is not the guard, the engine is. The
  confirmation sends back exactly the plan's own summary (`confirmationOf`) and, for a High plan, the tick
  `highRiskUnderstood` of the dialog (cleared at every opening); a finished cleanup empties the selection, and `findings-removed` drops the finding pages and marks the overview stale.
- **The cleanup's screens** (plan [008/07](../plans/008-safe-removal/07-interface/readme.md), rebuilt by plan 042):
  Review (`screens/review/`) opens on **the summary card** (`review-summary.tsx`: the size that goes to the Trash and
  its item count, "You can put them back from Cleanups", a bar split by group and a chip per group; the Look chip says
  how many items are not checked yet), the separator "What goes", then **one tile per category**
  (`review-tile.tsx`, grouped by `review-groups.ts`, by level then size): the all-checkbox (the only sender of
  `tick-all`), its icon, name and line (Look: "tick each one you have checked"), a pill "<n> of <m> checked" (green
  when all, amber when not; "<n> picked" for single picks), its size, and every item with its own checkbox, name and
  short path (`product/paths/short-path.ts`: `shortPath(path, AppInfo.home)` shows `~` for the home at a component
  boundary; the full path stays in the tooltip and the accessible name). A Look category brought from Scan comes
  with nothing ticked (`Selection.toCheck`). **The bar** (`review-bar.tsx`, "Ready to remove"): count, size, Clear,
  Move to the Trash, inactive while nothing is ticked. Its panel (`screens/review/cleanup-panel.tsx`) asks the core
  for a plan and shows it in the confirmation (`product/confirm-dialog/`: Cancel first, the permanent choice off and
  red when ticked, the High tick for a High plan, what will not be removed and why); then the progress with Stop
  (`product/cleanup-progress/`) and the result (`product/result-summary/`, the space measured beside the estimate).
  The Cleanups screen (`screens/cleanups/`) lists the history from the journal; one cleanup shows each item with its
  outcome and puts back one item or all still in the Trash. Every text of an outcome or a reason comes from the core.
- **The Trash line** (plan 037; `product/trash-line/`, fed by `screens/scan/use-trash.ts`): under the scan screen's
  disk card, one line per `TrashLine` of `get_trash`, "Waiting in the Trash: 23 GB, 20 GB of it from cleanups here", the
  date the app's share leaves by itself when the core gives one, and **Open Trash**, the only action (never one that
  empties; a test fails on any button named so). The view is asked again on mount, after a scan, after a finished
  cleanup and when the window gains focus; a failed call hides the line and shows nothing. The result of a cleanup
  keeps "Show in the Trash" and adds an invitation to look and empty it by hand.
- **The quit question** (`screens/quit-question.tsx`): while a cleanup runs, closing the window or the app menu's
  Quit (⌘Q, an item of the app's own in `app/tauri/src/shell.rs`) publishes `quit-requested`, and the UI asks; **Stop
  and quit** stops after the item in flight. The Dock's Quit cannot be held
  ([`../known-issues/app-quit-during-cleanup.md`](../known-issues/app-quit-during-cleanup.md)).
- **Described, not cleaned** (plan 032; `screens/scan/described-only.tsx`, `product/insight-card/`,
  `screens/insight/`): under the groups, a section of one card per insight of the last scan (the core's title,
  summary and figures; no checkbox, nothing selectable, not counted in the groups, the selection bar or the ring). A
  card has Details, which opens the insight screen (`Screen.insight`, a module and an identity): the figures, one block
  per part with its detail, effect, danger level as wording and its command with **Copy** (the web view's clipboard;
  the text stays selectable as the fallback, and a failed copy says so), and the notes. Every sentence is the core's;
  the UI owns only the section's heading and sentence, Details, Back, Copy, Copied and Could-not-copy. The screen
  returns to Scan when the insight is no longer in the last scan.
- **A category's details** (`screens/category/`, plan 042): Back to Scan; the name, the group (a dot and its name),
  size and count; three answers side by side (What it is, with who makes it under it; If you remove it; Comes back?);
  each left-out line as an "i" bar (`product/left-out-lines/`); the module's choices ("Remove everything", "Keep
  settings"); the items (`FindingRow` variant `details`: the name first, the path small under it, no level badge),
  with "Select all" for Safe and Usually safe only. It replaced the module pages.
- **One menu on every row and tile** (`design/primitives/menu/`, `screens/use-row-menu.ts`): Show in Finder and
  Ignore this item (rows with a path), a separator, Ignore all <category>; the pointed item
  fills with the accent colour; an item carries only its label, no hint, and no menu turns a module off (Settings
  does). A tool's object has only the category item. The same menu is on the details' rows,
  Review's rows and Scan's tiles.
- **Settings** (`screens/settings/`, plan 042): a second sidebar (`settings-sidebar.tsx`): the search, then General,
  Scan, Ignored, Permissions, About, then "What it looks for", the module groups as a tree that opens and closes
  (`product/module-groups.ts`, texts `core.group.<group>.name`; a group holding the current page stays open), each
  module a page (`settings-page.tsx`: its group, its name, its description, its settings, its on/off switch last).
  Fresher's own data sits on General. **The search** (`settings-search.ts`, `settings-search-view.tsx`) matches
  labels, descriptions (with the current value) and section names; results list "Group › Module", the setting's name
  and its description with the match marked; the tree dims sections without a match and counts matches; Enter in the
  field or a click opens the page with the setting focused (`Screen.settings.focus`, `focus-setting.ts`); Esc clears.
  A whole-number setting is a slider (`design/primitives/slider/`) from its minimum to its maximum, sent on release,
  with a number field beside it; the core's refusals stay. The app's sidebar lists no module; a module is ignored
  from a menu and restored in Settings → Ignored.
- **Settings → Ignored** (`screens/settings/ignored-group.tsx`) has three tabs, "Sections", "Not looked for" and
  "Items" (plan 005's categories and paths), each with its count; the tabs are the design system's `Tabs`
  (`design/layout/tabs/`: an ARIA tab list, arrow keys, one tab stop). Scan's "N ignored" link opens the first tab
  that has an entry (`Screen.settings.tab`).
- **The High switch in the UI**: `AppInfo.high` says whether the core offers High; while it is `withheld` the sidebar
  leaves out Largest (`screens/frame-sidebar.tsx`; also hidden until the app info is known). Everything else that
  High would show (the groups, Review's tiles, an app page's choices) comes from the core, which leaves it out.
- **The design pieces** of plan 042: the mockups' colours as tokens (`design/tokens/`), the primary button's sheen and
  the menu's accent hover, the section separator, the tile, the category icon badge, the disk card, the swoosh, the
  slider, and the logo beside "Fresher" in the sidebar (`design/brand/logo-mark.tsx`).
- **Navigation** (`state/navigation.ts`): the screen is a discriminated union (`scan`, `category` with its category,
  `largest`, `review`, `cleanups`, `cleanup` with its cleanup, `insight` with its module and identity, `settings` with
  its page, an optional tab and an optional setting to focus); no router. `screenToShow` returns to Scan when a shown
  category is no longer among the categories the core lists. The sidebar has Scan, Largest (only while High is
  offered), Apps, Review, Cleanups and Settings and scrolls in a short window. When the focused element leaves the
  screen, the focus goes to the screen's heading (`screens/keep-focus.ts`).

### The demo player

`app/ui/src/demo/` (plan [038](../plans/038-demo-recordings/readme.md)) plays a scenario on the real interface for the
recordings: a finder of targets by role and name, a drawn pointer, the steps, the player and its start. It is a lazy
chunk imported in `main.tsx` only under `import.meta.env.DEV && inApp()`, so a release bundle has none of it (a check:
`data-demo-player` is absent from `app/ui/dist`). It asks the shell through the one IPC module and does nothing outside
demo mode.

### The window

- An overlay title bar with the traffic lights inset into the sidebar (`titleBarStyle: "Overlay"`, `hiddenTitle`,
  `trafficLightPosition`); the top strip is the drag region.
- The sidebar is the native sidebar material through a transparent window (`transparent: true`, `windowEffects`
  `sidebar`, following the window's active state). `main.tsx` sets `data-window="vibrant"` only inside the app; in a
  browser and in every test the window is drawn solid (`design/base/window.css`).
- `app.macOSPrivateApi: true` with the `macos-private-api` feature of `tauri`: the transparent window needs it, and it
  only bars the Mac App Store, which the app does not use (the user's decision of 2026-11-15; plan
  [013](../plans/013-release/readme.md) distributes outside it). Since Tauri 2.12.1 both are documented as no-ops, but
  `tauri-build` still checks that they agree.
- The capability (`app/tauri/capabilities/default.json`) allows the app's own commands and exactly two window
  permissions, for the drag region: `core:window:allow-start-dragging` and `core:window:allow-internal-toggle-maximize`
  (a double click zooms, as in every Mac window). A test in `app/tauri/tests/` fails on any other `core:` or plugin
  permission.
