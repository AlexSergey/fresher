# 002 — Base architecture

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** core, engine, store, modules, rules,
testing, app

## Goal

Strict modularity from the first line of product code: a shared core of types, ports, events and text keys; every
feature a module in its own crate that never knows another; an engine that runs one shared walk for all modules and
streams typed events; settings saved in plain versioned files with nothing to install; every label from a dictionary.
The whole path works end to end on a made-up `demo` module. The design: [`00-design.md`](./00-design.md).

Branch: `feat/002-base-architecture`, squash-merged into `main` once when done
([`../../workflow.md`](../../workflow.md)). Depends on plan [001](../001-repository-setup/readme.md) (its branch is the
base of this one).

## Plans

| #   | Plan                                                          | Status      | Depends on      | What                                                                                    |
| --- | ------------------------------------------------------------- | ----------- | --------------- | --------------------------------------------------------------------------------------- |
| 01  | [Core contracts](./01-core-contracts/readme.md)               | ✔️ done     | —               | domain types, the module trait, ports, settings schema, events, errors, dictionary type |
| 02  | [Engine](./02-engine/readme.md)                               | ✔️ done     | 01              | registry, event bus, state machine, scan session, settings; the `demo` module           |
| 03  | [Settings and app data](./03-settings-and-app-data/readme.md) | ✔️ done     | 01              | `fresher-store`: data folder, atomic files, `settings.json`, versions, footprint        |
| 04  | [Dictionary](./04-dictionary/readme.md)                       | ✔️ done     | 01              | the core's `locales/en.json` and key checks; `t()` and chrome texts in the UI           |
| 05  | [IPC and shell](./05-ipc-and-shell/readme.md)                 | ✔️ done     | 02, 03, 04      | generated types, commands, event channel, capabilities, the UI's store, e2e             |
| 06  | [Module kit](./06-module-kit/readme.md)                       | ✔️ done     | 01 (06/04: 02)  | rule format and matcher, test harness, category contract, template                      |
| 07  | [Boundaries and docs](./07-boundaries-and-docs/readme.md)     | ✔️ done     | 01 (07/02: all) | the dependency guard; goal, architecture and repository docs; version 0.1.0             |

### Order and parallel work

```
01 ──┬── 02 ──────────┬── 05 ──┐
     ├── 03 ──────────┤        │
     ├── 04 ──────────┘        ├── 07/02 ── conclusions
     ├── 06/01–03 ── 06/04 ────┤   (06/04 also after 02/05)
     └── 07/01 ────────────────┘
```

- **Wave 1**: 01, its steps in order.
- **Wave 2, in parallel**: 02, 03, 04, 06/01–06/03, 07/01. Each runs in its own worktree branched from
  `feat/002-base-architecture` and is merged back when its plan is done; two agents never share a worktree (the git
  index and `target/` are shared state).
- **Wave 3, in parallel**: 05 (after 02, 03, 04) and 06/04 (after 02/05 and 06/03).
- **Wave 4**: 07/02, then the conclusions.

Which files each plan touches, so parallel plans stay apart:

| Plan | Owns (only this plan edits it during its wave)                                                             | Shared files it touches                                                                               |
| ---- | ---------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| 01   | `crates/core/`                                                                                             | root `Cargo.toml` (`[workspace.dependencies]`), `Cargo.lock`, `.commitlintrc.cjs`, `docs/quality.md`  |
| 02   | `crates/engine/`, `modules/demo/`                                                                          | root `Cargo.toml` (members; 02/05 only), `Cargo.lock`                                                 |
| 03   | `crates/store/`                                                                                            | `Cargo.lock`                                                                                          |
| 04   | `crates/core/locales/`, `crates/core/src/texts.rs`, `app/ui/src/i18n/`, `app/ui/src/locales/`              | `crates/core/src/lib.rs` (one `mod`, one `pub use`)                                                   |
| 05   | `app/tauri/`, `app/ui/src/ipc/`, `app/ui/src/state/`, `app/ui/src/app.tsx`, `app/ui/e2e/`, `crates/e2e/`   | `Cargo.lock`, `crates/engine/` (05/02), `docs/hacks/README.md`, `fixtures/golden/`, ignore configs   |
| 06   | `crates/rules/`, `crates/testing/`, `docs/architecture/modules.md`                                         | `Cargo.lock`, `docs/categories/README.md`, `modules/demo/` (06/04)                                    |
| 07   | `scripts/check-crate-deps.ts`, its test                                                                    | `package.json` (07/01), `docs/quality.md` (07/01); all docs and the version files (07/02)             |

