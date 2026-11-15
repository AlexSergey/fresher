# 002/03/02 — The settings file

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** store

## Files

- `crates/store/src/settings_file.rs` — `FileSettingsStore::new(DataFolder)` implements the core's `SettingsStore`:
  `settings.json`, format `settings`, version 1, policy `Migrate`.
  - Missing file → empty `StoredSettings` (all defaults).
  - Broken file → `StoreError::Broken`; the file is left as it is. The first `save` after that renames it to
    `settings.broken.json` (replacing an older one) before writing, so the user's file is kept once.
  - Newer version → `StoreError::NewerVersion`; `save` refuses and never writes.
- `crates/store/src/migrate.rs` — the migration chain: `migrations: &[fn(Value) -> Result<Value, StoreError>]`, one per
  version step, applied in order; empty for version 1. The written file is always the current version; the older one
  is replaced only by the atomic rename, after the new content is complete.

## Produces

The real `SettingsStore` the app wires (05/02) and the e2e uses (05/05).

## Checks

- Negative: `never_overwrites_a_file_from_a_newer_version`, `a_broken_file_is_an_error_and_stays_untouched`,
  `keeps_the_broken_file_aside_before_the_first_save`, `a_failing_migration_leaves_the_old_file`.
- Positive: `a_missing_file_gives_empty_settings`, `round_trips_settings`,
  `migrates_an_old_version_step_by_step` (with a two-step chain defined in the test), `writes_keys_in_sorted_order`
  (a stable diff for a user who copies the file).

## Notes from the implementation

- The file is `{ "format": "settings", "version": 1, "values": { … } }`, pretty JSON ending with a newline; the values
  are the core's `StoredSettings`, a sorted map, so the keys are written in sorted order. A body without `values`, or
  with `values` that is not an object, is `Broken`.
- `load` never writes: an older file is migrated in memory and replaced by the next `save`, through the atomic rename.
- `save` reads the file first: a newer version (or a read error) refuses the save; a broken file — also one whose
  migration fails — is renamed to `settings.broken.json` (replacing an older one) and then the new file is written.
- `migrate(body, from, to, migrations)` is `pub(crate)` in `migrate.rs`; `migrations[i]` takes version `i + 1` to
  `i + 2`, and a chain that does not cover the steps is `Broken`. The settings chain is empty, and a test checks that
  its length matches `StoredSettings::VERSION`. The migration tests use a test-only chain (`migrate::fixtures`) and a
  store built at another version with it.
- Public API: `FileSettingsStore::new(DataFolder)`, implementing `SettingsStore`.
