# 005/02/01 — Ignore and restore

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, engine, app, app/ui (ipc)

## Files

- `crates/core/src/command.rs` — `IgnoreTarget` (`{ kind: "category", category }` | `{ kind: "finding", finding }`),
  `IgnoredEntry` (`{ kind: "category", category }` | `{ kind: "path", path }`), `Overview.ignored`
  (`{ categories, paths }`).
- `crates/core/src/error.rs` — `CommandError::CannotIgnore` (a finding of a tool's object) with its text.
- `crates/engine/src/settings.rs` — `Settings::ignored()` (the two lists), `Settings::ignore` and
  `Settings::restore`: the list changed, checked and saved as one `update`; nothing when nothing changes.
- `crates/engine/src/engine.rs` — `Engine::ignore` (a category nobody registered is `UnknownCategory`, a finding of
  another scan `UnknownFinding`), `Engine::restore_ignored`.
- `app/tauri/src/{commands,lib}.rs` — `ignore`, `restore_ignored`; `app/ui/src/ipc/index.ts`; the bindings.

## Checks

- Negative: an unknown category or finding is refused and nothing is saved; a tool's object cannot be ignored;
  ignoring twice saves once; restoring what is not ignored saves nothing; a read-only file refuses both.
- Positive: ignoring a category and a finding saves the lists and publishes `SettingChanged`; restore takes the entry
  out; the overview counts the entries.
