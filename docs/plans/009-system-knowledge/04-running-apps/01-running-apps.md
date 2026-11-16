# 009/04/01 — The running apps

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** macos, system, app

## Files

- `crates/macos/src/process.rs` — `running_processes() -> io::Result<Vec<Process>>` (`pid`, short name, executable),
  over the existing `libproc` calls; `MacOpenFiles` shares the listing of process ids.
- `crates/system/src/running.rs` (new) — `MacRunningApps`: each executable's enclosing bundles (`.app`, `.appex`,
  `.xpc`), their ids through `bundle.rs` (cached per call), the outermost app's name; processes outside a bundle left
  out. The mapping is a pure function tested on fixture bundles.
- `app/tauri/src/compose.rs` — `MacRunningApps` in the system ports.

## Checks

- Negative: an executable outside any bundle gives no app; a bundle without an id gives no id.
- Positive: a helper inside an app gives both ids and the app's name; the read-only check lists at least one process.

## Conclusions

- Built as written. `fresher_macos::running_processes()` gives every process but this one with its short name and
  executable, over the existing `libproc` calls (no new `unsafe`); `MacOpenFiles` keeps its own walk of the user's
  processes — both start from `libproc::all_pids`, so no second listing code was written.
- `MacRunningApps` maps each executable to its enclosing bundles (`.app`, `.appex`, `.xpc`), innermost first; the name
  is the outermost bundle's; each bundle is read once per call, the same app from many processes is listed once.
  `fresher-system` now depends on `fresher-macos` (allowed by the crate guard's table).
- The app composes `MacRunningApps`; the stand-in `NoRunningApps` is gone.
- The read-only checks: a child `/bin/sleep` is in the process table with its executable and name; this Mac's table
  can be listed.