The core is frozen after plan 01: a contract missing in a later plan stops that plan, and the change goes into
`crates/core` in the main session. `Cargo.lock` is never merged by hand: on a conflict take either side, regenerate it
(`cargo metadata --format-version 1 > /dev/null`) and run the checks.

## Ideas this touches

[User settings](../../ideas/user-settings.md) and the [ignore list](../../ideas/ignore-list.md) (the settings file, the
per-module `enabled` switch, the stable `FindingKey`), the demo mode of [best practices](../../ideas/best-practices.md)
(the `demo` module), [rules as data](../../ideas/architecture/rules-as-data.md) (`fresher-rules`), the
[storage explainer](../../ideas/storage-explainer.md) (one claim per path, no double counting).

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 %.
- `npm run tauri dev` opens the app; "Scan" streams the demo findings into a list grouped by category, with every label
  from the dictionary; a second scan during a scan is refused with a plain message; closing the window quits.
- The core e2e (`crates/e2e/tests/demo.rs`) runs engine + demo module + settings file end to end against its golden
  report.
- A module crate that depends on another module, or the engine depending on the store, fails the dependency guard.
- `app/ui/src/ipc/bindings.generated.ts` is up to date (its test passes); no component calls `invoke`.
- Settings survive a restart; a file from a newer version is never overwritten.
- [`../../goal.md`](../../goal.md), [`../../architecture/`](../../architecture/README.md),
  [`../../rules.md`](../../rules.md) and [`CLAUDE.md`](../../../CLAUDE.md) describe files instead of SQLite and the
  module layout that was built.

## Conclusions

