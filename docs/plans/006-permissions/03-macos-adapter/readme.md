# 006/03 — macOS adapter and shell

**Status:** ✅ approved · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** macos, app, app/ui

The macOS half: what macOS guards and how to tell its state without a prompt, in `fresher-macos::privacy`; the app's
two adapters that put it behind the core's ports; and the shell's part — commands, the re-check when the window gains
focus, the reason strings in `Info.plist`, and the UI's IPC functions. The design: [`../00-design.md`](../00-design.md),
sections 3, 4, 7 and 11.

Depends on plan 01 (the measured places, candidates and addresses), 02/01 (the ports) and plan 004/02 (`fresher-macos`
exists). Step 04 also needs 02/03.

## Steps

| #   | Step                                                          | Status   | Depends on       | Shared files                                                                        |
| --- | ------------------------------------------------------------- | -------- | ---------------- | ----------------------------------------------------------------------------------- |
| 01  | [Guarded places and checks](./01-guarded-places-and-checks.md) | ✅ approved | 01/01, 004/02   | `crates/macos/src/lib.rs`                                                           |
| 02  | [Settings panes](./02-settings-panes.md)                      | ✅ approved | 01/02, 004/02    | `crates/macos/src/lib.rs`                                                           |
| 03  | [The app's adapters](./03-app-adapters.md)                    | ✅ approved | 01, 02, 02/01    | `app/tauri/src/{adapters,compose}.rs`, `docs/hacks/README.md`                        |
| 04  | [Commands, focus and `Info.plist`](./04-commands-focus-plist.md) | ✅ approved | 03, 02/03     | `app/tauri/src/{commands,shell,lib}.rs`, `app/tauri/build.rs`, `app/tauri/capabilities/default.json`, `app/ui/src/ipc/` |

Steps 01 and 02 touch separate files of `crates/macos/src/privacy/` and may run in parallel.

## Conclusions

Not written yet.
