# 023/01 — The module

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules, testing, app, docs

`modules/local-ai-models` (`fresher-module-local-ai-models`): its descriptor, its table of stores, three categories
and their texts, the names and the decision, its fixtures and golden, and its registration in every build. The design:
[`../00-design.md`](../00-design.md), sections 1 to 6.

## Steps

| #   | Step                                                                   | Status      | Shared files                                                                                   |
| --- | ---------------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------- |
| 01  | [The crate, the table and the texts](./01-crate.md)                    | 🔨 in progress | `Cargo.lock`, `docs/categories/README.md`, `docs/dependencies.md`                              |
| 02  | [The names and the decision](./02-decision.md)                         | 🔨 in progress | `modules/local-ai-models/` only                                                                |
| 03  | [Fixtures, golden and registration](./03-fixtures-and-registration.md) | 🔨 in progress | `app/tauri/Cargo.toml`, `app/tauri/src/{compose,commands}.rs` (the module lists), `Cargo.lock` |

In order: 01, then 02, then 03.

## Conclusions

The crate, its table and texts, the names and the decision (nine rows, each a unit test), the fixtures, the golden and
the registration are built as written; the two deviations (the verdict's `&'static str` category, the LM Studio
identity with the publisher) are in the steps and in the [global conclusions](../readme.md#conclusions).

