# 035/02 — The selection

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** engine, app, docs

A ticked row of the Largest screen becomes a finding of the core's category `core.own-file` (High), kept apart from the
scan's findings, so the existing selection, Review, plan, rehearsal, confirmation, cleaner, journal and put back carry
it unchanged; and the three Tauri commands of the screen. The design: [`../00-design.md`](../00-design.md), section 5.

## Steps

| #   | Step                                                   | Status      | Shared files                                                                                                          |
| --- | ------------------------------------------------------ | ----------- | --------------------------------------------------------------------------------------------------------------------- |
| 01  | [The pick, the plan and the commands](./01-own-pick.md) | 🔨 in progress | `crates/engine/src/{results.rs,cleanup/pending.rs,cleanup/operation.rs}`, `app/tauri/{build.rs,src/lib.rs,capabilities/default.json,src/storage_commands.rs}`, `app/ui/src/ipc/bindings.generated.ts`, `docs/restrictions/deletion-safety.md`, `docs/categories/README.md` |

## Conclusions

A picked row is one finding of `core.own-file` that goes through the plan, the cleaner, the journal and put back
unchanged. What changed from the design: a pick is judged again at prepare and at confirm (defence in depth), and
`gone` outlives a cancelled scan. Safety: no new deletion point; the cleaner is untouched.
