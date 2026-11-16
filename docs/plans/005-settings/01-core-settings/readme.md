# 005/01 — Core settings

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, engine, app

The catalogue of the core's settings: where each goes and when it applies, the two new kinds of the ignore list, the
guard rails as a test, "Rescan to apply" and "Reset all settings" in the engine. The design:
[`../00-design.md`](../00-design.md), sections 1–3.

## Steps

| #   | Step                                                     | Status      | Shared files                                                          |
| --- | -------------------------------------------------------- | ----------- | --------------------------------------------------------------------- |
| 01  | [Groups, effects and motion](./01-groups-and-effects.md) | ✔️ done    | `crates/core/locales/en.json`, `bindings.generated.ts`                |
| 02  | [The kinds of the ignore list](./02-ignore-kinds.md)     | ✔️ done    | `crates/core/locales/en.json`, `bindings.generated.ts`, the row       |
| 03  | [Rescan to apply, reset all](./03-rescan-and-reset.md)   | ✔️ done    | `crates/engine/src/engine.rs`, `app/tauri/src/`, `app/ui/src/ipc/`    |

In order.

## Conclusions

Built as written, and committed with plan 02 in one commit (`feat(core): 005/01–02`): both change the IPC types,
and the bindings and the UI's fixtures had to move together for the hooks to pass.

- `SettingDescriptor` carries `group` and `effect`; `SettingDescriptor::new` (a module's) always makes `module` /
  `next-scan`, the core's own come from `unchecked` in `core_schema.rs`. The guard test lists the words no core key
  may hold (`confirm`, `danger`, `level`, `removal`, `remove`, `delete`, `permanent`, `trash`).
- `core.motion` (`system` / `reduced`), `core.ignore.categories` (`ids`) and `core.ignore.paths` (`paths`, absolute
  plain names, anywhere). A bad id is the new problem `refused-id`; a bad path reuses `refused-folder`.
- `LastScan.settingsChanged` is always sent (a plain `boolean`, not omitted when false, so the UI's type stays simple).
- `reset_settings` drops every stored key but the ignore lists, saves once, publishes one `SettingChanged` per value
  that changed, and is refused on a read-only file even when nothing would change, so the screen never claims a
  reset that could not be saved. A change of the roots, a module's switch or an ignore list kills the pending plan.
