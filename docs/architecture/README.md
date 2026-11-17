# Architecture

The crates, the layers and the data, as plan [002](../plans/002-base-architecture/readme.md) built them, plan
[004](../plans/004-fast-scan/readme.md) added the disk walk to them and plan
[008](../plans/008-safe-removal/readme.md) the removal, and plan [009](../plans/009-system-knowledge/readme.md) the
system's facts; their designs argue every choice
([002](../plans/002-base-architecture/00-design.md), [004](../plans/004-fast-scan/00-design.md),
[008](../plans/008-safe-removal/00-design.md)). A crate a later plan creates keeps its row here and is described in
detail by that plan. How to write a module: [`modules.md`](./modules.md); how anything is removed, for a developer
about to touch it: [`removal.md`](./removal.md).

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
- **The engine depends on the core alone**: everything it needs from outside is a port (a trait) in `fresher-core`,
  implemented in another crate and injected by the app.
- **Events, not polling**: the engine streams typed events (`scan-started`, `findings-added`, …) to the UI through one
  channel; the commands take ids, never paths.
- **No async runtime in the core**: `std::thread` and `std::sync::mpsc`; Tauri's runtime stays in the shell.

## Crates

| Crate                 | Folder           | Responsibility                                                                                                                                                                          | May depend on |
| --------------------- | ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| `fresher-core`        | `crates/core`    | domain types, the module contract, ports (traits), settings schema, events, commands, errors, texts; no IO                                                                              | —             |
| `fresher-macos`       | `crates/macos`   | the only crate with `unsafe` (in `mod ffi`): the bulk read, volumes, the change journal, the IO policy, the Trash call, open files, the protection list, the calls beneath held folders | core          |
| `fresher-scanner`     | `crates/scanner` | the walk, the tree, the matcher, the cache codec, the rescan and the queries behind `DiskWalk`/`DiskState`                                                                              | core, macos   |
| `fresher-rules`       | `crates/rules`   | the shared matcher for declarative rule data (TOML); holds no rules, each module owns its own                                                                                           | core          |
| `fresher-system`      | `crates/system`  | the safe adapters of system knowledge (plan 009): `MacKnownFolders`, `MacInstalledApps` with its own `Info.plist` and code-signature readers, `MacRunningApps`, `SystemClock`, `StartedClock`, `ProcessCommandRunner` | core, macos   |
| `fresher-store`       | `crates/store`   | the only crate that touches the app's data folder: atomic versioned files, `settings.json`, the scan cache, the cleanup journal                                                         | core          |
| `fresher-cleaner`     | `crates/cleaner` | the only code that removes: rehearses and executes a confirmed plan through its guard, Trash first, puts back ([`removal.md`](./removal.md))                                            | core, macos   |
| `fresher-engine`      | `crates/engine`  | module registry, event bus, one operation at a time, the scan session, settings, the disk queries, the plan builder, the cleanup and its history                                        | core          |
| `fresher-module-<id>` | `modules/<id>`   | one feature: descriptor, rules or code, texts; `jetbrains` (older IDE versions' folders); `demo` and `trial` are dev-only (debug builds)                                                | core, rules   |
| `fresher-testing`     | `crates/testing` | fixture trees, fakes of the ports, the module harness and the category contract (dev only)                                                                                              | core          |
| `fresher-sandbox`     | `crates/sandbox` | the case files of a feature, their checks, the sandbox layout and its generator (a binary); dev only, a dependency of the app and the tests alone                                                                                     | core, testing, macos |
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
- A development build and every test get `Reach::Sandbox`: the cleaner acts only below a folder that holds the
  sandbox marker; only a release build's composition makes the token for the real disk
  ([`removal.md`](./removal.md), "Reach and the sandbox").
- **Sandbox mode** (plan [007](../plans/007-safe-development/readme.md), a debug build only, `app/tauri/src/sandbox.rs`):
  `FRESHER_SANDBOX=<folder>` makes the composition (`sandbox_engine`) live entirely in a folder the sandbox generator
  made: its `home/` is `~` and the known folders, `.fresher-data/` the data folder, `home/.Trash` the Trash, the
  cleaner is `Reach::SandboxFolder`, and the protection is the real Mac's list joined with the sandbox's
  (`Protection::joined`). The scanner's avoid list is the places that prompt of both homes, so no scan raises a
  system prompt. Volumes, open files, commands and access stay real, read-only; the running apps are never the Mac's
  but the sandbox's declared ones (`world.toml`, none without it: plan 016), and the clock starts at the moment the
  generator wrote into `world.toml` and runs from there (`StartedClock`; the system's without one), so the case files'
  ages hold however long ago the sandbox was generated (plan 016); the process's IO policy is set
  only by the app's start, never by `sandbox_engine`, so the autotests run the same composition on fakes. The window
  shows a banner naming the sandbox and saying "Open apps are simulated." (a development build outside one: "nothing
  can be removed outside a sandbox"; nothing in a release build). A release build has no such branch.
- External programs (the Docker CLI, `tmutil`) are called through the `CommandRunner` port, so tests use fakes.
- **The system's facts** (plan [009](../plans/009-system-knowledge/readme.md)): the engine's `Ports.system`
  (`SystemPorts`: known folders, installed apps, running apps) becomes one lazy `SystemFacts` per scan, handed to
  every module as `cx.system()`; the engine adds its own interests for the projects under `ModuleId::system()` and
  sets the projects before any `finish`. Nothing of it is written to disk.
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
| `Trash`          | `fresher-macos`'s `MacTrash` (the system's Trash call; put back by `renamex_np`)                                | `FakeTrash` (a Trash folder inside the fixture)             |
| `OpenFiles`      | `fresher-macos`'s `MacOpenFiles` (the user's own processes, `libproc`)                                          | `FakeOpenFiles`                                             |
| `CommandRunner`  | `fresher-system`'s `ProcessCommandRunner` (`std::process`, no shell; a 60 s timeout; the caller's cancel token kills it) | `FakeCommandRunner`                                         |
| `KnownFolders`   | `fresher-system`'s `MacKnownFolders` (the macOS layout of a home folder; opens nothing)                         | `FakeKnownFolders` (`home_layout` of a fixture root)        |
| `InstalledApps`  | `fresher-system`'s `MacInstalledApps` (the apps folders and one below, the older JetBrains Toolbox's folder four below; an app link read where it points; `Info.plist`, code signature; what it cannot read in `unread`; `find` by bundle id through Spotlight, `mdfind`, or the listed apps in a sandbox) | `FakeInstalledApps` (`partial`), `installed_app`            |
| `RunningApps`    | `fresher-system`'s `MacRunningApps` (`fresher-macos`'s process table, each executable's bundles)                | `FakeRunningApps` (a script), `running_app`                 |
| `Cleaner`        | `fresher-cleaner`'s `DiskCleaner`                                                                               | `FakeCleaner` (records the executed plan)                   |
| `CleanupJournal` | `fresher-store`'s `FileCleanupJournal`                                                                          | `MemoryCleanupJournal`                                      |
| `DiskWalk`       | `fresher-scanner`'s `Scanner`, with the macOS reader in the app                                                 | `NaiveWalk` on a fixture tree                               |
| `FolderReader`   | `fresher-macos`'s `MacFolderReader` (bulk read); `fresher-scanner`'s portable `StdReader`                       | `StdReader` on a fixture tree                               |
| `ChangeJournal`  | `fresher-macos`'s `MacChangeJournal` (FSEvents)                                                                 | `FakeChangeJournal` (a script)                              |
| `Volumes`        | `fresher-macos`'s `MacVolumes`: `identify`, `available` (what a cleanup measures), `space` (the map's volume)   | `FakeVolumes` (`with_space`, `answering_available`)         |
| `ScanCache`      | `fresher-store`'s `FileScanCache`                                                                               | `MemoryScanCache`                                           |
| `DiskState`      | `fresher-scanner`'s `TreeState`, returned by `DiskWalk::commit`                                                 | `FixedDiskState` (a tree from paths and sizes)              |
| `SettingsStore`  | `fresher-store` (`FileSettingsStore`)                                                                           | in memory, in the engine's tests                            |
| `AccessProbe`    | plan 006 (`fresher-macos`); `Unknown` and `NotAsked` until then                                                 | `FakeAccess` (any state, guarded places, tries)             |
| `SystemUi`       | plan 006 (`/usr/bin/open`); `Unsupported` until then                                                            | `FakeSystemUi` (records every call)                         |

