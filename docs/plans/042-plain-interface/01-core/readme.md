# 042/01 — The core

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 ·
**Modules:** core, engine, testing, sandbox, e2e, modules, app/tauri, app/ui (mocks and bindings only), docs

Everything the new screens read comes from the core first: the High switch and its refusals, the app that sets it,
the disk's five parts, a category's icon and one line, the group texts and the scan's estimate. The design:
[`../00-design.md`](../00-design.md), sections 1 to 5.

## Steps

| #   | Step                                                          | Status      | Shared files |
| --- | ------------------------------------------------------------- | ----------- | ------------ |
| 01  | [The High switch in the engine](./01-high-switch.md)          | 🔨 awaiting the user's walk | `crates/core/src/{danger.rs,error.rs}`, `crates/core/locales/en.json`, `crates/engine/src/`, `crates/e2e/tests/`, `app/tauri/src/{compose.rs,testing.rs}`, `docs/restrictions/deletion-safety.md` |
| 02  | [The app withholds High](./02-app-withholds.md)               | 🔨 awaiting the user's walk | `crates/core/src/app_info.rs`, `app/tauri/src/{compose.rs,sandbox.rs,commands.rs}`, `app/tauri/tests/sandboxes/harness.rs`, `crates/sandbox/src/{case.rs,expected.rs}`, `fixtures/sandboxes/`, the UI's mocks |
| 03  | [The disk's five parts](./03-five-parts.md)                   | 🔨 awaiting the user's walk | `crates/core/src/storage/`, `crates/core/locales/en.json`, `crates/core/src/texts.rs`, the UI's mocks |
| 04  | [A category's icon and one line](./04-icon-and-line.md)       | 🔨 awaiting the user's walk | `crates/core/src/category.rs`, `crates/testing/src/contract.rs`, every module's `src/lib.rs` and `locales/en.json`, `docs/architecture/modules.md` |
| 05  | [The group texts](./05-group-texts.md)                        | 🔨 awaiting the user's walk | `crates/core/locales/en.json`, the UI's fixtures |
| 06  | [The scan's estimate](./06-scan-estimate.md)                  | 🔨 awaiting the user's walk | `crates/core/src/event.rs`, `crates/engine/src/session.rs`, the UI's mocks |

01, then 02, 03, 04, 05, 06, one at a time (they share the core's dictionary, `texts.rs` and the bindings).

## Conclusions

- The High switch and its refusals in the engine, set by the app in every build; the five parts of the disk; an icon and
  a line on every category; the group texts; the scan's estimate. Everything the new screens read comes from the core.
- Beyond the design: `FindingsAdded` filtered too; insight parts, app page items and Apps rows of a High kind left out
  (the final review and 09/01).
