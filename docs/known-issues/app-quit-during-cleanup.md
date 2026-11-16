# Quitting from the Dock during a cleanup does not ask

**Status:** open, narrowed · **Updated:** 2026-11-16

## Symptom

Closing the window, **⌘Q** and the app menu's **Quit** during a cleanup are held, and the UI asks first ("A cleanup
is running. Stop after the current item and quit?", plan 008, section 9). Quitting with the Dock's **Quit** does not
ask: the cleanup stops after the item in flight (the engine waits for it at most 5 s), its journal is closed, and the
app ends.

## Cause

The Dock's Quit ends the app through macOS's `terminate:`. The Tauri runtime (`tao` 0.37) answers it in
`applicationWillTerminate`, which cannot be refused, and reports it only as `RunEvent::Exit`: there is no
`ExitRequested` to prevent, as there is when the last window closes.

## What was tried

- Plan 008/06/04 holds the window's close (`CloseRequested`) and shuts the engine down on `RunEvent::Exit` too, so
  the quit never leaves a cleanup half recorded: the item in flight finishes or, past 5 s, is shown as unknown in the
  history.
- Plan 008/07 replaces the default menu's **Quit** with an item of the app's own (same title, same ⌘Q;
  `app/tauri/src/shell.rs`, `menu` and `on_menu_event`): while a cleanup runs it publishes `quit-requested` and the UI
  asks, as for the window's close; otherwise the app exits. The menu itself needs the main thread, so its shape is not
  covered by a test; the held quit is (`quit_from_the_menu_while_cleaning_is_held_and_asks`).

## Options left

- A runtime that implements `applicationShouldTerminate:` (a change in `tao`), so every quit can be held, the Dock's
  too.
