# 016/01 — The sandbox world: apps

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
sandbox, testing, app, app/ui

The part of the deferred [sandbox world](../../../deferred/sandbox-world.md) this module needs: a case declares
installed apps (written as real bundles) and running apps, and the left-out lines it expects. The design:
[`../00-design.md`](../00-design.md), section 7.

## Steps

| #   | Step                                                | Status                                  | Shared files                                                                                  |
| --- | --------------------------------------------------- | --------------------------------------- | --------------------------------------------------------------------------------------------- |
| 01  | [Installed apps as bundles](./01-installed-apps.md) | ✔️ done | `crates/testing/src/{tree_description,fixture_tree}.rs`, `crates/sandbox/src/{case,generate,expected}.rs` |
| 02  | [Running apps](./02-running-apps.md)                | ✔️ done | `crates/sandbox/src/`, `app/tauri/src/sandbox.rs`, `app/tauri/tests/sandboxes/harness.rs`, `app/ui/src/product/build-banner/` |
| 03  | [The left-out expectation](./03-left-out.md)        | ✔️ done | `crates/sandbox/src/{case,expected}.rs`, `app/tauri/tests/sandboxes/harness.rs`, `docs/testing.md` |

In order.

## Conclusions

The sandbox now declares installed and running apps and the left-outs a module must report; sandbox mode never takes
the running apps of the real Mac. Tests: `fresher-testing`, `fresher-sandbox`, `fresher-app` and `fresher-engine` pass
(about 580 tests in the three crates at the end of 01/03); `npm run sandbox:test` unchanged goldens. Every
review was clean; the small leftovers they parked (control characters in app names, a case file that targets an
`Info.plist`, `world.toml` size cap, a misplaced doc) are in the list at the end of the plan's conclusions.
