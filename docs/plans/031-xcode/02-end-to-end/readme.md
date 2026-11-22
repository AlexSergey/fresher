# 031/02 — End to end

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, e2e, app, app/ui, docs

The module in the app and around it: its case files with their goldens and the app's registration, the core e2e flows
beside `dev-caches`, and the interface's path. The design: [`../00-design.md`](../00-design.md), section 8.

## Steps

| #   | Step                                               | Status      | Shared files                                                                                                       |
| --- | -------------------------------------------------- | ----------- | ------------------------------------------------------------------------------------------------------------------ |
| 01  | [The case files and the registration](./01-case-files.md) | 🔨 awaiting the walk | `app/tauri/Cargo.toml`, `app/tauri/src/{compose,commands}.rs`, `app/tauri/tests/sandboxes.rs`, `docs/testing.md`, `Cargo.lock` |
| 02  | [The core e2e flows](./02-core-e2e.md)             | 🔨 awaiting the walk | `crates/e2e/Cargo.toml`, `docs/testing.md`, `Cargo.lock`                                                          |
| 03  | [The UI](./03-ui.md)                               | 🔨 awaiting the walk | `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`, `docs/testing.md` |

01 and 02 after plan 01, in parallel (each its own files; `docs/testing.md` gets one paragraph each, kept apart). 03
after 01/01 only (it reads the texts), in parallel with the rest.

## Conclusions

The module is in the app and around it: both case files, 9 core flows, the UI path. See the three steps.
