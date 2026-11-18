# 017/02 — The module

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
modules, core, app, app/ui, docs

`modules/temp` (`fresher-module-temp`) in the new group System: one Safe category, the decision of the design's
section 7, eight left-out reasons, one setting. The design: [`../00-design.md`](../00-design.md), sections 7 and 8.

## Steps

| #   | Step                                                                  | Status                                  | Shared files                                                                                                     |
| --- | --------------------------------------------------------------------- | --------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| 01  | [The crate, the group System and the texts](./01-crate.md)            | ✔️ done                                | root `Cargo.toml`, `Cargo.lock`, `crates/core/src/group.rs`, `crates/core/locales/en.json`, the bindings, `app/ui/src/locales/en.json`, `docs/categories/README.md` |
| 02  | [The decision and the setting](./02-decision.md)                      | ✔️ done                                | `modules/temp/` only                                                                                             |
| 03  | [Fixture tests and registration](./03-fixtures-and-registration.md)   | ✔️ done                                | `app/tauri/Cargo.toml`, `app/tauri/src/compose.rs`                                                               |

01 after 01/01; 02 after 01 and 01/03; 03 after 02 and 01/05.

## Conclusions

`modules/temp` is built and registered in every build: the group System, one Safe category, the ten rows of the decision
with the review's rulings (an app match with one leading dot stripped, on bytes; a folder holding a lock-named entry
kept as a lock), eight left-out reasons each used by a fixture, the setting from 7 to 30 days, five fixture homes and
the contract test. `modules/temp` covers 100 % of its lines.
