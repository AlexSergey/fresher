# 024/03 — End to end

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** sandbox, fixtures, e2e, app, app/ui, docs

The module through the whole app: its case files and their goldens (rule 8), the core e2e flows with the real scanner
and cleaner in a sandbox, and the interface's path. The design: [`../00-design.md`](../00-design.md), section 9.

## Steps

| #   | Step                                  | Status      | Shared files                                                                                       |
| --- | ------------------------------------- | ----------- | -------------------------------------------------------------------------------------------------- |
| 01  | [The case files](./01-case-files.md)  | 🔨 in progress | `app/tauri/tests/sandboxes.rs`, `docs/testing.md`                                                  |
| 02  | [The core e2e](./02-core-e2e.md)      | 🔨 in progress | `crates/e2e/Cargo.toml`, `docs/testing.md`                                                         |
| 03  | [The UI](./03-ui.md)                  | 🔨 in progress | `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`    |

01 and 02 after 02/04; 03 after 02/01 (the texts). 01 and 02 both add a line to `docs/testing.md`: the second to
finish rebases.

## Conclusions

Built: the four case files, the core e2e flows, the UI cases. See each step for the changes (the case files' access
grant, `all` leaving out clashing files, the e2e switch through `update_setting`, the renamed restored-scan test, the
UI on plan 027's screens, the mock's not-yours left-out). See the [global conclusions](../readme.md#conclusions).
