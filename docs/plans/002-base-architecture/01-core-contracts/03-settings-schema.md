# 002/01/03 — Settings schema and file versions

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** core

## Files

- `crates/core/src/settings.rs` — `SettingKey`, `SettingKind`, `SettingValue`, `Unit`, `SettingDescriptor` (label and
  description keys derived: `<key>.label`, `<key>.description`), `SettingsSchema` (built from the core's descriptors
  and each module's; adds `<module>.enabled`, a switch defaulting to on), `SettingsSchema::resolve(&StoredSettings) ->
  ResolvedSettings { values, problems }`, `SettingsSchema::check(key, value)`, `SettingProblem`
  (`UnknownKey`, `WrongType`, `OutOfRange`, `UnknownChoice`), `ModuleSettings` (a module's view of its values).
- `crates/core/src/settings/core_schema.rs` — the core's settings: `core.scan.roots` (folders, default the home folder,
  stored as `~`).
- `crates/core/src/store.rs` — the `SettingsStore` port, `StoredSettings` (raw `serde_json::Value`s by key),
  `FormatVersion`, `VersionPolicy { Migrate, Discard }`, `StoreError` (`NewerVersion { found, supported }`, `Broken`,
  `Io { kind }` — no path inside, paths are not logged).

## Produces

The schema the engine resolves (02/04), the port the store implements (03/02), the version policy every file follows.

## Checks

- Negative: `falls_back_to_the_default_for_a_value_of_the_wrong_type`,
  `reports_an_unknown_key_and_keeps_it_out_of_the_values`, `rejects_a_number_outside_its_range`,
  `rejects_a_choice_not_in_the_list`, `rejects_a_module_setting_outside_its_namespace`,
  `has_no_key_that_turns_off_the_confirmation_or_lowers_a_danger_level` (no key in the core schema mentions
  confirmation or danger).
- Positive: `resolves_a_valid_value`, `adds_an_enabled_switch_for_every_module`, `defaults_the_scan_roots_to_home`,
  `derives_the_label_and_description_keys`.
- Property: `resolve_never_fails_whatever_the_stored_json` (arbitrary JSON values give values + problems, no panic).

## Notes from the implementation

- The settings code is split in three files: `settings.rs` (key, kind, value, descriptor, problem),
  `settings/schema.rs` (`SettingsSchema`, `ResolvedSettings`, `ModuleSettings`) and `settings/core_schema.rs`.
- `ModuleDescriptor::new` now takes the module's settings (`id, categories, settings, locales`) and refuses a setting
  outside the module's namespace (`ForeignSetting`), a duplicate (`DuplicateSetting`) and `<module>.enabled`
  (`ReservedSetting`: the schema adds it). `ModuleContext::new` takes the module's `&ModuleSettings`.
- `SettingsSchema::new(modules)` returns `SchemaError::DuplicateKey` for two modules with one id. `SettingProblem`
  is serialized with a `kind` tag; `SettingDescriptor::new` refuses a default that does not fit its kind
  (`SettingError::BadDefault`), `SettingKey::new` an invalid key (`SettingError::InvalidKey`).
- `SettingDescriptor::read(json)` reads a plain stored JSON value (`true`, `30`, `"fast"`, `["~"]`); a float, a
  string for a number or anything else of the wrong shape is `WrongType`. `SettingValue::to_json` writes it back.
- Folder values are kept as written: `~` stays `~`; the app fills in the home folder (the core does no IO).
- `ModuleSettings` reads a module's values by the name after `<module>.` (`switch`, `number`, `choice`, `folders`,
  `get`); `ModuleSettings::empty(module)` is the view of a module without resolved settings.
- The settings file's constants sit on `StoredSettings`: `FORMAT = "settings"`, `VERSION = FormatVersion(1)`,
  `POLICY = VersionPolicy::Migrate`. `StoredSettings` is a transparent map of raw JSON values.
- **Fixed after review: folder settings are checked.** `SettingKind::Folders` used to accept any path. Each folder must
  now be `~`, `~/…` or an absolute path of plain names (no empty name, `.` or `..`), and not a protected system folder
  of [`deletion-safety.md`](../../../restrictions/deletion-safety.md), nor one inside it or above it (`/`, `/private`,
  `/var`, `/Users`; `/usr/local` stays allowed); case is ignored, as on APFS. A refused folder is
  `SettingProblem::RefusedFolder { key, folder }` (a new IPC variant: the TypeScript bindings are regenerated) and the
  setting keeps its default; through `update_setting` it is `InvalidSettingValue`. The check is
  `crates/core/src/settings/folders.rs`.
