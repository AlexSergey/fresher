# 008/06/04 — The app wired

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** app

## Files

- `app/tauri/Cargo.toml` — `fresher-cleaner` as a dependency.
- `app/tauri/src/compose.rs` — the real adapters: `DiskCleaner::new(reach, fresher_macos::current_user(),
  fresher_macos::protection(home, data_folder, bundle)?, CleanerPorts { trash: MacTrash, open_files: MacOpenFiles,
  commands, clock })`, where `reach` is `Reach::RealDisk(RealDiskToken::new())` under `cfg(not(debug_assertions))` and
  `Reach::Sandbox` otherwise — the only place in the workspace that names the token; `FileCleanupJournal::open(&data)`;
  `MacVolumes` into the engine's `Ports`. `MacTrash` is given to the cleaner and to nothing else.
- `app/tauri/src/commands.rs` — `prepare_cleanup`, `discard_cleanup`, `confirm_cleanup`, `cancel_cleanup`,
  `get_cleanups`, `get_cleanup`, `put_back`, `reveal_in_trash`, `quit_after_cleanup`: each passes its input to the
  engine and returns its answer or a `Refusal`.
- `app/tauri/src/shell.rs` — on the window's close request: `engine.request_quit()`; while it answers `true` the close
  is prevented (the UI asks); `quit_after_cleanup` ends with the app's exit.
- `app/tauri/build.rs`, `app/tauri/capabilities/default.json` — the new commands allowlisted; the capability test
  still allows no plugin and no other `core:` permission.
- `app/tauri/tests/bindings.rs`, `app/ui/src/ipc/bindings.generated.ts` — regenerated.
- `docs/hacks/README.md` — a debug build cleans only in a sandbox folder (`Reach::Sandbox`), and why.

## Produces

The commands and events plan 07 calls; the app able to clean (a release build) or to clean a sandbox folder (a debug
build).

## Checks

- Negative (first): `confirm_with_a_mismatched_echo_is_refused` (through the command),
  `put_back_while_scanning_is_busy`, `a_debug_build_cleans_nothing_outside_a_sandbox` (the shell's test on a fixture
  without the marker: every item left out with `NotInSandbox`), `the_capability_allows_no_new_permission`.
- Positive: `prepare_then_confirm_cleans_a_sandbox_through_the_commands` (the test shell with `FakeTrash` on a fixture
  tree), `the_bindings_are_up_to_date`, `close_while_cleaning_is_held_and_quit_after_cleanup_quits`.
- `npm run tauri dev` with a sandbox folder as the scan root: a cleanup runs end to end (checked by hand, written in
  the step's conclusions).

## Conclusions

- Built as written: `compose.rs` builds the engine with `Engine::with_cleanup`; its `cleanup` function makes the
  `DiskCleaner` (`reach()`, `current_user()`, the home, `protection(home, data folder, bundle)`, `MacTrash`,
  `MacOpenFiles`, the app's command runner and clock), `FileCleanupJournal::open` and `MacVolumes` (one instance,
  shared with the scanner). The nine commands pass their input to the engine; the window's close is held while the
  engine answers `request_quit` with `true`. The bindings were regenerated: unchanged, plan 02 had exported every type.
- **The commands** (inputs as the UI names them): `prepare_cleanup { selection: CleanupSelection }` →
  `CleanupPlanView`; `discard_cleanup { plan: PlanId }`; `confirm_cleanup { confirmation: Confirmation }` →
  `CleanupId`; `cancel_cleanup`; `get_cleanups { query: CleanupsQuery }` → `CleanupsPage` (never refused);
  `get_cleanup { cleanup: CleanupId }` → `CleanupDetail`; `put_back { request: PutBackRequest }` → `PutBackReport`;
  `reveal_in_trash { item: CleanupItemRef }`; `quit_after_cleanup`. Each refusal is a `Refusal`.
- Decided where the step was silent, each the safer or the simpler way:
  - **The reach in the app's tests**: `Reach::Sandbox` under `cfg(any(debug_assertions, test))`, so a test built in
    release mode never gets the real disk either; the token is made only in a release build of the app itself.
  - **The bundle** is the outermost `*.app` folder around the executable (a bundle may hold helper apps); none in a
    development build. An executable path that cannot be read stops the app from starting (`ComposeError::Executable`)
    rather than leave its bundle out of the protected places.
  - **Quit after cleanup**: the engine's `quit_after_cleanup` (cancel, the item in flight, the journal closed, at most
    5 s), then the main window is **destroyed**, which does not ask again: the app ends as when its last window closes
    (`ExitRequested`, the engine shut down). `app.exit(0)` is used only when the window is gone; the mock runtime of
    the tests cannot exit.
  - **⌘Q, the app menu's Quit and the Dock's Quit cannot be held**: the runtime reports them only as `RunEvent::Exit`
    (`applicationWillTerminate`). The shell shuts the engine down on `Exit` too, so the cleanup stops after the item in
    flight and its journal is closed, but the user is not asked: written in
    [`known-issues/app-quit-during-cleanup.md`](../../../known-issues/app-quit-during-cleanup.md).
  - **`reveal_in_trash` answers `internal`** until plan 006/03 replaces `NoSystemUi` (the hack's entry says so).
  - **The test shell** composes the engine as the app does (`Engine::with_cleanup` with `compose::cleanup`), with
    `FakeTrash` in its own folder, `FakeOpenFiles`, `FakeVolumes` and the reference walk on
    `fixtures/homes/trial.toml`, whose scan root is set to the fixture (the reference walk does not read `~`). Outside
    a sandbox, the trial folders sit next to a `.fresher-sandbox` file that is not a marker: the module offers them and
    the cleaner leaves them out (`NotInSandbox`).
- Extra checks beyond the step: `a_discarded_plan_cannot_be_confirmed`,
  `the_cleanup_commands_refuse_what_is_not_there`, `close_while_idle_is_not_held`, the bundle's two tests and
  `the_engine_records_its_cleanups_in_the_data_folder`. `the_capability_allows_no_new_permission` compares the whole
  capability with the handler's commands and the drag region's two.
- **By hand**: a debug build starts with the real composition (the protection of the real home, the journal in the
  real data folder) and stays up. A cleanup through `npm run tauri dev` cannot be run yet: the review screen's button
  stays disabled until plan 07 ("Review cannot remove yet"); it is part of the sandbox run of step 08/02.
- `cargo nextest run -p fresher-app --release` fails the tests that need the dev-only modules (`demo`, `trial`), the
  ones before this step too; the composition test passes there, and `cargo clippy -p fresher-app --release` builds
  the release composition.
