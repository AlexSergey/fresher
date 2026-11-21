# 030/02 — End to end

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** sandbox, fixtures, e2e, app, app/ui, docs

The module through the whole app: its case files and their goldens (rule 8) with the checklist's negative sentence,
the core e2e flows with the real cleaner in a sandbox beside the other modules of `Library/Caches`, and the
interface's path. The design: [`../00-design.md`](../00-design.md), section 7.

## Steps

| #   | Step                                  | Status      | Shared files                                                                                       |
| --- | ------------------------------------- | ----------- | -------------------------------------------------------------------------------------------------- |
| 01  | [The case files](./01-case-files.md)  | 🔨 in progress | `crates/sandbox/src/expected.rs`, `app/tauri/tests/sandboxes.rs`, `docs/testing.md`               |
| 02  | [The core e2e](./02-core-e2e.md)      | 🔨 in progress | `crates/e2e/Cargo.toml`, `docs/testing.md`                                                         |
| 03  | [The UI](./03-ui.md)                  | 🔨 in progress | `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`    |

01 and 02 after 01/03; 03 after 01/01 (the texts). 01 and 02 both add a line to `docs/testing.md`: the second to
finish rebases.

## Conclusions

Built: the two case files, the core e2e flows and the UI scenarios with their baselines; the registration of the module in the app landed here. See the steps' conclusions and [the global readme](../readme.md#conclusions).
