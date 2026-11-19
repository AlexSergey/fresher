# 020/04 — End to end

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 ·
**Modules:** sandbox, fixtures, e2e, app, app/ui, docs

The module through every layer: its three case files in the sandbox (rule 8), the core e2e flows on the real scanner,
engine, cleaner and journal, and the UI's scenarios on the mocked core. The design:
[`../00-design.md`](../00-design.md), section 4.

## Steps

| #   | Step                               | Status      | Shared files                                                                                          |
| --- | ---------------------------------- | ----------- | ----------------------------------------------------------------------------------------------------- |
| 01  | [The case files](./01-case-files.md) | ✔️ done | `app/tauri/tests/sandboxes.rs`, `docs/testing.md`, `docs/known-issues/`                              |
| 02  | [The core e2e](./02-core-e2e.md)   | ✔️ done | `crates/e2e/Cargo.toml`, `crates/e2e/tests/support/`                                                  |
| 03  | [The UI](./03-ui.md)               | ✔️ done | `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`       |

All three in parallel after plan 03 (01 also after plan 02).

## Conclusions

Not written yet.
