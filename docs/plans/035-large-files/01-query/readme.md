# 035/01 — The query

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** core, scanner, testing, engine

The rows of the Largest screen from the committed tree: the core's types, kinds, texts and setting; one pass of the
scanner over the tree (`DiskState::items`); the engine's two queries, Files and Folders, with one judgement of whether a
row can be chosen. No walk, cache or cleaner change. The design: [`../00-design.md`](../00-design.md), sections 1 to 4,
6 and 7.

## Steps

| #   | Step                                                     | Status      | Shared files                                                                                       |
| --- | -------------------------------------------------------- | ----------- | -------------------------------------------------------------------------------------------------- |
| 01  | [The core's types, kinds, texts and setting](./01-core-types.md) | 🔨 in progress | `crates/core/src/{lib.rs,error.rs,texts.rs}`, `crates/core/src/settings/core_schema.rs`, `crates/core/locales/en.json` |
| 02  | [The scanner's items query](./02-scanner-items.md)       | 🔨 in progress | `crates/core/src/disk_state.rs`, `crates/scanner/src/state/mod.rs`, `crates/testing/src/fixed_disk_state.rs`, `crates/engine/src/disk_tests.rs` |
| 03  | [The engine's queries](./03-engine-queries.md)           | 🔨 in progress | `crates/engine/src/{lib.rs,results.rs}`                                                            |

In order: 01, then 02, then 03.

## Conclusions

Built over three steps: the core's types and texts, the scanner's items query, the engine's two queries with one
judgement. What changed from the design is in the steps: the digit-led text segment, the `MAX_ITEMS + 1` cap, and the
judge that asks every folder above a row (`judge_reached`). Two reviews; the second round's findings were safety
findings (a package or a claimed folder could be opened) and are fixed with tests by id.
