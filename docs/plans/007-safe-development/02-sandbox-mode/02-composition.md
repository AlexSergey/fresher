# 007/02/02 — The sandbox composition

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** app

## Files

- `app/tauri/src/sandbox.rs` (new, every item under `cfg(debug_assertions)`) — `SandboxMode::from_env(var)` reads
  `FRESHER_SANDBOX` (an absolute path) and `FRESHER_SANDBOX_TRASH` (`real` or absent; anything else is refused)
  through `var` (`std::env::var_os` in the app; the tests inject an environment, since setting a variable is unsafe in
  edition 2024); `sandbox_engine(sandbox, real_home, real_data_folder, ports: SandboxPorts)`: the engine of section 4
  of the design — the sandbox's known folders and home, its data folder `<sandbox>/.fresher-data`, `FakeTrash` into
  `home/.Trash` (or `MacTrash` for `real`), `Reach::SandboxFolder`, the real home's protection list joined with the
  sandbox home's, and the places that prompt of both homes avoided by the walk. `real_data_folder` is the development
  build's own data folder, protected by the real Mac's list: that list cannot take the sandbox's `.fresher-data`,
  since a protected place inside the sandbox would make `SandboxFolder::new` refuse the sandbox as its ancestor; the
  sandbox home's list protects `.fresher-data`. `sandbox_engine` leaves the process's IO policy alone (the app's start
  sets it), and its error names a refusal (`SandboxEngineError::refusal`). `SandboxPorts` holds what stays real in the
  app and becomes a fake in the tests: clock, volumes, open files, running apps, commands, access.
  `pub fn sandbox_engine` is re-exported by `lib.rs` in debug builds only, for the autotests of plan 03.
- `app/tauri/src/compose.rs` — `engine` splits into the shared parts and the two callers; the normal path is unchanged.
- `app/tauri/src/lib.rs` — the setup: in a debug build, a `SandboxMode` from the environment chooses the sandbox engine;
  a refusal (no valid marker, a link, a protected place, no `home/`, a bad `FRESHER_SANDBOX_TRASH`) stops the start
  with the reason in the log and on the terminal. A release build has no such branch.
- `app/tauri/Cargo.toml` — `fresher-sandbox` and `fresher-testing` as normal dependencies, used in debug builds only
  (the hack "The demo module linked in release builds" names them too).
- `docs/hacks/README.md` — that hack's entry; "Development builds clean only in a sandbox folder" gains sandbox mode.

## Produces

Sandbox mode in the app, and `sandbox_engine` for plan 03.

## Checks

- Negative: `a_folder_without_a_marker_does_not_start`, `the_real_home_does_not_start`,
  `an_unknown_trash_choice_does_not_start`, `the_sandbox_engine_never_writes_the_normal_data_folder`,
  `a_release_build_ignores_the_variable` (a test of `cfg(not(debug_assertions))`, run by `cargo test --release` in
  05/02), `an_item_outside_the_sandbox_is_left_out` (a scan root added outside the sandbox: its findings are left out
  as "not in the sandbox").
- Positive: `the_sandbox_home_is_the_scan_home` (`~` resolves to `home/`), `the_settings_and_journal_live_in_the_sandbox`,
  `a_cleanup_lands_in_the_sandbox_trash` (on a generated `trial` sandbox, fakes for the real ports).
- `cargo nextest run -p fresher-app`.

## Conclusions

A debug build started with `FRESHER_SANDBOX` composes the engine over the sandbox. Deviation: `sandbox_engine` takes a
fourth parameter, `real_data_folder`, because the real Mac's protection list needs a data folder and the sandbox's own would
put a protected place inside it. `fresher-sandbox` and `fresher-testing` are normal dependencies (Cargo cannot gate on
`debug_assertions`) whose code is compiled out of release. Review finding: the places that prompt of the real home are
avoided too in sandbox mode. Final review: the banner's path is the canonical folder.
