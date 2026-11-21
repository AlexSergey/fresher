# 029/03 — End to end

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** sandbox, fixtures, e2e, app, app/ui, docs

The module and the together removal proven through the sandbox, the engine with the real cleaner, and the interface.
The design: [`../00-design.md`](../00-design.md), section 10.

## Steps

| #   | Step                                    | Status      | Shared files                                                                  |
| --- | --------------------------------------- | ----------- | ----------------------------------------------------------------------------- |
| 01  | [The case files](./01-case-files.md)    | 🔨 in progress | `fixtures/sandboxes/`, `app/tauri/tests/sandboxes.rs`, `docs/testing.md`      |
| 02  | [The core e2e](./02-core-e2e.md)        | 🔨 in progress | `crates/e2e/`, `docs/testing.md`                                              |
| 03  | [The UI](./03-ui.md)                    | 🔨 in progress | `app/ui/e2e/`, `app/ui/src/test/`                                             |

01, 02 and 03 in parallel; `docs/testing.md` is touched by 01 and 02 (the second rebases).

## Conclusions

All three steps are built and reviewed. The sandbox cases, the core flows and the UI scenarios prove the module and the
together removal end to end; the by-hand walk of the sandbox is the user's and is listed in
[04/02](../04-finish/02-run-and-conclusions.md). The e2e journal goldens of ten older flows were regenerated at
journal version 2 (only the version, `more`, `went` and `back` lines changed). Numbers:
[the global readme](../readme.md#conclusions).
