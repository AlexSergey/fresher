# Architecture

The crates, the layers and the data, as plan [002](../plans/002-base-architecture/readme.md) built them; its design
argues every choice ([`00-design.md`](../plans/002-base-architecture/00-design.md)). A crate a later plan creates keeps
its row here and is described in detail by that plan. How to write a module: [`modules.md`](./modules.md).

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
              ports of the core: DiskWalk     modules/* (one crate per feature)    SettingsStore
              (fresher-scanner, plan 004)     over ONE shared walk → findings      (fresher-store: files)
                                                         │
                                                         └──▶ fresher-cleaner (only from a confirmed plan, plan 008)
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
| `fresher-macos`       | `crates/macos`   | the only crate with `unsafe`: macOS calls behind safe functions (plan 004)                                 | —             |
| `fresher-scanner`     | `crates/scanner` | the fast parallel walk behind the `DiskWalk` port; hard links counted once (plan 004)                      | core, macos   |
| `fresher-rules`       | `crates/rules`   | the shared matcher for declarative rule data (TOML); holds no rules, each module owns its own              | core          |
| `fresher-store`       | `crates/store`   | the only crate that touches the app's data folder: atomic versioned files, `settings.json`                 | core          |
| `fresher-cleaner`     | `crates/cleaner` | the only code that deletes: executes a confirmed plan, Trash first (plan 008)                              | core, macos   |
| `fresher-engine`      | `crates/engine`  | module registry, event bus, one operation at a time, the scan session, settings in memory                  | core          |
| `fresher-module-<id>` | `modules/<id>`   | one feature: descriptor, rules or code, texts; `demo` makes up findings and reads no disk                  | core, rules   |
| `fresher-testing`     | `crates/testing` | fixture trees, fakes of the ports, the module harness and the category contract (dev only)                 | core          |
| `fresher-e2e`         | `crates/e2e`     | end-to-end and golden tests of the core (tests only)                                                       | anything      |
| `fresher-app`         | `app/tauri`      | the composition root: real adapters, module registration, commands, the event channel                      | anything      |

The "May depend on" column lists normal dependencies; `fresher-testing` is allowed as a dev-dependency of any crate,
and a module never depends on another module. The column is the constant `TABLE` (and `MODULE_ALLOWED`) of
[`scripts/check-crate-deps.ts`](../../scripts/check-crate-deps.ts), which reads `cargo metadata` and fails on any edge
outside it, and on a crate that is not in it (`npm run check:crate-deps`, in the pre-push hook and CI;
[`../quality.md`](../quality.md)).

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

| Port            | Implemented by                                                     | Fake for tests                   |
| --------------- | ------------------------------------------------------------------ | -------------------------------- |
| `Clock`         | the app (system time)                                              | `FakeClock`                      |
| `Trash`         | plan 008                                                           | `FakeTrash`                      |
| `CommandRunner` | the app (`std::process`, plan 008)                                 | `FakeCommandRunner`              |
| `DiskWalk`      | `fresher-scanner` (plan 004); a walk that finds nothing until then | `NaiveWalk` on a fixture tree    |
| `SettingsStore` | `fresher-store` (`FileSettingsStore`)                              | in memory, in the engine's tests |
| `AccessProbe`   | plan 006 (`fresher-macos`); `Unknown` until then                   | a fixed answer                   |

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
- **The webview keeps nothing**: the window runs `incognito`.
- The scan cache (plan 004), the cleanup journal (plan 008) and history (plan 010) choose their files in their own
  plans, under the same policy.