The stand-ins until a later plan are listed in [`../hacks/`](../hacks/README.md).

## Data

No database: the app keeps its own state in small plain files, with nothing to install.

- **The data folder** is `~/Library/Application Support/io.github.alexsergey.fresher/` — never inside a scanned root. The app
  passes it to `fresher-store` (`DataFolder::open`); the store never looks it up itself.
  A development build (`npm run dev`) runs with `app/tauri/tauri.dev.conf.json`: the identifier
  `io.github.alexsergey.fresher.dev`, so its data folder, single-instance lock and permission grants are separate; a
  debug build with the release identifier refuses to start.
- **`settings.json`**: `{ "format": "settings", "version": 1, "values": { "<key>": <value>, … } }`. Values are checked
  against the settings schema in the core (pure); an unknown key or a broken value falls back to its default and is
  reported, never a crash. Each setting carries its **group** (where the settings screen shows it: `general`, `scan`,
  `ignored`, `permissions`, or `module` for every module's) and its **effect** (`now`, or `next-scan`: a change marks
  the last scan, and the UI offers "Rescan to apply"; the app never rescans on its own). The kinds are `switch`,
  `number`, `choice`, `folders` (`~`, `~/…` or a plain absolute path, never a protected folder), `ids` (of
  categories, modules or groups, checked by what they hold: `IdOf`) and `paths` (the ignore lists). `reset_settings` puts every setting but the ignore lists back to its default in one save.
