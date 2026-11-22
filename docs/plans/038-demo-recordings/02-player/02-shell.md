# 038/02/02 — The shell's demo mode

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** app/tauri, app/ui

In a development build started with `FRESHER_SANDBOX` and `FRESHER_DEMO=<absolute path of a scenario>`, the shell
reads the scenario, sizes and places the window, and offers the UI two debug-only commands: `demo_play` (the scenario
and the private texts; it writes the window's rectangle for the recorder) and `demo_finished` (writes the result and
ends the app). The UI's IPC module gets the two typed calls. The design: [`../00-design.md`](../00-design.md), section
2, agent's calls 11, 12, 13, 14.

Read first: 02/01 (as built: `Scenario`, `DemoPlay`, `DemoOutcome`), `app/tauri/src/lib.rs` (`run`, `handler`,
`start`), `app/tauri/src/compose.rs` (`start`, `Start`), `app/tauri/src/sandbox.rs` (`SandboxMode::from_env`,
`SandboxRefusal`), `app/tauri/src/start_failure.rs`, `app/tauri/src/dev_runs/commands.rs` (the classification and its
test), `app/tauri/src/shell.rs` (the main window's label), `app/tauri/tests/bindings.rs`, `app/ui/src/ipc/index.ts`
and `index.test.ts`.

## Files

- `app/tauri/src/demo.rs` (new, the module declared `#[cfg(debug_assertions)] mod demo;` in `lib.rs`) — module doc:
  demo mode of a development build, what it reads, what it writes, that it exists in debug builds only.
  - `const DEMO_VARIABLE: &str = "FRESHER_DEMO";`, `const WINDOW_FILE: &str = "demo-window.json";`,
    `const RESULT_FILE: &str = "demo-result.json";`, `const DEMO_PLACE: (f64, f64) = (80.0, 80.0);` (logical).
  - `pub(crate) struct DemoMode { scenario: Scenario, sandbox: PathBuf, private: Vec<String> }`.
  - `DemoMode::from_env(var, sandbox: Option<&Path>, real_home: &Path) -> Result<Option<DemoMode>, DemoRefusal>`:
    `None` when `FRESHER_DEMO` is unset; refused (`DemoRefusal`, thiserror) when it is set without a sandbox
    (`WithoutSandbox`), not absolute (`NotAbsolute`), or not a valid scenario (`Scenario(ScenarioError)`). `private`
    is `[real_home, sandbox]` as strings (lossy is fine: they are compared with page text).
  - `DemoMode::place(&self, window: &WebviewWindow)` — `set_size(LogicalSize(width, height))` and
    `set_position(LogicalPosition(DEMO_PLACE))`; errors logged with `tracing::warn!`, never fatal.
  - `#[tauri::command] pub(crate) fn demo_play(window: WebviewWindow, demo: State<'_, Option<DemoMode>>) ->
    Option<DemoPlay>` — `None` outside demo mode. In demo mode it writes `<sandbox>/demo-window.json`:
    `{"x":…, "y":…, "width":…, "height":…, "scale":…}` — the window's content rectangle in physical pixels relative
    to its monitor's origin (`inner_position` minus `current_monitor().position()`, `inner_size`, `scale_factor`), then
    answers `DemoPlay { scenario, private }`. A failed write is logged and the answer is still given (the check mode
    does not need the file).
  - `#[tauri::command] pub(crate) fn demo_finished(app: AppHandle, demo: State<'_, Option<DemoMode>>, outcome:
    DemoOutcome) -> Result<(), Refusal-like error>` — outside demo mode refused (no file, no exit). In demo mode it
    writes `<sandbox>/demo-result.json` (the outcome as JSON, the same shape as the IPC's), logs a failed outcome with
    `tracing::error!` naming the step and reason, and calls `app.exit(0)` for `Finished`, `app.exit(1)` for `Failed`.
    Use the error type the other commands use for a refusal if one fits without a core change; otherwise a `String`
    error is acceptable for this debug-only command (state the choice in the conclusions).
  - The file writing is a pure function over a folder (`write_window(folder, rect)`, `write_result(folder, outcome)`)
    so it is tested without a window.
- `app/tauri/src/compose.rs` / `app/tauri/src/lib.rs` — at start, in a debug build: after sandbox mode is resolved,
  `DemoMode::from_env(|n| std::env::var_os(n), sandbox_path, &home)`; a refusal stops the start like a sandbox refusal
  (the start failure's message names it). The result is managed as `Option<DemoMode>` (managed in every debug build,
  `None` without demo). In demo mode, `place` is called on the main window during setup.
- `app/tauri/src/lib.rs` — `handler()` lists `#[cfg(debug_assertions)] demo::demo_play` and `#[cfg(debug_assertions)]
  demo::demo_finished` (tauri-macros 2.7.4 accepts `#[cfg]` on a command).
- `app/tauri/src/dev_runs/commands.rs` — `demo_finished` in `ACTIONS`, `demo_play` in `READS` (array lengths bumped);
  the classification test stays green.
- `app/tauri/tests/bindings.rs` — the declarations of `DemoPlay`, `Scenario`, `DemoWindow`, `Step`, `Target`, `Role`,
  `NameMatch`, `Wait`, `AppState`, `DemoOutcome` (imported from `fresher_sandbox`) appended after the core's, in that
  order; `app/ui/src/ipc/bindings.generated.ts` regenerated with `UPDATE_BINDINGS=1 cargo nextest run -p fresher-app
  --test bindings`, never edited by hand.
- `app/ui/src/ipc/index.ts` — `COMMANDS.demoPlay = 'demo_play'`, `COMMANDS.demoFinished = 'demo_finished'` (sorted as
  the others); `export const demoPlay = (): Promise<DemoPlay | null> => call('demoPlay');` with a comment "Debug builds
  only: the scenario a demo plays, or `null` outside demo mode"; `export const demoFinished = (outcome: DemoOutcome):
  Promise<void> => call('demoFinished', { outcome });` ("the app writes the result and quits").
- `app/ui/src/ipc/mock.ts` — if the mock lists every command, `demoPlay` answers `null` and `demoFinished` nothing.

## Produces

`demo_play`, `demo_finished`, the two files, and `demoPlay()` / `demoFinished()` in the IPC module, for 02/03 and
03/01.

## Checks

- `demo.rs` tests, negatives first: `from_env` without `FRESHER_DEMO` is `None`; with it but no sandbox, a relative
  path, a missing file, an invalid scenario: each refusal; `demo_finished`'s handler outside demo mode is refused and
  writes nothing (test the inner function). Then: a valid scenario gives a mode whose `private` holds the real home
  and the sandbox; `write_window` writes the exact JSON; `write_result` writes `Finished` and `Failed` as the IPC
  shape (`{"kind":"failed","step":3,"reason":"…"}`).
- The dev-runs classification test passes with both commands classified.
- `cargo nextest run -p fresher-app` (including `--test bindings` with the regenerated file); clippy; fmt.
- `cargo build -p fresher-app --release` compiles with no demo code (the module and the two handler entries are
  `cfg(debug_assertions)`; no unused-import warning).
- `app/ui/src/ipc/index.test.ts`: `demoPlay` and `demoFinished` call `invoke` with `demo_play` and `demo_finished`
  and the outcome (negative first: a refusal rejects with `CommandFailed`). `npx vitest run src/ipc`.
- `npm run lint:ts`.

## Conclusions

- `demo_finished` refuses outside demo mode with a `String` error (`the app is not in demo mode`): no `CommandError`
  fits without a core change. The UI receives it as `CommandFailed` with an `unreachable` failure.
- A result that cannot be written ends the app with 1, whatever the outcome, so the recorder never reads a stale
  success.
- The app manifest (`build.rs`) and the capability list `demo_play` and `demo_finished` in every build: Tauri checks the
  capability against the manifest at build time and cannot vary either by `debug_assertions`. A release build's
  handler has neither command, so the two grants reach nothing (agent's call 13).
- The two tests that read the handler list from `lib.rs` (`dev_runs` classification, `tests/config.rs`) end it at the
  list's own closing bracket, because a `#[cfg(debug_assertions)]` holds a `]` too.
