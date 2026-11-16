# 005/01/03 — Rescan to apply, reset all

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, engine, app, app/ui (ipc)

## Files

- `crates/core/src/command.rs` — `LastScan.settingsChanged` (`bool`, always sent).
- `crates/engine/src/settings.rs` — `Settings::reset`: every value but the ignore list back to its default, saved once
  through the port; answers the `SettingChanged` events of the values that changed. Refused with `SettingsReadOnly`
  on a read-only file.
- `crates/engine/src/{engine,results}.rs` — a change of a `next-scan` setting marks the last scan; the next scan starts
  unmarked; `Engine::reset_settings` (kills the pending plan when a `next-scan` value changed).
- `app/tauri/src/{commands,lib}.rs` — `reset_settings`; `app/ui/src/ipc/index.ts` — `resetSettings`; bindings.

## Checks

- Negative: a change of a `now` setting does not mark the scan; reset is refused on a read-only file; reset keeps the
  ignore list; a reset of defaults publishes nothing.
- Positive: a change of the roots marks the last scan; the next scan clears the mark; reset saves once and announces
  each change; the command test in `app/tauri`.
