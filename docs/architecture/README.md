# Architecture

The crates, the layers and the data, as plan [002](../plans/002-base-architecture/readme.md) built them and plan
[004](../plans/004-fast-scan/readme.md) added the disk walk to them; their designs argue every choice
([002](../plans/002-base-architecture/00-design.md), [004](../plans/004-fast-scan/00-design.md)). A crate a later plan
creates keeps its row here and is described in detail by that plan. How to write a module: [`modules.md`](./modules.md).

## Layers

```
app/ui (React)  ──typed IPC (generated types)──▶  app/tauri: commands, the composition root
      ▲                                                  │
      └───────────── events (one channel) ───────────────┤
                                                         ▼
                                           fresher-engine: registry, event bus,
                                           one operation at a time, settings in memory
                                                         │
                         ┌───────────────────────────────┼────────────────────────────┐
                         ▼                               ▼                            ▼
              ports of the core: DiskWalk     modules/* (one crate per feature)    SettingsStore, ScanCache
              (fresher-scanner) → DiskState   over ONE shared walk → findings      (fresher-store: files)
                         │                               │
                         ▼                               └──▶ fresher-cleaner (only from a confirmed plan, plan 008)
              OS ports: FolderReader, ChangeJournal, Volumes
              (fresher-macos: the only unsafe code)
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

| Crate                 | Folder           | Responsibility                                                                                             | May depend on |
| --------------------- | ---------------- | ---------------------------------------------------------------------------------------------------------- | ------------- |
| `fresher-core`        | `crates/core`    | domain types, the module contract, ports (traits), settings schema, events, commands, errors, texts; no IO | —             |
| `fresher-macos`       | `crates/macos`   | the only crate with `unsafe` (in `mod ffi`): the bulk read, volumes, the change journal, the IO policy     | core          |
| `fresher-scanner`     | `crates/scanner` | the walk, the tree, the matcher, the cache codec, the rescan and the queries behind `DiskWalk`/`DiskState` | core, macos   |
| `fresher-rules`       | `crates/rules`   | the shared matcher for declarative rule data (TOML); holds no rules, each module owns its own              | core          |
| `fresher-store`       | `crates/store`   | the only crate that touches the app's data folder: atomic versioned files, `settings.json`, the scan cache | core          |
| `fresher-cleaner`     | `crates/cleaner` | the only code that deletes: executes a confirmed plan, Trash first (plan 008)                              | core, macos   |
| `fresher-engine`      | `crates/engine`  | module registry, event bus, one operation at a time, the scan session, settings, the disk queries          | core          |
| `fresher-module-<id>` | `modules/<id>`   | one feature: descriptor, rules or code, texts; `demo` makes up findings and reads no disk                  | core, rules   |
| `fresher-testing`     | `crates/testing` | fixture trees, fakes of the ports, the module harness and the category contract (dev only)                 | core          |
| `fresher-e2e`         | `crates/e2e`     | end-to-end and golden tests of the core (tests only)                                                       | anything      |
| `fresher-app`         | `app/tauri`      | the composition root: real adapters, module registration, commands, the event channel                      | anything      |

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
  [009](../plans/009-system-knowledge/readme.md), `KnownFolders`), never a macOS path such as `~/Library/Caches`.
- Only `fresher-cleaner` may call a deleting file system function (enforced by Clippy's `disallowed-methods`).
- External programs (the Docker CLI, `tmutil`) are called through the `CommandRunner` port, so tests use fakes.
- The UI gets everything it shows (names, descriptions, sizes, danger levels, the text of a refusal) from the core, as
  dictionary keys with arguments; the UI's own dictionary holds only its chrome.
- The IPC types are generated from the core's Rust types (`ts-rs`) into `app/ui/src/ipc/bindings.generated.ts`; a
  test keeps the file up to date.

## Ports

| Port            | Implemented by                                                                        | Fake for tests                                   |
| --------------- | ------------------------------------------------------------------------------------- | ------------------------------------------------ |
| `Clock`         | the app (system time)                                                                 | `FakeClock`                                      |
| `Trash`         | plan 008                                                                              | `FakeTrash`                                      |
| `CommandRunner` | the app (`std::process`, plan 008)                                                    | `FakeCommandRunner`                              |
| `DiskWalk`      | `fresher-scanner`'s `Scanner`, with the macOS reader in the app                       | `NaiveWalk` on a fixture tree                    |
| `FolderReader`  | `fresher-macos`'s `MacFolderReader` (bulk read); `fresher-scanner`'s portable `StdReader` | `StdReader` on a fixture tree                |
| `ChangeJournal` | `fresher-macos`'s `MacChangeJournal` (FSEvents)                                       | `FakeChangeJournal` (a script)                   |
| `Volumes`       | `fresher-macos`'s `MacVolumes` (plan 009 extends it)                                  | `FakeVolumes` (fixed facts per root)             |
| `ScanCache`     | `fresher-store`'s `FileScanCache`                                                     | `MemoryScanCache`                                |
| `DiskState`     | `fresher-scanner`'s `TreeState`, returned by `DiskWalk::commit`                       | `FixedDiskState` (a tree from paths and sizes)   |
| `SettingsStore` | `fresher-store` (`FileSettingsStore`)                                                 | in memory, in the engine's tests                 |
| `AccessProbe`   | plan 006 (`fresher-macos`); `Unknown` until then                                      | a fixed answer                                   |

The stand-ins until a later plan are listed in [`../hacks/`](../hacks/README.md).

## Data

No database: the app keeps its own state in small plain files, with nothing to install.

- **The data folder** is `~/Library/Application Support/io.github.alexsergey.fresher/` — never inside a scanned root. The app
  passes it to `fresher-store` (`DataFolder::open`); the store never looks it up itself.
- **`settings.json`**: `{ "format": "settings", "version": 1, "values": { "<key>": <value>, … } }`. Values are checked
  against the settings schema in the core (pure); an unknown key or a broken value falls back to its default and is
  reported, never a crash.
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
  flags, child count, own bytes, modification time, hits and owner, all as variable-length integers — then the hard
  link and clone groups. Totals are not stored: the reader sums them again. A body that does not read whole is
  refused (one full scan). A body whose file would pass the cap (measured compressed, as the store writes it) has its
  smallest subtrees with no hit, claim or unreadable folder collapsed into their folder (totals and counts only)
  until it fits; one that does not fit even then is not saved.
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
  larger files and matched files are nodes of their own; hard links and clones are counted once. The committed tree
  is the `DiskState`.
- **Queries** (the engine's `disk.rs` over the last committed `DiskState`): `disk_roots`, `disk_node`,
  `disk_children` (by size or by name, paged), `largest_files`, `search_names` (case ignored, in memory, a lowercase
  copy of the names built by the first search) and `unreadable_places`, every limit capped at 500. A query reads the
  state's `Arc` outside the engine's lock, so it never blocks a scan; `NoScan` before any commit, `StaleNode` for an
  id of another state. The IPC commands come with the screens that use them (plans 003 and 012).
- **The webview keeps nothing**: the window runs `incognito`.
- The cleanup journal (plan 008) and history (plan 010) choose their files in their own plans, under the same
  policy.

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
├── screens/   the app frame and the screens (overview, module, review, settings): wire state to product
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

### State

- **The store keeps totals, not findings**: the engine status, progress, module outcomes, the overview (totals per
  category and `Overview.found`), the settings, the access status, the dictionary, the last failure. A batch of
  findings only marks the overview stale; `state/refresh.ts` asks `get_overview` at most 4 times a second during a
  scan and once more at its end; progress reaches React at most 10 times a second.
- **Lists are paged**: `state/findings.ts` keeps at most 10 pages of 100 findings per app, least recently used
  dropped; `useFindings(view)` gives a virtualised list its total and the rows it shows. The module page asks the core
  for `FindingsQuery.order: largest`.
- **Selection** (`state/selection.ts`): findings, and whole categories minus the findings ticked off in them; emptied
  when a scan starts; nothing is pre-selected, selecting by level stops at Safe. Its total is a preview summed with
  the core's rule; the confirmation shows only the core's cleanup plan (plan 008).
- **Navigation** (`state/navigation.ts`): the screen is a discriminated union (`overview`, `module`, `review`,
  `settings`); no router. When the focused element leaves the screen, the focus goes to the screen's heading
  (`screens/keep-focus.ts`).

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
