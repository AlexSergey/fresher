# 038/01 — The demo world

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, testing, app, fixtures

A believable made-up Mac, written by the sandbox generator: the case file's `[demo]` header (its clock, its sizes,
its volume, the categories it must show) and the `scenery` case kind; a stated allocated size and a stated volume, so
gigabytes show as gigabytes and the ring shows the world's disk; the first world, `developer-mac`, and the test that
scans it. The design: [`../00-design.md`](../00-design.md), section 1.

## Steps

| #   | Step                                                            | Status      | Shared files                                                                  |
| --- | --------------------------------------------------------------- | ----------- | ----------------------------------------------------------------------------- |
| 01  | [The demo header and the scenery kind](./01-demo-header.md)     | 🔨 in progress | `crates/sandbox/src/{case,expected,generate,main}.rs`                         |
| 02  | [Stated sizes and the stated volume](./02-stated-sizes.md)      | 🔨 in progress | `crates/testing/src/facts_lens.rs`, `crates/sandbox/src/world.rs`, `app/tauri/src/sandbox.rs` |
| 03  | [The first world and its scan test](./03-first-world.md)        | 🔨 in progress | `fixtures/demos/`, `app/tauri/tests/`                                         |

In order: 01, then 02, then 03.

## Conclusions

The world is built: the `[demo]` header, scenery cases, stated allocated sizes and volume, `developer-mac` and its scan
test. See each step. Numbers: 58 files, about 144 GB declared, a 460 GiB stated disk; sandbox 242 tests, testing 499,
the app's `demos`, `sandboxes` and `bindings` tests 122 passing. No core change.
