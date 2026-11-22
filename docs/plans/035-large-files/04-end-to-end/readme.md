# 035/04 — End to end

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** e2e, sandbox, testing, app, app/ui, fixtures, docs

The screen through the whole app: the core e2e flows with the real scanner and the real cleaner in a sandbox, the
items query's budget on the `LARGE` tree, the sandbox case file with its new `[case.largest]` expectation and its
golden (rule 8), and the UI's e2e scenarios, baselines and accessibility. The design:
[`../00-design.md`](../00-design.md), section 9.

## Steps

| #   | Step                                          | Status      | Shared files                                                                                       |
| --- | --------------------------------------------- | ----------- | -------------------------------------------------------------------------------------------------- |
| 01  | [The core e2e and the budget](./01-core-e2e.md) | 🔨 in progress | `crates/e2e/Cargo.toml`, `crates/engine/src/bench.rs`, `docs/testing.md`                       |
| 02  | [The sandbox case file](./02-sandbox.md)      | 🔨 in progress | `crates/sandbox/src/{case.rs,expected.rs,lib.rs}`, `app/tauri/tests/{sandboxes.rs,sandboxes/harness.rs}`, `docs/testing.md` |
| 03  | [The UI e2e](./03-ui-e2e.md)                  | 🔨 in progress | `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`    |

01 and 02 after 02/01, in parallel with plan 03; 03 after 03/02. 01 and 02 both add lines to `docs/testing.md`: the
second to finish rebases.

## Conclusions

The core e2e (17 flows and the items budget), the sandbox case file (`large-files.toml`, `[case.largest]`) and the UI
e2e are built. The sandbox work fixed a pre-existing bug: a stated access was compared with paths without expanding
`~`, so a denied folder was still scanned; the `downloads-denied` golden gained one remainder reason. After the rebase
onto plan 038 the harness of the demo test (`app/tauri/tests/demos.rs`) also includes the Largest check.