Written 2026-11-15. The user tried the app in a real window and approved the merge the same day ("the window is
fine"). Their one report, an uncaught `transformCallback` error, came from the dev page opened in a plain browser; the
UI now shows the unreachable core there instead.

### What was built

- **01 Core contracts**: `fresher-core` — every type, trait and port of the plan, no IO: ids, sizes, danger levels,
  subjects, removals, findings; the `Module` trait and its descriptor; the ports (`DiskWalk`, `SettingsStore`,
  `AccessProbe`, `Clock`, `Trash`, `CommandRunner`); the settings schema; events, commands, `CommandError` and
  `Refusal`; text keys and the dictionary type.
- **02 Engine**: `fresher-engine` — registry, event bus, one operation at a time with cancel and shutdown, the scan
  session over one shared walk with batched findings, settings in memory; and `fresher-module-demo`.
- **03 Settings and app data**: `fresher-store` — the data folder, atomic versioned files, `settings.json` with
  migration and the newer-version guard, the footprint (50 MB budget). No SQLite, and it deletes nothing.
- **04 Dictionary**: the core's `locales/en.json` with checks that every key the core can emit exists; `t()` and
  `tText()` in the UI with plurals and `Intl` formats.
- **05 IPC and shell**: the composition root, eleven allowlisted commands and one event channel, one instance, quit on
  close, an incognito webview; the generated TypeScript types; the UI's IPC module, store and a bare scan screen; the
  core e2e on the `demo` module against `fixtures/golden/demo.json`.
- **06 Module kit**: `fresher-rules` (TOML rules, a pure matcher), the module harness and the category contract in
  `fresher-testing`, the template and checklist in [`architecture/modules.md`](../../architecture/modules.md).
- **07 Boundaries and docs**: the crate dependency guard; the docs describe files instead of SQLite and the crates that
  were built; version 0.1.0.

### Measurements (the branch's last commit)

- `npm run pre-push` passes. Rust: 358 tests (`cargo nextest`), 98.2 % of the workspace's lines covered
  (`cargo llvm-cov`): core 99.8 %, rules 99.8 %, engine 99.4 %, demo 97.7 %, store 97.6 %, testing 96.4 %, app 87.8 %.
  UI: 42 tests, 100 % of statements, functions and lines, 97.4 % of branches. Repository scripts: 19 tests.
- `npm run test:e2e` passes: 6 core e2e tests, 12 Playwright tests (WebKit and Chromium).
- `npm run build` bundles `Mac Fresher.app` 0.1.0: 5.5 MiB (release, without the `demo` module's registration).

### Deviations from the design

All are recorded as agent's calls in [`00-design.md`](./00-design.md), "Agent's calls (to check)", and in the steps'
"Notes from the implementation":

- Calls 15–17 (made while writing the steps): the allowlisted commands, the single-instance plugin and the incognito
  window; findings batched by count, time and module end; the version bump in the last step.
- Calls 18–24 (plan 01): `CommandRunError`; kebab-case tags and adjacently tagged enums; ids as strings in commands;
  `core` a reserved module id; `~` kept in folder settings; the settings file's format constants; a dictionary that
  refuses duplicate and invalid keys.
- Calls 25–27 (plan 05): the settings file's state in the IPC contract; an unreadable settings file is read-only; the
  core dictionary merged in the registry.
- Calls 28–29 (finishing): a refusal carries the core's text; camelCase on the IPC boundary.
- The stand-ins until later plans (no disk walk until 004, access `Unknown` until 006, `CACHEDIR.TAG` read by the
  rules, command arguments by value, the `demo` module linked in release builds) are in
  [`../../hacks/`](../../hacks/README.md).

### Review fixes (core)

A code review of the plan found these defects in the core crates; each is fixed with a test that failed before:

- **A panicking module** killed the scan thread (the engine stayed `Scanning`; a release build aborted). The release
  profile unwinds; every module call runs under `catch_unwind` and fails alone (`core.module.panicked`); any other
  panic still ends the scan `Partial` (decision of 2026-11-15 in the design).
- **`shutdown` waited for ever** on the scan thread: it now waits at most 5 s, then detaches the thread.
- **A failed thread spawn wiped the previous results**: the scan's thread empties them as it starts.
- **Scan progress was not throttled**: at most one `ScanProgress` per 50 ms on the clock, and the last one always.
- **Scan roots were not checked**: a folder setting accepts only `~`, `~/…` or a plain absolute path outside the
  protected system folders (`SettingProblem::RefusedFolder`).
- **An atomic write failed after its rename** when the folder could not be synced: now logged, the write succeeds;
  temporary files are named `.<target>.<random>.tmp`. Removing stale ones is a
  [known issue](../../known-issues/store-stale-temporary-files.md), left to plan 005.
- **`CACHEDIR.TAG` was checked, then opened by path** (a race): it is opened once with `O_NOFOLLOW | O_NONBLOCK` and
  checked through the handle.

### Checks by hand (done by the user, 2026-11-15)

Nothing was tried in a real window: the agent cannot open one, and `tauri-driver` does not run on macOS. The UI was
tested on a mocked core (Vitest and Playwright), the Rust side end to end without a window. To check by hand:

1. `npm run tauri dev` opens the app; the window shows "Version 0.1.0" and the "Scan" button.
2. "Scan" streams the demo findings into a list grouped by category, with every label from the dictionary (no raw key
   such as `demo.caches.name` on screen), each with a size and a danger level.
3. "Scan" pressed again while a scan runs shows the refusal "The app is already running a scan…" (the demo scan is
   short: press twice quickly).
4. Launching the app a second time focuses the first window instead of opening another (the mocked runtime of
   `tauri::test` does not track focus).
5. Closing the window ends the process (nothing left in Activity Monitor).
6. `~/Library/WebKit/dev.macfresher.app` is not created (the incognito webview).
7. `~/Library/Application Support/dev.macfresher.app/` is created on the first start, readable by the user only, and
   holds no `settings.json` yet (a setting is saved only by `update_setting`, and no screen calls it until plan 005).
8. Optionally, open the built `target/release/bundle/macos/Mac Fresher.app`: it starts, and "Scan" finds nothing (the
   release build registers no `demo` module and has no disk walk until plan 004).
