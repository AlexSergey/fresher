# 009/02 — Contracts

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, testing, engine, cleaner, system, modules, app

The core's types and ports of the facts, one scan's `SystemFacts` in every module's context, and the cancel token in
`CommandRunner`. The design: [`../00-design.md`](../00-design.md), sections 3 to 7.

## Steps

| #   | Step                                                       | Status      | Shared files                                          |
| --- | ---------------------------------------------------------- | ----------- | ----------------------------------------------------- |
| 01  | [Types, ports and fakes](./01-types-ports-fakes.md)        | ✔️ done     | `crates/core/src/lib.rs`, `crates/testing/src/lib.rs` |
| 02  | [The facts in the module context](./02-system-facts.md)    | ✔️ done     | `crates/core/src/module.rs`, the engine's `Ports`     |
| 03  | [Commands that can be stopped](./03-stoppable-commands.md) | ✔️ done     | `crates/core/src/ports.rs`                            |

In order.

## Conclusions

The three steps are built; one change from the design: the cleaner's tool removal runs to its end instead of taking
the cleanup's token (Stop ends a cleanup after the item in flight). `SystemFacts` is lazy, one per scan, and every
port has its fake.
