# 019/02 — The module

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** modules, fixtures, app, docs

`modules/app-leftovers` (`fresher-module-app-leftovers`) in the group Apps: the table of ten leftover forms in nine
places, the exclusion table, three categories (Safe, Medium, High), the day setting, two choices, and the decision of
the design's section 3 with nine left-out reasons. The design: [`../00-design.md`](../00-design.md), sections 2 to 5.

## Steps

| #   | Step                                                                | Status      | Shared files                                                                                  |
| --- | ------------------------------------------------------------------- | ----------- | --------------------------------------------------------------------------------------------- |
| 01  | [The crate, the tables and the texts](./01-crate.md)                | ✅ approved | root `Cargo.toml`, `Cargo.lock`, `docs/categories/README.md`, `docs/dependencies.md`           |
| 02  | [The decision](./02-decision.md)                                    | ✅ approved | `modules/app-leftovers/` only                                                                 |
| 03  | [Fixture tests and registration](./03-fixtures-and-registration.md) | ✅ approved | `app/tauri/Cargo.toml`, `app/tauri/src/compose.rs`                                            |

01 after 01/01 (`only_bundle_ids`) and 01/04 (the group and the choices); 02 after 01, 01/02 and 01/03; 03 after 02.
01 creates `src/lib.rs`, `locales/en.json` and `tests/contract.rs`; 02 extends `lib.rs` (`on_match`, `finish`); 03
extends `tests/contract.rs` (`assert_texts_used`, `check_removals_matched`) and creates `tests/fixtures.rs`.

## Conclusions

Built as 02/01 to 02/03 say: the crate with its tables, categories and texts (nine interests, not eight), the decision
(ten rows and three item rows, Spotlight last, any doubt keeps the id), the fixtures, the golden and the registration.
The additions the steps name: a seventh fixture home (`proof-absent`), Spotlight asked for every dot-prefix of the id, a
wider exclusion check. The day setting is copied from `jetbrains` (deferred: a shared one).
