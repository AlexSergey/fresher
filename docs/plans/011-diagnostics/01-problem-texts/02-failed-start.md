# 011/01/02 — A failed start says so

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** app, core

## Files

- `app/tauri/src/start_failure.rs` (new) — `fn start_failure_message(error: &dyn Display, debug: bool) -> String`:
  the template of `core.start.failed` from `core_dictionary()`, and in a debug build a blank line and the error's
  `Display` under it. Pure; no window. If the core's dictionary cannot be read (a bug the core's tests already catch),
  a fixed English sentence of the same meaning stands in.
- `app/tauri/src/lib.rs` — the body of `setup` moves into one function returning its error; on an error the setup
  logs it (as today), shows `start_failure_message(&error, cfg!(debug_assertions))` with
  `app.dialog().message(…).kind(MessageDialogKind::Error).blocking_show()`, and returns the error, so the app ends as
  today. The identity check, the data folder, `compose::start` (sandbox refusals included) and `DataFolder::open` all
  go through it.
- `crates/core/locales/en.json` — `core.start.failed`: "Fresher could not start. It changed nothing on your Mac. Quit
  it and open it again; if this happens again, reinstall Fresher."
- `crates/core/src/texts.rs` — the key joins `core_keys()`.

## Checks

- Negative: `a_release_message_holds_no_error_text` (with `debug = false`, the message is the template alone, whatever
  the error), `a_message_never_names_a_placeholder`.
- Positive: `a_debug_message_shows_the_error_under_the_sentence`, `the_start_text_is_in_the_core_dictionary` (through
  `core_keys()`).
- `cargo nextest run -p fresher-app -p fresher-core`. The window itself is checked by hand in 03/01
  (`FRESHER_SANDBOX=relative npm run dev`).

## Conclusions

Built (2363560) in three rounds. The first version used a blocking dialog on the main thread, where setup runs; the
plugin's own documentation warns against it, so it became the non-blocking `show` whose callback exits the app with
code 1. A hidden window still runs script, and a command without its managed state can panic, so `report` destroys the
main window, manages a `StartFailed` marker and `shell::on_run_event` holds an exit without a code (`holds_exit`, two
unit tests) until the dialog is dismissed. `core.start.failed` is a core text; a development build adds the error under
it, a release build never does. The window itself is checked by hand in 03/01.

### Agent's calls taken during the build (to check)

1. The dialog is non-blocking and the app exits from its callback; the main window is destroyed and a `StartFailed` marker holds the exit until the dialog is dismissed. Cost if wrong: a stuck dialog keeps a half-dead app alive until the user dismisses it; the old blocking form could hang the main thread.

