# 021/03 — End to end

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** sandbox, fixtures, e2e, app,
app/ui, docs

The module through every layer: its case file in the sandbox (rule 8), the core e2e flows on the real scanner,
engine, cleaner and journal, and the UI's scenarios on the mocked core. The design:
[`../00-design.md`](../00-design.md), section 4.

## Steps

| #   | Step                                 | Status      | Shared files                                                                                          |
| --- | ------------------------------------ | ----------- | ----------------------------------------------------------------------------------------------------- |
| 01  | [The case file](./01-case-file.md)   | ✔️ done     | `app/tauri/tests/sandboxes.rs`, `docs/testing.md`                                                     |
| 02  | [The core e2e](./02-core-e2e.md)     | ✔️ done     | `crates/e2e/Cargo.toml`, `docs/testing.md`                                                            |
| 03  | [The UI](./03-ui.md)                 | ✔️ done     | `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`       |

All three in parallel after plan 02. 01 and 02 each add their own paragraph to `docs/testing.md`: the second merge
keeps both.

## Conclusions

Not written yet.
