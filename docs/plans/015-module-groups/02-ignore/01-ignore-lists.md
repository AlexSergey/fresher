# 015/02/01 — The ignore lists of modules and groups

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
engine, app

## Files

- `crates/core/src/settings.rs` — `SettingKind::Ids { of: IdOf }`, `IdOf { Category, Module, Group }`; the check of
  each kind (a category id as today; a module id: one kebab-case segment; a group: a `ModuleGroup`'s serialized name).
  `core.ignore.categories` becomes `Ids { of: Category }`; its stored form is unchanged.
- `crates/core/src/settings/core_schema.rs` — `IGNORE_MODULES` (`core.ignore.modules`) and `IGNORE_GROUPS`
  (`core.ignore.groups`): Ignored, Now, `[]`; listed before the categories and paths. The guard test of forbidden
  words still passes.
- `crates/core/src/command.rs` — `IgnoreTarget::{Module { module }, Group { group }}`,
  `IgnoredEntry::{Module { module }, Group { group }}`; `IgnoredCount { categories, paths, modules, groups }`.
- `crates/core/src/error.rs` — `CommandError::UnknownModule { id }`, `UnknownGroup { id }` in `CODES`, with texts.
- `crates/core/locales/en.json` — the two settings' labels and descriptions (section 3's sentences), the two errors.
- `crates/engine/src/settings.rs` — `Ignored { categories, paths, modules, groups, hidden_modules }`;
  `hidden_modules` resolved through the registry when the lists are taken; `hides` and `hides_place` check the
  finding's module; `is_ignore_list` names the four keys; `reset` keeps all four.
- `crates/engine/src/ignore.rs` — `ignore` and `restore_ignored` for the two kinds, refusing an unregistered module or
  group.
- `crates/engine/src/results.rs` — `ignored_count` counts four lists.
- `crates/engine/src/storage/{shares,meaning}_tests.rs` — a hidden module's finding in the map.
- `crates/core/tests/typescript.rs`, `app/tauri/tests/bindings.rs` (`IdOf` listed),
  `app/ui/src/ipc/bindings.generated.ts` (regenerated), `app/ui/src/ipc/mock.ts`
  (the new kinds typed).

## Produces

The commands' new kinds and the overview's counts for 02/02, 02/03 and 02/04.

## Checks

- Negative first (engine): an unknown module or group is refused and nothing is saved; an ignored module's findings
  are in no answer and cannot be selected; a module restored inside an ignored group stays hidden; ignoring twice or
  restoring what is not ignored publishes nothing; a read-only file refuses both kinds.
- Positive: an ignored group hides every module of it and only those; restoring the group brings back what is not
  ignored on its own, without a rescan; the overview counts the lists; a change kills the pending plan; reset keeps
  the four lists; the map counts a hidden finding as your folders, or in its app's data when it names an app.
- Core unit: the `Ids` checks of each `IdOf` (an upper-case module id, an empty name, an unknown group refused); the
  new kinds round-trip; every new error code has its text.
- `cargo nextest run -p fresher-core -p fresher-engine`; the bindings test.

## Conclusions

Built as written, with these shapes: `SettingKind::Ids { of: IdOf }` (`Category`, `Module`, `Group`),
`core.ignore.modules` and `core.ignore.groups` listed before categories and paths, `IgnoreTarget` and `IgnoredEntry`
gained `Module` and `Group`, `IgnoredCount { categories, paths, modules, groups }`, `CommandError::UnknownGroup`,
`Ignored.hidden_modules` resolved by `Settings::ignored(&Registry)` (a group hides its registered modules), and
`hides_category` follows it, so every answer, the storage map's shares and the left-out drop follow. `reset` keeps all
four lists; a change of any kills the pending plan. Core and engine tests (negatives first) in
`crates/engine/src/ignore_module_tests.rs`; nextest of the workspace 1938 passed; goldens gained only the two counts.
`Ignored.modules` is a `Vec<String>`, so an id no longer registered is still counted and restorable.
