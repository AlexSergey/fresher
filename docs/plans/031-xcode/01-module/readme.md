# 031/01 — The module

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** modules, testing, docs

`modules/xcode` (`fresher-module-xcode`): its descriptor, its table of places, four categories and their texts, the
names, the decision, its fixtures and golden. Its registration in the app is made in 02/01, with the case files the
app's sandbox test needs. The design: [`../00-design.md`](../00-design.md), sections 1 to 6.

## Steps

| #   | Step                                                         | Status      | Shared files                                                         |
| --- | ------------------------------------------------------------ | ----------- | -------------------------------------------------------------------- |
| 01  | [The crate, the table and the texts](./01-crate.md)          | 🔨 awaiting the walk | `Cargo.lock`, `docs/categories/README.md`, `docs/dependencies.md`    |
| 02  | [The names and the decision](./02-decision.md)               | 🔨 awaiting the walk | `modules/xcode/` only                                                |
| 03  | [Fixtures, golden and the contract](./03-fixtures.md)        | 🔨 awaiting the walk | `Cargo.lock`, `docs/dependencies.md`                                 |

In order: 01, then 02, then 03.

## Conclusions

The module is built: 85 tests, 99.29 % of its lines, the contract test passes with `Catalogue::Required`. Details in the
three steps.
