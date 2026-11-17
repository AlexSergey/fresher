# 016/03 — End to end

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
fixtures, testing, app, app/ui

The module through every layer, as the user asked: the sandbox case file (the autotest and the run by hand), the core
e2e flows and the UI e2e scenarios. The design: [`../00-design.md`](../00-design.md), section 8.

## Steps

| #   | Step                                         | Status                                  | Shared files                                                                         |
| --- | -------------------------------------------- | --------------------------------------- | ------------------------------------------------------------------------------------ |
| 01  | [The sandbox case file](./01-case-file.md)   | ✔️ done | `app/tauri/tests/sandboxes.rs`                                                       |
| 02  | [The core e2e flows](./02-core-e2e.md)       | ✔️ done | `crates/e2e/Cargo.toml`                                                              |
| 03  | [The UI e2e scenarios](./03-ui-e2e.md)       | ✔️ done | `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`                  |

In parallel: 01 after plan 01 and 02/03; 02 after 02/03; 03 after 02/01.

## Conclusions

The case files, the core e2e and the UI e2e are built and green: 33 sandbox tests, 10 core e2e tests, 295 Playwright
tests (both browsers). What each could not express is written in its step.
