# 036/04 — End to end

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, testing, e2e, app

The feature through the real scanner, the real cleaner and the app's own sandbox composition: the case files'
`[case.uninstall]` expectation and its autotest, the two case files and their goldens, and the core's end-to-end flows
of both choices, put back and every refusal. The design: [`../00-design.md`](../00-design.md), sections 9 and 13.

## Steps

| #   | Step                                                      | Status      | Shared files                                                                                             |
| --- | --------------------------------------------------------- | ----------- | -------------------------------------------------------------------------------------------------------- |
| 01  | [The sandbox's uninstall cases](./01-sandbox.md)          | 🔨 awaiting the walk | `crates/sandbox/src/{case,expected,world,generate}.rs`, `crates/testing/src/app_bundle.rs`, `app/tauri/tests/sandboxes.rs`, `app/tauri/tests/sandboxes/harness.rs`, `app/tauri/src/sandbox.rs` |
| 02  | [The core's end-to-end flows](./02-core-e2e.md)           | 🔨 awaiting the walk | `crates/e2e/Cargo.toml`                                                                                  |

01 and 02 in parallel: they share no file.

## Conclusions

Two sandbox case files with goldens and 17 core flows, mutation-checked against the second wall. The scan folds small Library files, so the case files use 1 MiB parts (deferred).
