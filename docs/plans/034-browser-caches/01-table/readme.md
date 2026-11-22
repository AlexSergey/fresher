# 034/01 — The table

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** modules, testing, docs

The module `app-caches` (plan 030) gains the table `apps.toml` and its two categories, a second way to find a
folder's owner (the row's bundle id), the report of a table row's finding, the one new text, and the fixture homes and
golden that prove it. The design: [`../00-design.md`](../00-design.md), sections 1 to 6.

## Steps

| #   | Step                                                                  | Status      | Shared files                    |
| --- | --------------------------------------------------------------------- | ----------- | ------------------------------- |
| 01  | [The table and the two categories](./01-table-and-categories.md)      | ✔️ done | `docs/categories/README.md`     |
| 02  | [The owner, the decision and the report](./02-decision-and-report.md) | ✔️ done | `modules/app-caches/src/` only  |
| 03  | [The text, the fixtures, the golden and the contract](./03-fixtures.md) | ✔️ done | `modules/app-caches/` only      |

In order: 01, then 02, then 03. Each leaves `cargo nextest run -p fresher-module-app-caches` green.

## Conclusions

The module reads the table, finds the owner of a row by its bundle id, decides with plan 030's eleven rows (rows 2 and
4 by the table) and reports one finding per cache folder; it is proven on fixture homes and a golden. 139 tests in the
module; every file of the crate at least 92 % of lines. Nothing outside `modules/app-caches` changed in Rust. The
changes from the design are in the three steps' conclusions; the calls are in
[`../00-design.md`](../00-design.md).
