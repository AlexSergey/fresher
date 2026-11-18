# 018/02 — The module

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** modules, testing, app, docs

`modules/dev-caches` (`fresher-module-dev-caches`) in the group Development: the table of ten places and their signs,
nine categories (seven Low, two Safe), the decision of the design's section 3 with four left-out reasons, no setting
beyond its switch. The design: [`../00-design.md`](../00-design.md), sections 2 to 5.

## Steps

| #   | Step                                                                | Status      | Shared files                                                                                       |
| --- | ------------------------------------------------------------------- | ----------- | -------------------------------------------------------------------------------------------------- |
| 01  | [The crate, the table and the texts](./01-crate.md)                 | ✅ approved | root `Cargo.toml`, `Cargo.lock`, `docs/categories/README.md`, `docs/dependencies.md`                |
| 02  | [The decision](./02-decision.md)                                    | ✅ approved | `modules/dev-caches/` only                                                                         |
| 03  | [Fixture tests and registration](./03-fixtures-and-registration.md) | ✅ approved | `crates/testing/src/tree_description.rs`, `app/tauri/Cargo.toml`, `app/tauri/src/compose.rs`       |

01 after 01/01; 02 after 01; 03 after 02.

## Conclusions

Built as 02/01 to 02/03 say: the crate with its table, categories and texts, the decision and its left-outs, the
fixtures, the golden and the registration, with the additions each step names (`ToolsError::DuplicateSign`,
`InterestMap::holder_of`, the home `dev-caches-not-folders.toml`).
