# 005/01/02 — The kinds of the ignore list

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, app/ui

## Files

- `crates/core/src/settings.rs` — `SettingKind::Ids` / `SettingValue::Ids(Vec<String>)` (each a category id: two or
  more dot-separated kebab-case segments) and `SettingKind::Paths` / `SettingValue::Paths(Vec<PathBuf>)` (absolute,
  plain names, anywhere); read, checked and stored as plain JSON like the other kinds.
- `crates/core/src/settings/folders.rs` — the plain absolute path check shared by `Folders` and `Paths`.
- `crates/core/src/settings/core_schema.rs` — `core.ignore.categories` and `core.ignore.paths` (Ignored, Now, empty).
- `crates/core/locales/en.json` — their texts.
- `app/ui/src/product/settings-row/` — the two kinds render as a read-only list (the Ignored group of 03/03 is their
  real screen).
- `bindings.generated.ts` — regenerated.

## Checks

- Negative: an id that is not a category id is refused; a relative or dotted ignored path is refused; a stored list
  that is not strings is the wrong type.
- Positive: ids and paths are read from plain JSON and stored back; an ignored path may name a protected place.
