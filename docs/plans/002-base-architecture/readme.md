# 002 — Base architecture

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** core, engine, store, modules, app

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

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

## Non-goals

- Real modules (Docker, `node_modules`, JetBrains, duplicates, …): each gets its own plan and its
  [category](../../categories/README.md) files.
- The fast scan (plan [004](../004-fast-scan/readme.md)) and the visual design (plan
  [003](../003-base-ui/readme.md)).
- Deletion: the cleaner gets its own plan. This plan only fixes the types a module uses to describe a removal.

## Design

### Crates

```
crates/
├── core/       fresher-core     domain types, ports (traits), events, text keys; no IO
├── macos/      fresher-macos    the only crate with `unsafe`: safe wrappers over macOS calls
├── scanner/    fresher-scanner  the walk and the in-memory tree (plan 004)
├── rules/      fresher-rules    the shared matcher for declarative rule data; holds no rules itself
├── store/      fresher-store    the only crate that touches the app's data folder
├── cleaner/    fresher-cleaner  the only code that deletes
├── engine/     fresher-engine   module registry, event bus, scan and clean sessions
├── modules/
│   ├── demo/           made-up findings, reads nothing from the disk
│   └── <one crate per feature: docker, node-modules, duplicates, jetbrains, …>
├── testing/, e2e/
app/tauri/      fresher-app      the composition root: real adapters, module registration, event forwarding
```

Who may depend on whom:

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
| `app`       | everything (it wires the adapters together)  |

A repository guard reads `cargo metadata` and fails on any edge outside this table, so modularity is checked, not
promised.

### A module

A module implements one trait from the core. In short, it:

- **describes itself**: id, the categories it produces, its settings with defaults, its dictionary;
- **declares interests**: what it wants to see during the one shared walk of the disk (a folder name, a path prefix, a
  marker file). A module never walks the disk itself — the disk is read once for all modules;
- **turns matches into findings** and publishes them; a module that reads no files (Docker) asks its data through a
  port (`CommandRunner`) instead;
- **describes a removal as data** (`Trash(path)` or `ToolCommand(...)`), never performs it: only the cleaner deletes.

A finding carries: a stable id, its category, the subject (a path or an external object), sizes with their completeness,
the danger level (the category's, raised but never lowered), text keys with arguments (name, why it matched, what
happens if removed, how it comes back), and the removal.

Shared knowledge two modules need (installed apps, detected projects, the clock) is a port in the core with one
implementation — not a call from one module into another.

### The module kit

Everything a new module needs, so that a module is written to a pattern rather than invented:

- **Rules as data.** A module keeps its rules in TOML next to its code; `fresher-rules` reads and matches them. A rule
  states: the category it produces, where to look, how the item is recognized (a marker file, a sibling, a name or
  version pattern, a minimum age), **what must not match**, the danger level, how the item comes back, and the removal.
  `CACHEDIR.TAG` is honoured.
- **The category contract.** A category is complete only with its name, what it is, which app makes it, what happens if
  it is removed, how it comes back and its danger level (rule 3 of [`rules.md`](../../rules.md)). A test fails for a
  category with a missing text, and for a category id without its file in
  [`docs/categories/`](../../categories/README.md) — code and catalogue cannot drift apart.
- **The test harness.** `fresher-testing` runs one module against a fixture home and returns its findings; the module's
  tests are fixture → expected findings, negative cases first (look-alike folders, the current version, a link out of
  the root), and one golden report per module.
- **A template and a checklist**: the crate skeleton (`rules.toml`, `locales/en.json`, settings, tests) and the list a
  module must tick before it ships, kept in [`docs/architecture/`](../../architecture/README.md).

### Events and commands

- **Commands** go down (UI → shell → engine): `start_scan`, `cancel_scan`, `get_overview`, `get_findings`,
  `get_settings`, `update_setting`, `get_dictionary`, `get_access_status`. A command returns at once; long work reports
  through events.
- **Events** go up: one typed enum in the core (`ScanStarted`, `ScanProgress`, `FindingsChanged`, `ModuleFinished`,
  `ScanFinished`, `SettingChanged`, `AccessChanged`, …). The engine's bus fans them out to subscribers: the shell (which
  forwards them to the UI over one Tauri channel), the store, other modules.
- Findings are sent in batches, at most a few dozen times a second, so a scan never floods the UI.
- The UI's TypeScript types for commands and events are **generated from the Rust types** (`*.generated.ts`) and a test
  fails when they are stale.
- The core has no async runtime: threads and channels. Tauri's runtime stays in the shell.

### Sessions

- The engine is an explicit state machine: idle → scanning → idle, idle → cleaning → idle. **One operation at a time**:
  a scan cannot start during a cleanup, nor a cleanup during a scan; a command that does not fit the state is refused
  with a reason.
- Every operation can be cancelled and ends in a defined state; a cancelled scan keeps its result marked partial.
- **Closing the window quits the app** — there is no background work ([`goal.md`](../../goal.md), section 8). During a
  cleanup the app asks first, finishes the item in hand, stops and writes down what was done.
- One instance: starting the app again brings the open window forward.

### IPC safety

- **The UI names things by id, never by path.** A command that leads to a removal takes ids of findings the core itself
  produced in this session; the core looks the paths up. Even a compromised webview cannot ask for an arbitrary path to
  be removed.
