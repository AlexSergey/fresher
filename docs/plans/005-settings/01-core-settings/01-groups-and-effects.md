# 005/01/01 — Groups, effects and motion

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core

## Files

- `crates/core/src/settings.rs` — `SettingGroup` (`general`, `scan`, `ignored`, `permissions`, `module`) and
  `SettingEffect` (`now`, `next-scan`) on `SettingDescriptor`, serialized; `SettingDescriptor::new` (the modules'
  constructor) makes `module` / `next-scan`.
- `crates/core/src/settings/core_schema.rs` — every core setting with its group and effect; `core.motion` (choice
  `system` / `reduced`, default `system`, General, Now).
- `crates/core/src/settings/schema.rs` — the `<module>.enabled` switch is `module` / `next-scan`.
- `crates/core/locales/en.json` — the label, description and option texts of `core.motion`.
- `app/ui/src/ipc/bindings.generated.ts` — regenerated (`UPDATE_BINDINGS=1`).

## Produces

The group and effect the screen (03/01) and the engine (01/03) read.

## Checks

- Negative: no core key names the confirmation, a danger level or the way items are removed; a module setting is never
  in a core group.
- Positive: every core setting has its group and effect; motion follows the system by default.