- **The ignore list** (`core.ignore.modules`, `core.ignore.groups`, `core.ignore.categories`, `core.ignore.paths`):
  the engine keeps a scan's ignored findings in memory but leaves them out of every answer and plan, so a restore
  shows them again without a rescan; the cleaner gets the ignored paths as protection. An ignored group hides every
  registered module of it (resolved through the registry when the lists are taken), whatever the module's own entry
  says; `ignore` refuses a module or a group nobody registered. Ignoring a module is not `<module>.enabled`: the
  module still runs, and its findings are only hidden. The overview says which modules are hidden and why
  (`Overview.hiddenModules`, each `{ module, by }`, `by` the module's own entry or its group), so the UI never
  works it out. `ModuleInfo.group` carries each module's group; the engine's `hidden_modules` resolves the lists. `ignore` and `restore_ignored` change the lists through the same check and
  save as `update_setting`. The app's own commands beside the engine: `pick_folder` (the system's folder picker,
  `tauri-plugin-dialog`, called from Rust only) and `get_own_data` (the footprint below).
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
  flags, child count, own bytes, modification time, hits, owner and access (one byte), all as variable-length
  integers — then the hard link and clone groups. Totals are not stored: the reader sums them again. A body that does
  not read whole is refused (one full scan). A body whose file would pass the cap (measured compressed, as the store
  writes it) has its smallest subtrees with no hit, claim or unreadable folder collapsed into their folder (totals and
  counts only) until it fits; one that does not fit even then is not saved.
