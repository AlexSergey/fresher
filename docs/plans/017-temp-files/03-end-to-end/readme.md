# 017/03 — End to end

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
fixtures, e2e, app, app/ui

The module through every layer: its sandbox case file with stated facts and left-out expectations, the core flows on
the real scanner and cleaner, and the interface's scenarios. The design: [`../00-design.md`](../00-design.md),
section 9.

## Steps

| #   | Step                                         | Status                                  | Shared files                                                                    |
| --- | -------------------------------------------- | --------------------------------------- | ------------------------------------------------------------------------------- |
| 01  | [The sandbox case file](./01-case-file.md)   | ✔️ done                                | `app/tauri/tests/sandboxes.rs`                                                  |
| 02  | [The core e2e flows](./02-core-e2e.md)       | ✔️ done                                | `crates/e2e/Cargo.toml`                                                         |
| 03  | [The UI e2e scenarios](./03-ui-e2e.md)       | ✔️ done                                | `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, screenshots |

01 after 02/03 and 01/05; 02 after 02/03 and 01/04, in parallel with 01; 03 after 02/01, in parallel with both.

## Conclusions

The case file, the core e2e and the UI e2e are built and green: 37 sandbox autotests (26 cases in `temp.toml`), 13 core
e2e tests of the temp flows (150 runs in a row) among 67 in the crate, 320 of 321 Playwright tests (the one failure a
flake that was there before). What each could not express is written in its step.
