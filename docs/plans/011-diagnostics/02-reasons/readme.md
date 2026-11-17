# 011/02 — Reasons

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
engine, testing, modules, app/ui

One way for a module to say why ([`../00-design.md`](../00-design.md), sections 3–5): the texts it may report are
declared in its descriptor; an item it recognized and did not offer is reported with `out.left_out(…)`, counted by the
engine per category and reason, and shown under the category. Failures keep their way (`ModuleError::Failed`), now
with declared texts. The reasons of plans 008 and 012 (`SkipReason`, `Coverage`, `NotScanned`, `RemainderReason`)
stay as they are.

## Steps

| #   | Step                                                          | Status                                  | Shared files                                                                                                     |
| --- | ------------------------------------------------------------- | --------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| 01  | [The module's API](./01-module-api.md)                        | ✔️ done | `crates/core/src/{lib,module,event,texts}.rs`, `crates/core/locales/en.json`, `crates/core/tests/typescript.rs`, `crates/engine/src/session.rs` (a stub), `app/ui/src/ipc/bindings.generated.ts` |
| 02  | [Counted, checked, used](./02-engine-and-contract.md)         | ✔️ done | `crates/engine/src/{session,testing}.rs`, `crates/testing/src/{run_module,contract}.rs`, `modules/{demo,trial}/` |
| 03  | [The UI shows why](./03-ui-left-out.md)                       | ✔️ done | `app/ui/src/state/reduce.ts`, `app/ui/src/screens/module/category-section.tsx`, `app/ui/src/locales/en.json`      |

In order: 01 after 01/01 (it uses `DETAIL_ARG`); 02 after 01; 03 after 02 and after 01/03 (both edit
`app/ui/src/locales/en.json` and the module screen's stories).

## Conclusions

All three steps built. A module declares its problems and reports left-outs with `out.left_out`; the engine counts
them per category and reason, the contract checks them, and the module page shows them. Corrections to the design: the
type is `LeftOutItem`; a foreign-category left-out is dropped and refused. Call 4 stands as reworded: left-outs are
event-only, so after a restart no left-out line shows until the next scan. An ignored category's left-outs are not
counted, and the module page drops a left-out-only category once it is ignored. Calls: see the steps.

