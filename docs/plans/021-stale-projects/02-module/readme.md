# 021/02 — The module

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** modules, testing, app, docs

`modules/stale-projects` (`fresher-module-stale-projects`): its descriptor, its table, four categories, the decision
and the texts, its fixtures and golden, and its registration in every build. The design:
[`../00-design.md`](../00-design.md), section 3.

## Steps

| #   | Step                                                                   | Status      | Shared files                                                                                       |
| --- | ---------------------------------------------------------------------- | ----------- | -------------------------------------------------------------------------------------------------- |
| 01  | [The crate, the table and the texts](./01-crate.md)                    | ✔️ done | root `Cargo.toml`, `Cargo.lock`, `docs/categories/README.md`, `docs/dependencies.md`               |
| 02  | [The decision](./02-decision.md)                                       | ✔️ done | `modules/stale-projects/` only                                                                     |
| 03  | [Fixtures, golden and registration](./03-fixtures-and-registration.md) | ✔️ done | `app/tauri/Cargo.toml`, `app/tauri/src/{compose,commands}.rs` (the module lists), `Cargo.lock`     |

In order: 01 after 01/02 (`ProjectKind::caches`, `LastChange`); 02 after 01; 03 after 02 and 01/03 (`run_module`
finds the projects).

## Conclusions

Not written yet.
