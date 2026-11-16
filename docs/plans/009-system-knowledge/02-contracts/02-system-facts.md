# 009/02/02 — The facts in the module context

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, engine, testing, rules, modules, app, e2e

## Files

- `crates/core/src/system/facts.rs` (new) — `SystemPorts` and `SystemFacts` (lazy, a `OnceLock` per fact).
- `crates/core/src/module.rs` — `ModuleContext::new(clock, commands, system, settings, cancel)`, `cx.system()`.
- `crates/engine/src/{engine,session}.rs` — `Ports.system`; one `SystemFacts` per scan.
- `crates/testing/src/run_module.rs` — the harness's `Ports` carry the three fakes.
- Callers: `crates/rules/src/evaluate.rs` tests, `modules/demo`, `crates/e2e`, `app/tauri/src/compose.rs`
  (`MacKnownFolders`, and empty stand-ins for apps and running apps until 03 and 04).

## Produces

`cx.system()` for every module.

## Checks

- Negative: a scan whose modules never ask calls neither fake; a failing fake's error is answered to every ask, and
  asked once.
- Positive: two asks in one scan read the port once; a new scan reads it again.

## Conclusions

- Built as written. `SystemFacts` keeps one `OnceLock` per fact (apps, running apps, projects); an error is kept and
  answered to every ask, so a failing port is asked once per scan. `set_projects` keeps the first set.
- `ModuleContext::new(clock, commands, system, settings, cancel)`; `cx.system()`. The engine's `Ports.system` is a
  `SystemPorts`; `session::run` makes one `SystemFacts` per scan. The finish loop moved into `finish_modules` to keep
  `run` within Clippy's length (no change of behaviour).
- `fresher_testing::fake_system(home)` (home layout, nothing installed or running) for the engine's fixture, the
  e2e and the app's tests; the harness's `Ports` gained `apps`, `running` (shared, so a test reads their calls) and
  `known_folders` (`None`: the home layout under the fixture root, the same for the walk and the module).
- The app composes `MacKnownFolders` and two stand-ins, `NoInstalledApps` and `NoRunningApps`, in its adapters until
  03 and 04 replace them.
- Checks: `facts_tests.rs` in the engine (a scan whose modules never ask gathers nothing; two modules asking twice in
  each of two scans make two calls per port), and the core's own (`facts.rs`). 1419 tests pass.
