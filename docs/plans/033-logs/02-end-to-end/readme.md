# 033/02 — End to end

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, fixtures, e2e, app, app/ui, docs

The module through the whole app: its case file and golden (rule 8) with the app's registration, the core e2e flows
with the real scanner and cleaner beside the other modules of `Library/Logs` (`jetbrains`, `app-leftovers`), and the
interface's path. The design: [`../00-design.md`](../00-design.md), section 9.

## Steps

| #   | Step                                                  | Status      | Shared files                                                                                                    |
| --- | ----------------------------------------------------- | ----------- | --------------------------------------------------------------------------------------------------------------- |
| 01  | [The case file and the registration](./01-case-file.md) | 🔨 in progress | `app/tauri/{Cargo.toml,src/compose.rs,src/commands.rs,tests/sandboxes.rs}`, `docs/testing.md`, `Cargo.lock`    |
| 02  | [The core e2e](./02-core-e2e.md)                      | 🔨 in progress | `crates/e2e/Cargo.toml`, `docs/testing.md`, `Cargo.lock`                                                        |
| 03  | [The UI](./03-ui.md)                                  | 🔨 in progress | `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`                 |

01 and 02 after 01/03; 03 after 01/01 (the texts). 01 and 02 both add a line to `docs/testing.md`: the second to
finish rebases.

## Conclusions

The case file and the registration, seven core flows and the UI scenarios are built and reviewed. Core e2e: 145 passed
(2 skipped); Playwright in both browsers: 552 passed. See [the global readme](../readme.md#conclusions).
