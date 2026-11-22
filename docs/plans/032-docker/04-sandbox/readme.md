# 032/04 — The sandbox

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, testing, core, app, app/ui

The sandbox learns what Docker needs: answers of tools (the tools part of
[`sandbox-world.md`](../../../deferred/sandbox-world.md)), with sandbox mode running no program but `/usr/bin/open`
and the autotest failing on any call a case file does not state; other apps' data in the world's access, so the disk
files are read; written bytes at the head of a sparse file; insight expectations. Then the four case files, their
goldens, and the module's registration in the app. The design: [`../00-design.md`](../00-design.md), section 8.

## Steps

| #   | Step                                                                   | Status      | Shared files                                                                                       |
| --- | ---------------------------------------------------------------------- | ----------- | -------------------------------------------------------------------------------------------------- |
| 01  | [Tool answers](./01-tool-world.md)                                     | 🔨 awaiting the user's walk | `crates/sandbox/src/{case,world,error}.rs`, `crates/core/src/app_info.rs`, `app/tauri/src/sandbox{.rs,/tests.rs}`, `app/tauri/tests/sandboxes/harness.rs`, `app/ui/src/locales/en.json`, `bindings.generated.ts` |
| 02  | [Other apps' data, written bytes, insight expectations](./02-access-and-expectations.md) | 🔨 awaiting the user's walk | `crates/sandbox/src/{case,world,expected,error}.rs`, `crates/testing/src/tree_description.rs`, `app/tauri/tests/sandboxes{.rs,/harness.rs}` |
| 03  | [The case files and the registration](./03-case-files.md)              | 🔨 awaiting the user's walk | `app/tauri/{Cargo.toml,src/compose.rs,src/commands.rs}`, `app/tauri/tests/sandboxes.rs`, `docs/dependencies.md` |

In order: 01, then 02, then 03 (03 also after 02/03).

## Conclusions

Not written yet.
