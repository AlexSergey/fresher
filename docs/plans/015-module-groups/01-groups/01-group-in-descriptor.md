# 015/01/01 — The group in the descriptor

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
engine, modules, testing, app

## Files

- `crates/core/src/group.rs` (new) — `ModuleGroup { Development, Debug }` (serde kebab-case, `TS`, `Ord` in the
  screen's order), `ModuleGroup::ALL`, `name_key()` and `description_key()` (`core.group.<group>.name`,
  `.description`).
- `crates/core/src/module.rs` — `ModuleDescriptor::new(id, group, categories, settings, locales)`; `group()`. Every
  caller passes it: the core's tests, `crates/testing`'s and `crates/engine`'s test modules, `modules/demo` and
  `modules/trial` (`ModuleGroup::Debug`).
- `crates/core/src/command.rs` — `ModuleInfo.group`, filled by `ModuleInfo::new`.
- `crates/core/src/lib.rs` — the export.
- `crates/core/locales/en.json` — `core.group.development.name` "Development", `.description` "Tools for building
  software: IDEs, package managers, containers and project folders."; `core.group.debug.name` "Debug",
  `.description` "Modules of a development build, for trying the app. They never ship."
- `crates/core/src/texts.rs` — the texts test covers every group.
- `crates/engine/src/registry.rs` — `RegistryError::DebugModule`: a `Debug` module is refused when the build is not a
  development one; the check is a function taking the build flag, so both answers are tested.
- `crates/core/tests/typescript.rs`, `app/tauri/tests/bindings.rs` (`ModuleGroup` listed),
  `app/ui/src/ipc/bindings.generated.ts` (regenerated), `app/ui/src/test/demo-scan.ts`
  (the demo's `ModuleInfo` with `group: 'debug'`).

## Produces

`ModuleGroup`, `ModuleDescriptor::group()`, `ModuleInfo.group` for 01/02, 02/01 and plan 016.

## Checks

- Negative first: a `Debug` module is refused for a release build; an unknown serialized group is refused when read.
- Positive: every group has its two texts; `ModuleInfo` carries the group; the registry keeps registration order.
- `cargo nextest run -p fresher-core -p fresher-engine -p fresher-module-demo -p fresher-module-trial`; the bindings
  test; `npm run sandbox:test` (no golden changes).

## Conclusions

Built as written. `ModuleGroup { Development, Debug }` in `crates/core/src/group.rs`, `ModuleDescriptor::new(id, group,
…)`, `ModuleInfo.group`, `RegistryError::DebugModule` (a private check with the build flag; `demo` and `trial` are
`Debug`, every test module `Development`), the group texts in the core's dictionary, bindings regenerated. Tests:
1050 passed, 2 skipped across core, engine, the modules, testing, rules, e2e and app; the sandbox 21/21, no golden
changed. No contract check of the group texts was added (the core's texts test already covers them).