- **Rescan and restore** (the scanner's `rescan`): every committed scan is saved to the cache with each root's journal
  position, taken before the walk read anything. A scan the user starts (`WalkMode::Auto`) loads the cache, checks it
  is of the same roots, volumes, avoid list and interests, asks the change journal what changed since (`ChangeJournal`,
  FSEvents), reads again only those folders with their parents and the unreadable ones, copies the rest of the tree,
  then reads a sample of the copied folders: one difference, a gap in the journal, a position older than two weeks or
  too many changes, and the scan is full (`WalkSummary::fallback` says why). At launch the engine restores the last
  state (`WalkMode::Restore`): the cache only, no folder read, the modules with interests run over its matches.
- **The scan in memory** (the scanner's `walk`, `tree` and `matcher`): one parallel pass, a worker per core (at most
  16) taking folders from one queue, each folder read in one bulk call per batch of entries, every entry matched
  against every module's interests with one lookup, and one aggregator building the tree. Files under 1 MiB are
  folded into their folder (a small hard-linked or cloned file keeps only its identity, in the link and clone tables);
  larger files and matched files are nodes of their own; hard links and clones are counted once. Each node keeps its
  entry's access for the user (the owner of the home folder) in one byte — owned, writable by the mode bits, the
  three locks — read in the same bulk call; a match's entry facts (removability) come from its node and its
  parent's. The committed tree is the `DiskState`.
- **Queries** (the engine's `disk.rs` over the last committed `DiskState`): `disk_roots`, `disk_node`,
  `disk_children` (by size or by name, paged), `largest_files`, `search_names` (case ignored, in memory, a lowercase
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
  `cleanups.jsonl`, private to the user (`0600`). A header line `{"format":"cleanup-journal","version":1}`, then one
  record per line tagged by `type` — `begin` (every item of the cleanup: its path, identity, action, category, finding
  key, reason, danger level and sizes), `item` (its outcome and where it landed in the Trash), `end` and `put-back` —
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
- History points (plan 010) choose their files in their own plan, under the same policy.

## The storage map

Plan [012](../plans/012-storage-map/readme.md): where the space of a volume went, in parts that add up to its
capacity. The core holds the types (`StorageMap`, `VolumeMap`, `MapPart`, `PartKind`, `RemainderReason`, `MeaningGroup`,
`PlaceView`, `DiskSpace`); the engine builds them; the UI draws them and decides nothing.

- **The parts**, in this order, each with a core-given name, description and danger tone: Explained (the findings of
  the modules; the only part offered for cleaning), Your folders (scanned, claimed by no module), Not scanned yet
  (during a scan only), the remainder (used space the scan did not measure), the system's volumes, the virtual
  memory volume, the user's other volumes by their own names, the disk's own space (capacity minus free minus every
  mounted volume's used space), and Free. `VolumeMap::new` is the only way to build a map and refuses parts that do
  not add up to the capacity or come out of order.
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
  (`place.rs`): a folder's children largest first, each with its explained size.
- **The three phases** (`query.rs`): no scan, scanning and complete (the phase is in the map, `MapPhase`); a query
  takes what it needs under the results' lock and builds after letting it go, so none blocks a scan.
- **Budget**: the shares of a scan of the `LARGE` tree take under 50 ms and 5 MB
  (`crates/e2e/tests/storage_budget.rs`, run by hand; [`../testing.md`](../testing.md)).
- The commands are `get_storage_map`, `get_storage_meaning(volume)` and `get_storage_place(node, descent, offset,
  limit)` (`app/tauri/src/storage_commands.rs`). The storage map's sandbox case file (plan 012/05) is not built yet: it
  waits for plan [007](../plans/007-safe-development/readme.md) to be merged into `main`.

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
counted, and the module page drops a left-out-only category once it, its module or its group is ignored. Left-outs are event-only: they are not in the scan cache, and the restore at start runs before the
UI subscribes, so after a restart no left-out line shows until the next scan. The rules engine reports none yet.

### A failed start

`app/tauri/src/start_failure.rs`: when setup fails, the app logs the error, destroys the half-started main window (a
hidden web view would still run script and call commands without their state), manages a `StartFailed` marker and
shows the native dialog with the plain sentence `core.start.failed` (a development build adds the error under it; a
release build never does). The dialog does not block the main thread: its callback ends the app with exit code 1.
`shell::on_run_event` holds an exit with no code while the marker is there, so closing the last window or Cmd-Q waits
for the dialog.

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
├── screens/   the frame and the screens (overview, module, review, cleanups, settings): wire state to product
├── modules/   optional custom views per module (none yet); the generic module page is the default
├── state/  ipc/  i18n/  locales/  test/
└── gallery/   the dev-only gallery: every component in every state (`gallery.html`, never in `dist/`)
```

The import boundaries, enforced by ESLint (`@typescript-eslint/no-restricted-imports`, one config per folder in
`eslint.config.ts`):

- `design/` imports `design/` and React; only `design/icons/` imports `lucide-react`. Never `product/`, `screens/`,
  `state/`, `ipc/` or `i18n/`.
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
- **The storage map** (`state/storage.ts`, `state/navigation.ts`): the store keeps the core's `StorageMap`, asked with
  the overview and again when a scan starts or ends; the by-meaning groups of a volume and the pages of a place are
  asked of the core when shown and kept in a small cache (at most 10 pages of 100 rows), dropped when the map's phase
  or state changes. The overview shows one view of the home volume at a time, kept for the session: by meaning
  (`product/meaning-list/`, cards largest first) or by place (`product/place-list/`, a breadcrumb that goes back up
  through the folders opened, and a roots list when a volume holds several roots). The bar (`product/storage-map/`)
  draws every part with space in the core's order; the tone of Explained is the highest danger level among its
  categories, from the core's groups (`explainedTone`).
- **Lists are paged**: `state/findings.ts` keeps at most 10 pages of 100 findings per app, least recently used
  dropped; `useFindings(view)` gives a virtualised list its total and the rows it shows. The module page asks the core
  for `FindingsQuery.order: largest`.
- **Selection** (`state/selection.ts`): findings, and whole categories minus the findings ticked off in them; emptied
  when a scan starts; nothing is pre-selected, selecting by level stops at Safe. Its total is a preview summed with
  the core's rule; the confirmation shows only the core's cleanup plan (plan 008).
- **Cleanup** (`state/cleanup.ts`): `idle`, `preparing`, `confirming { plan }`, `running`, `finished { report }`,
  `refused { refusal }`, from the review screen's button to the result; the commands are sent from
  `state/cleanup-actions.ts`. The confirmation sends back exactly the plan's own summary (`confirmationOf`); a
  finished cleanup empties the selection, and `findings-removed` drops the finding pages and marks the overview stale.
- **The cleanup's screens** (plan [008/07](../plans/008-safe-removal/07-interface/readme.md)): the review screen's
  panel (`screens/review/cleanup-panel.tsx`) asks the core for a plan and shows it in the confirmation
  (`product/confirm-dialog/`: Cancel first, the permanent choice off and red when ticked, what will not be removed and
  why); then the progress with Stop (`product/cleanup-progress/`) and the result (`product/result-summary/`, the
  space measured beside the estimate). The Cleanups screen (`screens/cleanups/`) lists the history from the journal;
  one cleanup shows each item with its outcome and puts back one item or all still in the Trash. Every text of an
  outcome or a reason comes from the core.
- **The quit question** (`screens/quit-question.tsx`): while a cleanup runs, closing the window or the app menu's
  Quit (⌘Q, an item of the app's own in `app/tauri/src/shell.rs`) publishes `quit-requested`, and the UI asks; **Stop
  and quit** stops after the item in flight. The Dock's Quit cannot be held
  ([`../known-issues/app-quit-during-cleanup.md`](../known-issues/app-quit-during-cleanup.md)).
- **Modules by group**: the sidebar, the scan's chips and Settings list modules under their group
  (`product/module-groups.ts`, texts `core.group.<group>.name`); a hidden module stays in the sidebar, dimmed with
  "Ignored", and its page says why with Restore.
- **Settings → Ignored** (`screens/settings/ignored-group.tsx`) has three tabs, "Ignored groups", "Ignored
  modules" and "Ignored items" (plan 005's categories and paths), each with its count; the tabs are the design
  system's `Tabs` (`design/layout/tabs/`: an ARIA tab list, arrow keys, one tab stop). The overview's "N ignored"
  link opens the first tab that has an entry (`Screen.settings.tab`).
- **Navigation** (`state/navigation.ts`): the screen is a discriminated union (`overview`, `module`, `review`,
  `cleanups`, `cleanup`, `settings`); no router. When the focused element leaves the screen, the focus goes to the
  screen's heading (`screens/keep-focus.ts`).

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
