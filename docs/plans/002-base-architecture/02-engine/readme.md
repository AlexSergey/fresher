# 002/02 — Engine

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** engine, modules

`fresher-engine`: the module registry, the event bus, the session state machine, the scan session over the `DiskWalk`
port, the settings in memory; and the `demo` module. The design: [`../00-design.md`](../00-design.md), "A module",
"Events and commands", "Sessions".

Depends on plan 01. Runs in parallel with 03, 04, 06/01–03 and 07/01: it owns `crates/engine/` and `modules/demo/`,
and edits no other crate. The engine depends on `fresher-core` only (plus `tracing`); its tests use fakes written in
its own `#[cfg(test)]` modules (a scripted walk, an in-memory settings store), so it does not touch `crates/testing/`.

## Steps

| #   | Step                                                  | Status      | Shared files                                  |
| --- | ----------------------------------------------------- | ----------- | --------------------------------------------- |
| 01  | [Registry and event bus](./01-registry-and-bus.md)    | ✔️ done     | `Cargo.lock` (a new crate under `crates/*`)   |
| 02  | [The state machine](./02-state-machine.md)            | ✔️ done     | —                                             |
| 03  | [The scan session](./03-scan-session.md)              | ✔️ done     | —                                             |
| 04  | [Settings in the engine](./04-settings.md)            | ✔️ done     | —                                             |
| 05  | [The demo module](./05-demo-module.md)                | ✔️ done     | root `Cargo.toml` (members), `Cargo.lock`     |

Steps 01–04 are in order. Step 05 depends only on plan 01; it is last so the plan's worktree commits one step at a
time, but it may go first.

## Conclusions

Built (2026-11-15): `fresher-engine` (depends on `fresher-core`, `thiserror`, `tracing`) and `fresher-module-demo`;
70 engine tests and 9 demo tests; `cargo llvm-cov nextest`: 99.4 % of the engine's lines, 98 % of the demo's, 99.0 % of
the workspace.

- **The engine's API**: `Registry` (`register`, `modules`, `categories`, `schema`, `dictionary`; `RegistryError`),
  `EventBus` (`subscribe`, `publish`), `Ports { walk, settings, clock, commands, access }`, `Engine` (`new`,
  `subscribe`, `status`, `start_scan`, `cancel_scan`, `shutdown`, `overview`, `findings`, `finding`, `settings`,
  `settings_file`, `update_setting`, `modules`, `dictionary`, `access_status`; dropping it shuts it down) and
  `SettingsFile`.
- **The demo module**: `DemoModule::new()`, `DemoError`, `DEMO_ROOT`; `modules/*` joined the workspace members.

Deviations from the steps (each written in its step file, "Notes from the implementation"):

- `Engine::new` returns `Result<Self, SchemaError>` and `Registry::schema()` a `Result`: the schema is built by checked
  constructors, and there is no panicking path.
- The core has no `read_only` in `SettingsView` and no settings problem for a broken file: the engine gives them as
  `Engine::settings_file() -> SettingsFile`. **To check in the main session**: add `read_only` (and a file problem)
  to the core for plan 05, or keep the separate call.
- `StoreError::Io` on load gives writable defaults, like `Broken` (to check).
- A scan ends `Partial` also when a module failed or a root could not be read; the roots are passed to the walk as
  written (`~` is the app's to fill in); finding ids keep increasing across scans.
- Settings loading moved forward into 02/03 (the disabled-module check needed it); `error.rs` came with 02/02.
- The demo module depends on `thiserror` too (its fallible constructor), and its tool is a made-up `demo-tool`.
- The merged dictionary holds the modules' texts only; the core's texts (plan 04) are merged in by plan 05.
