# 030/01 — The module

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules, testing, app, docs

`modules/app-caches` (`fresher-module-app-caches`): its descriptor, its tables of places and exclusions, one category
and its texts, the owner and the decision, its fixtures and golden (its registration in every build is made in 02/01,
with the case files the app's test needs). The design:
[`../00-design.md`](../00-design.md), sections 1 to 6.

## Steps

| #   | Step                                                                   | Status      | Shared files                                                                                   |
| --- | ---------------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------- |
| 01  | [The crate, the tables and the texts](./01-crate.md)                   | 🔨 in progress | `Cargo.lock`, `docs/categories/README.md`, `docs/dependencies.md`                              |
| 02  | [The owner and the decision](./02-decision.md)                         | 🔨 in progress | `modules/app-caches/` only                                                                     |
| 03  | [Fixtures, golden and the contract](./03-fixtures-and-registration.md) | 🔨 in progress | `Cargo.lock`                                                                                   |

In order: 01, then 02, then 03.

## Conclusions

Built: the crate, the tables, the owner and the decision, the fixtures and the contract; the registration moved to plan 02 (F1). See the steps' conclusions and [the global readme](../readme.md#conclusions).
