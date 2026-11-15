# 002/05 — IPC and shell

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** app, app/ui, e2e

The composition root and the typed path between the UI and the engine: generated TypeScript types, the commands, the
event channel, the capabilities, the shell's behaviour (one instance, quit on close, incognito webview), the UI's IPC
module and store, and the end-to-end test on the `demo` module. The design: [`../00-design.md`](../00-design.md),
"Events and commands", "IPC safety", "Sessions".

Depends on plans 02, 03 and 04. Runs in parallel with 06/04 (which touches only `modules/demo/` and the kit docs).

## Steps

| #   | Step                                                     | Status      | Depends on  | Shared files                                                              |
| --- | -------------------------------------------------------- | ----------- | ----------- | ------------------------------------------------------------------------- |
| 01  | [Generated TypeScript types](./01-generated-types.md)    | ✔️ done     | —           | `Cargo.lock`, `app/ui/src/i18n/types.ts`, `eslint.config.ts`, `knip.json`, `_typos.toml` (only if they need the generated file excluded) |
| 02  | [Commands and the event channel](./02-commands.md)       | ✔️ done     | 01          | `crates/engine/src/` (core dictionary in the merge), `docs/hacks/README.md`, `Cargo.lock` |
| 03  | [Shell behaviour](./03-shell-behaviour.md)               | ✔️ done     | 02          | `Cargo.lock`                                                              |
| 04  | [The UI's IPC module and store](./04-ui-ipc-and-store.md) | ✔️ done     | 01         | —                                                                         |
| 05  | [End to end on the demo module](./05-end-to-end.md)      | ✔️ done     | 02          | `fixtures/golden/`, `Cargo.lock`                                          |

Steps 03, 04 and 05 are independent of each other once their dependencies are done; they touch separate folders
(`app/tauri/`, `app/ui/`, `crates/e2e/`) and may run in parallel worktrees.

## Conclusions

Built (2026-11-15): the composition root, the commands and the event channel, the shell's behaviour, the UI's IPC
module and store, and the core e2e on the `demo` module. `npm run pre-push` passes: 352 Rust tests, 98.2 % of the
workspace's lines (`cargo llvm-cov`); 42 UI tests, 100 % of statements, functions and lines and 97.3 % of branches.
`npm run test:e2e` passes (6 core tests, 12 Playwright tests in WebKit and Chromium); `npm run build` bundles
`Mac Fresher.app` (5.5 MiB, release: no demo module).

- **Commands** (`app/tauri/src/commands.rs`, all `async`, off the main thread): `app_version`, `subscribe(channel)`,
  `start_scan`, `cancel_scan`, `get_overview`, `get_modules`, `get_findings(query)`, `get_settings`,
  `update_setting(update)`, `get_dictionary(locale)`, `get_access_status`. The same list is in `generate_handler!`,
  in `build.rs`'s app manifest and as `allow-<command>` in the capability, with no core or plugin permission (fixed
  after review: `core:default` was dropped); a test reads the three files.
- **Events**: `subscribe` forwards every `Event` of the engine's bus to a `Channel<Event>` from one thread for the
  whole app (fixed after review: a new `subscribe` replaces the channel instead of adding a thread); the UI's
  `subscribe(onEvent)` creates the channel.
- **Generated types**: `app/tauri/tests/bindings.rs` renders every IPC type with `ts-rs` into
  `app/ui/src/ipc/bindings.generated.ts` (byte for byte, not formatted; `UPDATE_BINDINGS=1` rewrites it) and checks
  that no listed type refers to an unlisted one. Checked by hand: a field added to `Finding` without regenerating
  fails the test.
- **The shell**: one instance (`tauri-plugin-single-instance` 2.5.5), the engine shut down on `ExitRequested`, the
  window `incognito`, the CSP unchanged with no remote origin.
- **The UI**: `src/ipc/` (one function per command, `CommandRefused` and its text), `src/state/` (`reduce`,
  `createCoreStore`, `useCore`, `connectCore`), a bare screen in `app.tsx`; screenshots updated (`start-screen`) and
  added (`finished-scan`), looked at.
- **The core e2e**: `crates/e2e/tests/demo.rs` against `fixtures/golden/demo.json`; settings survive a restart; a
  newer settings file is never overwritten.

Changes to the core and the engine (the main session's decisions, agent's calls 25–27 of the design):

- `SettingsView.file: SettingsFile` (`loaded`, `newer-version`, `broken`, `unreadable`), moved from the engine to the
  core; `Engine::settings_file()` removed; the text of `core.error.settings-read-only` names both causes.
- A settings file that could not be read is read-only, never saved over (its own `fix(engine)` commit first).
- `Registry::new()` starts with the core's dictionary and returns a `Result` (`RegistryError::CoreDictionary`).

Deviations from the steps (each written in its step file, "Notes from the implementation"):

- The adapters are in `app/tauri/src/adapters.rs`, the tests' mocked shell in `testing.rs` (`tauri`'s `test` feature
  for the dev-dependency); `get_modules`, `get_access_status` and `app_version` return their value without `Result`.
- The UI rebuilds a refusal's `Text` from the serialized `CommandError` (key `core.error.<code>`, fields as args);
  ESLint's `camelcase` allows the three snake_case fields of the generated types. Both are to check: the core could
  send the error's text, and serialize its fields in camelCase. Settled in 07/02 by the design's agent's calls 28 and 29: the
  core sends a `Refusal` with its text, and the IPC fields are camelCase.
- `app/ui/src/i18n/types.ts` takes `Completeness` and `Dictionary` from the generated file too.
- The store is three files (`reduce.ts`, `store.ts`, `connect.ts`) and `App` takes the store as a prop.
- `docs/hacks/` gained: no disk walk until plan 004, access `Unknown` until plan 006, `CACHEDIR.TAG` read by the rules,
  command arguments by value, the demo module linked in release builds.

Not verified: nothing was tried in a real window (the agent cannot open one). Still to do by hand: `npm run tauri dev`
opens the app and "Scan" streams the demo findings; a second scan during a scan shows the refusal; a second launch
focuses the first window; closing the window ends the process; `~/Library/WebKit/dev.macfresher.app` is not created.
The mock runtime of `tauri::test` does not track focus, so the second-launch test only proves the window is found.
