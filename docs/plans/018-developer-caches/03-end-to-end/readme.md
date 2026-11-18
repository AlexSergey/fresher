# 018/03 — End to end

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** fixtures, e2e, app, app/ui, docs

The module through every layer: its three sandbox case files with their left-out expectations, the core flows on the
real scanner and cleaner (after the deferred move of the e2e homes' shared helpers), and the interface's scenarios.
The design: [`../00-design.md`](../00-design.md), section 6.

## Steps

| #   | Step                                                          | Status      | Shared files                                                                      |
| --- | ------------------------------------------------------------- | ----------- | --------------------------------------------------------------------------------- |
| 01  | [The sandbox case files](./01-case-files.md)                  | ✅ approved | `app/tauri/tests/sandboxes.rs`, `docs/testing.md`                                 |
| 02  | [The shared helpers and the core e2e flows](./02-core-e2e.md) | ✅ approved | `crates/e2e/Cargo.toml`, `crates/e2e/tests/{support/,jetbrains_home/,temp_home/,places.rs,cleaning/}`, `docs/deferred/` |
| 03  | [The UI e2e scenarios](./03-ui-e2e.md)                        | ✅ approved | `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, screenshots |

01 after 02/03; 02 after 02/03, in parallel with 01; 03 after 02/01, in parallel with 02/02 and both.

## Conclusions

Built as 03/01 to 03/03 say: the three case files, the shared e2e helpers and the eight core flows, the UI scenarios on
two mock scans. One known issue came out of it: the scanner does not count an unreadable folder inside a claimed match
as not read; one item is deferred: the module harnesses' shared code.
