# 016/02 — The module

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
modules, app, docs

`modules/jetbrains` (`fresher-module-jetbrains`): its products and names, its decision, its categories with their
catalogue files, the data of an IDE that is no longer installed, its fixture tests and golden report, and its
registration. The design:
[`../00-design.md`](../00-design.md), sections 1 to 6 and 8; the pattern:
[`architecture/modules.md`](../../../architecture/modules.md), whose checklist every step ticks.

## Steps

| #   | Step                                                          | Status                                  | Shared files                                                             |
| --- | ------------------------------------------------------------- | --------------------------------------- | ------------------------------------------------------------------------ |
| 01  | [The crate, products, names and texts](./01-crate.md)         | ✔️ done | root `Cargo.toml`, `Cargo.lock`, `docs/categories/README.md`             |
| 02  | [The decision](./02-decision.md)                              | ✔️ done | —                                                                        |
| 03  | [Fixture tests and registration](./03-fixtures-and-registration.md) | ✔️ done | `app/tauri/Cargo.toml`, `app/tauri/src/compose.rs`                   |
| 04  | [The data of an IDE that is no longer installed](./04-uninstalled.md) | ✅ approved by the user (2026-11-17) | `crates/core/src/system/`, `crates/system/src/{apps,clock}.rs`, `crates/sandbox/src/{world,generate}.rs`, `crates/testing/src/{system_fakes,tree_description}.rs`, `app/tauri/src/sandbox.rs`, `app/tauri/tests/sandboxes/harness.rs`, `docs/categories/README.md` |

In order; 04, added on 2026-11-17 by the user's decision 10, is built after 02 and before 03, so 03's fixtures cover
its category.

## Conclusions

The module is built, registered in every build and tested at unit, fixture and contract level: 88+ tests, 98.0 %
of lines. Changed from the design: decision in `on_match` (call 12); the data of a removed IDE (decision 10, step 04,
added during the build); eight left-out reasons instead of six (eleven after the final review). Every review ended
approved.
