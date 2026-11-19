# 006/03 — macOS adapter and shell

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** macos, system, app, app/ui,
app-leftovers

The macOS half: what macOS guards and how to tell its state without a prompt, in `fresher-macos::privacy`; the two
adapters that put it behind the core's ports, in `fresher-system` (plan 009 built it, so agent's call 2 applies); the
shell's part — commands, the re-check when the window gains focus, the reason strings in `Info.plist`, and the UI's
IPC functions; and, before the walk may enter `Library/Containers`, the deferred containers item of `app-leftovers`.
The design: [`../00-design.md`](../00-design.md), sections 3, 4, 7 and 11, and its "Probe results" (from
documentation, unverified until the user's run of plan 01).

Depends on plan 01's "Probe results" (filled, unverified), 02/01 (the ports) and plan 004/02 (`fresher-macos`), all
present on `main`. Step 04 also needs 02/03 (done).

## Steps

| #   | Step                                                                     | Status      | Depends on | Shared files                                                                                                                                                  |
| --- | ------------------------------------------------------------------------ | ----------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01  | [Guarded places and checks](./01-guarded-places-and-checks.md)           | ✔️ done     | —          | `crates/macos/src/lib.rs`, `crates/macos/src/volume.rs` (`kind_of` visibility)                                                                                |
| 02  | [Settings panes](./02-settings-panes.md)                                 | ✔️ done     | 01         | `crates/macos/src/privacy/mod.rs`                                                                                                                              |
| 05  | [Containers in `app-leftovers`](./05-app-leftovers-containers.md)        | ✔️ done     | —          | `modules/app-leftovers/`, `crates/e2e/tests/app_leftovers.rs`, `fixtures/`, `docs/categories/`, `docs/deferred/`                                               |
| 03  | [The adapters](./03-app-adapters.md)                                     | ✔️ done     | 01, 02, 05 | `crates/system/src/lib.rs`, `app/tauri/src/{adapters,compose,sandbox,testing}.rs`, `crates/scanner/src/scanner.rs` (a doc line), `crates/core/locales/en.json`, `docs/hacks/README.md`, `docs/architecture/README.md` |
| 04  | [Commands, focus and `Info.plist`](./04-commands-focus-plist.md)         | ✔️ done     | 03, 02/03  | `app/tauri/src/{commands,shell,lib,testing}.rs`, `app/tauri/build.rs`, `app/tauri/capabilities/default.json`, `app/ui/src/ipc/`, `app/ui/src/test/demo-scan.ts`, `app/ui/e2e/core-mock.ts` |

Order (2026-11-19): **01, 02, 05, 03, 04**, one by one. Step 02 now uses 01's `Guard` (`pane_of`), so they no longer
run in parallel. Step 05 was added on 2026-11-19 for the deferred containers item; it touches no file of 01 or 02 and
must be merged before 03, which is the step that lets the walk enter `Library/Containers`.

## Conclusions

The macOS half of permissions is built, step by step in the order 01, 02, 05, 03, 04:

- `fresher-macos::privacy` (01, 02): the guarded places in one table (`PLACES`), the checks that never prompt (the
  Full Disk Access candidates in `FDA_CANDIDATES`), the volume guards, and the System Settings pane of each guard with
  the fixed arguments of `/usr/bin/open`. No new FFI, no `unsafe`.
- `app-leftovers` (05): a container is attributed to its app only by the app's proof and is never offered; the module
  declares `SeesMore(OtherAppsData)`, and the walk may enter `Library/Containers`.
- `fresher-system` (03): `MacAccess` and `MacSystemUi` behind the core's ports, and `macos_version`; the normal app
  composes them with no standing avoid list, sandbox mode keeps `UnknownAccess` and its avoid list. One `~` expansion,
  `fresher_core::expand_home`, serves the scanner and the access.
- The shell (04): the commands `refresh_access`, `open_access_settings` and `relaunch`, the re-check when the main
  window gains focus (off the event loop's thread), `app/tauri/Info.plist` with the five reason strings and nothing
  else, and the UI's typed IPC functions.

Every table and address encodes the design's "Probe results" as they stand: from documentation. A release build
confirms the bundler merges `Info.plist`; the rest (which candidate tells Full Disk Access, the pane addresses, whether
a grant needs a relaunch) waits for the user's run of plan 01, which may change `PLACES`, `FDA_CANDIDATES`, the pane
addresses, or remove `relaunch`.
