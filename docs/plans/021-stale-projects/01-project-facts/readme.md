# 021/01 — Projects with their facts

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 ·
**Modules:** core, scanner, engine, testing, e2e, app, docs

What the code lacks before a module can tell an old project from an active one and size its artifact folders: a
question to the walked tree before the commit, the projects' artifacts and Rust caches with their sizes, a precise
last activity (a deferred item), and the same projects in the module harness as in the engine. The design:
[`../00-design.md`](../00-design.md), sections 1 and 2.

## Steps

| #   | Step                                                                 | Status      | Shared files                                                                                                  |
| --- | -------------------------------------------------------------------- | ----------- | ------------------------------------------------------------------------------------------------------------- |
| 01  | [What the walk saw below a folder](./01-subtrees.md)                 | ✔️ done     | `crates/core/src/{walk,lib}.rs`, `crates/scanner/src/{scanner,lib}.rs`, `crates/scanner/{tests/reference.rs,benches/scan.rs}`, `crates/testing/src/naive_walk.rs`, `crates/engine/src/testing.rs`, `crates/e2e/tests/demo.rs`, `app/tauri/src/testing.rs`, `docs/architecture/README.md` |
| 02  | [The project finder in the core, used by the engine](./02-finder.md) | ✔️ done     | `crates/core/src/system/{mod,projects,facts}.rs`, `crates/core/src/lib.rs`, `crates/engine/src/{session,projects,lib,testing}.rs`, `crates/e2e/tests/projects.rs` |
| 03  | [The harness finds the projects; the docs](./03-harness.md)          | ✔️ done | `crates/testing/src/run_module.rs`, `docs/architecture/{modules,README}.md`, `docs/deferred/`, `docs/plans/{009-system-knowledge,021-stale-projects}/` |

In order: 02 needs 01's `DiskWalk::subtrees`; 03 needs 02's `ProjectFinder`.

## Conclusions

Not written yet.
