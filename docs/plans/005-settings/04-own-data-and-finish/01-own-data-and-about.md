# 005/04/01 — Own data and About

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, app, app/ui

## Files

- `crates/core/src/command.rs` — `OwnData { total, budget }`.
- `app/tauri/src/{commands,lib,compose}.rs` — `get_own_data` from `DataFolder::footprint`; the app keeps the data
  folder it composed the engine with.
- `app/ui/src/screens/settings/own-data-group.tsx` — "Fresher's own data takes N of its M budget"; **Reset all
  settings…** opens a confirmation ("Every setting goes back to its default. Your ignore list stays.") that calls
  `reset_settings`.
- `app/ui/src/screens/settings/about-group.tsx` — the version and the licence; the dependencies' licences stay deferred.

## Checks

- Negative: Reset all is disabled on a read-only file; cancelling the dialog sends nothing; a failed footprint shows
  nothing, not zero.
- Positive: the size and the budget are shown; confirming resets.
