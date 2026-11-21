# 029/01 — Several paths as one item

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 ·
**Modules:** core, engine, cleaner, store, testing, app/ui, docs

The capability the module needs and the core does not have: one confirmed item whose removal is several paths, each
checked by the guard before the first moves, journalled, and put back together. The design:
[`../00-design.md`](../00-design.md), "Why one item of several paths" and sections 1 to 4. This plan is the safety core
of the global plan: its tests are written to the cleaner's bar (every refusal before any act, the never property, the
race cases of a held handle).

## Steps

| #   | Step                                                            | Status      | Shared files                                                                                                     |
| --- | --------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------------- |
| 01  | [The core's types and the engine](./01-core-and-engine.md)      | 🔨 in progress | `crates/core/src/{removal,removability}.rs`, `crates/core/src/cleanup/`, `crates/engine/src/{session,claims}.rs`, `crates/engine/src/cleanup/plan.rs`, `crates/testing/src/` |
| 02  | [The cleaner](./02-cleaner.md)                                  | 🔨 in progress | `crates/cleaner/`, `docs/architecture/removal.md`                                                                |
| 03  | [The journal and put back](./03-journal-and-put-back.md)        | 🔨 in progress | `crates/store/src/journal/`, `crates/engine/src/cleanup/history.rs`                                              |
| 04  | [The confirmation](./04-confirmation.md)                        | 🔨 in progress | `app/ui/src/product/confirm-dialog/`, `app/ui/src/locales/`, `bindings.generated.ts`                             |

01 first; then 02, 03 and 04 in parallel (they touch separate crates).

## Conclusions

All four steps are built and reviewed; the cleaner is the safety core of the plan and its review found nothing open.
The changes from the design are in each step's conclusions; in short: the cancel is read before every part, the
journal gained `went` and `back`, the permanent way's partial result has its own text, and the confirmation is a
toggle that works in a virtual list. Numbers and the global conclusions: [the global readme](../readme.md#conclusions).
