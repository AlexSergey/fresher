# 037/03 — End to end

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, testing, e2e, app, app/ui

The Trash numbers proven through the app's own sandbox composition (every case file's golden and two invariants, and
two case files of their own), through the real cleaner and journal in the core e2e, and through the UI's Playwright
flows. The design: [`../00-design.md`](../00-design.md), sections 7 and 8.

## Steps

| #   | Step                                                          | Status      | Shared files                                                                                          |
| --- | ------------------------------------------------------------- | ----------- | ----------------------------------------------------------------------------------------------------- |
| 01  | [The sandbox: invariants and case files](./01-sandbox.md)     | 🔨 in progress | `app/tauri/tests/sandboxes{.rs,/harness.rs}`, every `fixtures/golden/sandbox-*.json`, `docs/testing.md` |
| 02  | [The core e2e](./02-core-e2e.md)                              | 🔨 in progress | `crates/e2e/Cargo.toml` (if a test entry is needed)                                                   |
| 03  | [The UI e2e](./03-ui-e2e.md)                                  | 🔨 in progress | `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`        |

01 and 02 run in parallel after plan 01; 03 after plan 02.

## Conclusions

All three steps are built: the sandbox's invariants and two case files (76 sandbox tests), the core e2e `trash.rs`
(11 flows, `fresher-e2e` 171 passed, 2 skipped) and the UI e2e (11 new tests, the full suite 624 passed, 1 failed:
the known chromium baseline `category-details-app-leftovers-chosen`, see 04/01). Mutations by hand fail the sandbox
and the core e2e as written in the steps. Awaiting the user's walk.
