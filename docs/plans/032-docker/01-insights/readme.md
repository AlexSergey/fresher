# 032/01 — Insights

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** core, engine, testing, app, app/ui

A module may now describe space it never offers: an `Insight` (a title, a summary, a few labelled sizes, parts with a
command to copy, notes), given through `FindingSink::describe` and sent with the module's `ModuleFinished`. It is
never a finding, so no total, plan or map number can include it. The design:
[`../00-design.md`](../00-design.md), section 1.

## Steps

| #   | Step                                                          | Status      | Shared files                                                                                       |
| --- | ------------------------------------------------------------- | ----------- | -------------------------------------------------------------------------------------------------- |
| 01  | [The type, the sink, the event and the engine](./01-core-and-engine.md) | 🔨 awaiting the user's walk | `crates/core/src/{lib,module,event}.rs`, `crates/engine/src/session.rs`, every `ModuleFinished` literal, `bindings.generated.ts`, `app/ui/src/state/reduce.ts`, UI test scans |
| 02  | [The test kit](./02-testing-kit.md)                           | 🔨 awaiting the user's walk | `crates/testing/src/{run_module,contract}.rs`                                                      |

In order: 01, then 02.

## Conclusions

Not written yet.
