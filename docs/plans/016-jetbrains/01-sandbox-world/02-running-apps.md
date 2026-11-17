# 016/01/02 — Running apps

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
sandbox, app, app/ui

## Files

- `crates/sandbox/src/world.rs` (new) — `World { running: Vec<RunningApp> }`: from every case's `[[case.app]]` with
  `running = true` (its name and bundle id) and `[[case.running]]` (`name`, `bundle-ids`); written as `world.toml` at
  the sandbox's root, read back by `World::read(sandbox)`; a missing file is an empty world. `World::running_apps()`
  gives a `FakeRunningApps`.
- `crates/sandbox/src/{case,error,generate,layout,lib}.rs` — `[[case.running]]` parsed and checked (a bundle id
  required and valid); `world.toml` is the sandbox's own file (reset, never listed as foreign); `all` joins the worlds.
- `crates/sandbox/src/expected.rs` — "(open)" after an installed app that runs; "Open, not installed: <name>".
- `app/tauri/src/sandbox.rs` — `SandboxPorts::real()` keeps every real port but `running`, which comes from the
  sandbox's `World` (decision 7); `Build::Sandbox` carries `simulated_running: true`.
- `app/tauri/tests/sandboxes/harness.rs` — the harness takes `running` from the generated sandbox's `world.toml`
  instead of an empty fake.
- `app/ui/src/product/build-banner/` — the sandbox banner adds "Open apps are simulated."; its test and story.
- `app/ui/src/locales/en.json`; the bindings regenerated if `Build` changes.

## Produces

Declared running apps in the autotest and in sandbox mode, for 03/01.

## Checks

- Negative first: a running entry with no or an invalid bundle id is refused; a sandbox with no `world.toml` has no
  running app (and never the real Mac's: a test with a real-looking port proves `SandboxPorts` takes the world's).
- Positive: an app declared `running = true` is in `running_apps()` with its bundle id; `[[case.running]]` too; `all`
  joins two case files' worlds; the banner's text.
- `cargo nextest run -p fresher-sandbox -p fresher-app`; `npm run sandbox:test` (every existing golden unchanged);
  `npx vitest run src/product/build-banner/`.

## Conclusions

Built as written. `world.toml` at the sandbox's root (`World`, in `crates/sandbox/src/world.rs`) holds the running
apps of every case (`running = true`, `[[case.running]]`), joined in `all`; a sandbox without a running app gets no
`world.toml` at that time, so the goldens did not change (later the file always carries `now`, step 02/04). Sandbox
mode takes its running apps from it, never from the real Mac (`SandboxPorts::real`); `Build::Sandbox` carries
`simulatedRunning` and the banner says "Open apps are simulated". The autotest harness reads the same file. A bad
`world.toml` refuses the start (`SandboxRefusal::World`).
