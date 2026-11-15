# 002/02/04 — Settings in the engine

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** engine

## Files

- `crates/engine/src/settings.rs` — at `Engine::new`: load through the `SettingsStore` port and resolve against the
  schema; `Engine::settings() -> SettingsView` (schema, values, problems, `read_only`);
  `Engine::update_setting(SettingUpdate)`: checks the value, saves through the port, then publishes `SettingChanged`.
  A `NewerVersion` file makes the settings read-only in memory (`SettingsReadOnly` on update); a `Broken` file gives
  defaults plus a problem. A save failure keeps the old value and returns `Storage`.
- The scan session reads `core.scan.roots` and the `<module>.enabled` switches from here.

## Produces

The settings behaviour plan 005 builds its screen on; `get_settings` and `update_setting` for 05/02.

## Checks

- Negative: `refuses_an_unknown_setting`, `refuses_a_value_of_the_wrong_type`,
  `keeps_working_on_defaults_when_the_file_is_newer`, `never_saves_when_the_file_is_newer`,
  `keeps_the_old_value_when_saving_fails`.
- Positive: `persists_and_announces_a_change`, `a_disabled_module_is_left_out_of_the_next_scan`.
- `cargo llvm-cov nextest -p fresher-engine` ≥ 90 % of lines; plan 02's conclusions written after step 05.

## Notes from the implementation

- **`SettingsView` of the core has no `read_only` field, and `SettingProblem` has no variant for a broken file**; the
  core is frozen for this plan, so the engine answers them separately: `Engine::settings()` is the core's
  `SettingsView` (schema, values, problems), and `Engine::settings_file()` gives a `SettingsFile`
  (`Loaded`, `NewerVersion`, `Broken`, `Unreadable`; `is_read_only()` is true for `NewerVersion`). Plan 05 can show
  both, or the core gains `read_only` and a file problem in the main session (to check).
- `StoreError::Io` on load gives the defaults and keeps the settings writable, like `Broken`; only `NewerVersion` is
  read-only (to check: a passing read error could let a save replace the user's values with the defaults). **Changed
  in plan 05** (the main session's decision): `Unreadable` is read-only too, so a save never follows a read error.
- An update is checked in this order: unknown key (`UnknownSetting`), the value against its descriptor
  (`InvalidSettingValue`), read-only (`SettingsReadOnly`); then the whole stored map, unknown keys kept, is saved
  through the port, and only then the value is taken, the key's problem cleared and `SettingChanged` published. A
  successful save after a broken or unreadable file makes it `Loaded`.
- A running scan keeps the settings it started with (02/03).
- `cargo llvm-cov nextest -p fresher-engine`: 99.4 % of lines.
