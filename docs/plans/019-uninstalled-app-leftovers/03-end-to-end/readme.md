# 019/03 — End to end

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 ·
**Modules:** sandbox, fixtures, e2e, app, app/ui, docs

The module through every layer: the sandbox's Spotlight answers and the two case files, the shared core e2e home
harness and the module's flows on the real scanner and cleaner, and the interface's choices and scenarios. The design:
[`../00-design.md`](../00-design.md), section 6.

## Steps

| #   | Step                                                                     | Status      | Shared files                                                                                               |
| --- | ------------------------------------------------------------------------ | ----------- | ---------------------------------------------------------------------------------------------------------- |
| 01  | [Spotlight in the sandbox and the case files](./01-case-files.md)        | ✅ approved | `crates/sandbox/src/{case,world,expected}.rs`, `app/tauri/src/sandbox.rs`, `app/tauri/tests/{sandboxes.rs,sandboxes/harness.rs}`, `docs/testing.md`, `docs/deferred/sandbox-world.md` |
| 02  | [The shared home harness and the core e2e flows](./02-core-e2e.md)       | ✅ approved | `crates/e2e/Cargo.toml`, `crates/e2e/tests/{support/,temp_home/,dev_caches_home/}`, `docs/testing.md`, `docs/deferred/` |
| 03  | [The choices on the module page and the UI e2e](./03-ui.md)              | ✅ approved | `app/ui/src/{state/selection.ts,screens/module/module-screen.tsx}`, `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, screenshots |

01 after 02/03; 02 after 02/03, in parallel with 01; 03 after 01/04 and 02/01, in parallel with 02/02 and the others.
01 and 02 both add lines to `docs/testing.md`, each in its own section (the case files' vocabulary; the e2e list).

## Conclusions

Built as 03/01 to 03/03 say: Spotlight's answers in the sandbox world and two case files with their goldens, one shared
home harness and eleven core flows (rescans asserted), and the choices on the module page with their UI scenarios. The
changes from the brief: the `DeclaredSearch` is built in `app/tauri`, the banner names a simulated Spotlight, flow 5 runs
12 days on (the rescan limit is 14).