- Every command validates its input in the core; the shell passes it through and decides nothing.
- The Tauri capabilities list exactly the commands the UI uses: no file system, shell, network or dialog plugin exposed
  to the webview beyond what a step asks for by name. The content security policy stays without a remote origin.

### Settings and the app's own data

**Recommended: plain files, no SQLite.**

| Option                    | For                                                              | Against                                                              |
| ------------------------- | ---------------------------------------------------------------- | -------------------------------------------------------------------- |
| SQLite (bundled)          | transactions, queries, one file; nothing to install either       | a snapshot of a scanned tree is hundreds of MB; C code in the build  |
| **Files (recommended)**   | tiny, readable, trivially exported; the scan cache stays compact | our own formats to write and test                                    |

- `settings.json` in `~/Library/Application Support/dev.macfresher.app/`: versioned, written atomically (a temporary
  file, then a rename). Readable and copyable by the user — which settles "export and import" of
  [user settings](../../ideas/user-settings.md) for free.
- Typed: the core's settings and each module's settings are declared with a type, a default, and label and description
  keys. An unknown or broken value falls back to its default and is reported, never crashes the app.
- No setting can switch off the confirmation before deletion: such a key does not exist.
- The scan cache is a compact binary file (plan [004](../004-fast-scan/readme.md)). History and the cleanup journal
  choose their storage in their own plans.
- Everything goes through traits of `fresher-store` (`SettingsStore`, …), so moving to SQLite later changes one crate.
- **Nothing to install**: either way the app is one self-contained bundle.
- **File versions.** Every file the app writes carries a format version, and one policy covers them all: what holds the
  user's decisions or history (settings, the cleanup journal, history points) is **migrated** forward, step by step,
  with the old file kept until the new one is written; what can be rebuilt (the scan cache) is **discarded** on a
  version change. A file from a newer version of the app is never overwritten: the app says so and works without it.

The app's own footprint is part of the architecture: one data folder, a size budget, no log files by default, and the
webview runs with a non-persistent data store so it leaves no cache of its own. Settings show "Mac Fresher's own data:
N MB" with a button to clear it.

### The dictionary

- Each crate that has user-facing text owns `locales/en.json`, embedded in the binary; keys are namespaced
  `<module>.<group>.<name>`. A module is self-contained: its texts ship with it.
- The core never produces English prose: it emits a `Text { key, args }`. The UI asks the merged dictionary once
  (`get_dictionary(locale)`) and resolves keys with a small `t()` of our own (placeholders, plural forms through
  `Intl.PluralRules`); no i18n library.
- The UI's own chrome (buttons, navigation) lives in `app/ui/src/locales/en.json`.
- Numbers, sizes and dates are formatted in the UI with `Intl`.
- Tests: every key the code can emit exists; every category has all the texts rule 3 of
  [`rules.md`](../../rules.md) requires; no key is unused.
- Only `en` ships now; a second language is one more file per crate.

## Plans (after approval)

| #   | Plan                  | What                                                                              |
| --- | --------------------- | --------------------------------------------------------------------------------- |
| 01  | Core contracts        | domain types, ports, events, text keys, the module trait                          |
| 02  | Engine                | registry, event bus, the session state machine, scan over a port; the `demo` module |
| 03  | Settings and app data | `fresher-store`: data folder, `settings.json`, typed schema, file versions, footprint budget |
| 04  | Dictionary            | `locales/`, the key checks, the dictionary command, `t()` in the UI               |
| 05  | IPC and shell         | commands by id with validation, capabilities, the event channel, generated TypeScript types, the UI's store |
| 06  | Module kit            | the rule format and matcher, the category contract, the test harness, the template |
| 07  | Boundaries and docs   | the dependency guard; `architecture/`, `goal.md` §5, `CLAUDE.md` map updated      |

## Ideas this touches

[User settings](../../ideas/user-settings.md) and the [ignore list](../../ideas/ignore-list.md) (the settings file and a
per-module "enabled" flag), the demo mode of [best practices](../../ideas/best-practices.md) (the `demo` module),
[rules as data](../../ideas/architecture/rules-as-data.md) (`fresher-rules`), the
[storage explainer](../../ideas/storage-explainer.md) (one claim per path, no double counting).

## Decisions

None yet.

## Agent's calls (to check)

1. **Files instead of SQLite** for settings and the scan cache. It changes `goal.md` §5 and
   [`architecture/README.md`](../../architecture/README.md), which name SQLite today; they are updated when this plan is
   built.
2. **One crate per module** under `crates/modules/`, with a guard on the dependency table.
3. **`fresher-rules` becomes the shared matcher**, and each module owns its rule data.
4. **A new `fresher-engine`** holds orchestration, so the Tauri shell stays thin; **a new `fresher-macos`** is the only
   crate allowed `unsafe`.
5. **A `demo` module** proves the contract and feeds plan 003 before any real scan exists.
6. **Module-owned dictionaries**, merged at start; our own `t()` instead of an i18n library.
7. **TypeScript types generated from Rust** (`ts-rs`; the exact crate and version are checked in the step).
8. **No async runtime in the core.**
9. **The webview keeps no persistent data.**
10. **Rules are TOML data owned by each module**, with a required "must not match" part; the catalogue in
    `docs/categories/` is checked against the code by a test.
11. **One operation at a time**, and closing the window quits the app.
12. **Commands take ids, never paths.**
13. **Migrate what holds decisions, discard what can be rebuilt**; never overwrite a file from a newer version.
14. **Target version 0.1.0.**

## Conclusions

Not written yet.
