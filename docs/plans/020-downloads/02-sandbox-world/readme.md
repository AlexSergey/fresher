# 020/02 — The sandbox's world

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 ·
**Modules:** sandbox, testing, core, app, app/ui, docs

Sandbox mode answers every permission `NotAsked` and never opens the sandbox's own `Downloads`, so no case could show
the module at all; and a sandbox's installers would depend on the Mac's receipts and version. A part of the deferred
item [the sandbox world](../../../deferred/sandbox-world.md): a case states the access of the guarded folders, the
receipts, the macOS version and an app's change time; sandbox mode and the autotest take them from `world.toml`,
never from the Mac. The design: [`../00-design.md`](../00-design.md), section 2 (decision 9, agent's calls 6, 13
and 14).

## Steps

| #   | Step                                                              | Status      | Shared files                                                                                                  |
| --- | ----------------------------------------------------------------- | ----------- | ------------------------------------------------------------------------------------------------------------- |
| 01  | [Access in the world and the banner](./01-access.md)              | ✔️ done     | `crates/sandbox/src/{case,world,expected}.rs`, `crates/core/src/app_info.rs`, `app/tauri/src/{sandbox,adapters,commands}.rs`, `app/tauri/tests/sandboxes/harness.rs`, `app/ui/src/product/build-banner/`, `app/ui/e2e/{core-mock,visual.spec}.ts`, `app/ui/e2e/screenshots/`, `app/ui/src/locales/en.json`, `app/ui/src/ipc/bindings.generated.ts`, `docs/testing.md`, `docs/hacks/README.md`, `docs/architecture/permissions.md`, `docs/deferred/` |
| 02  | [Installs and an app's change time in the world](./02-installs.md) | ✔️ done     | `crates/sandbox/src/{case,world,expected}.rs`, `crates/testing/src/{stated_apps,tree_description,app_bundle,lib}.rs`, `app/tauri/src/{sandbox,compose}.rs`, `app/tauri/tests/sandboxes/harness.rs`, `docs/testing.md`, `docs/deferred/` |

01 after 01/04 (the engine learns the tried answers); 02 after 01/03 and 01 (the same files). Neither changes a case
file: the downloads case files are 04/01's.

## Conclusions

Not written yet.
