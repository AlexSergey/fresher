# 012/03 — Places and the query

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** engine

The by-place view with its auto-descent and explained shares, and the engine's two queries that give the UI the map in
its three phases. The design: [`../00-design.md`](../00-design.md), sections 4, 6 and 7.

## Steps

| #   | Step                                         | Status                                   | Shared files                                |
| --- | -------------------------------------------- | ---------------------------------------- | ------------------------------------------- |
| 01  | [The place view](./01-place-view.md)         | ✔️ done | `crates/engine/src/storage/mod.rs`          |
| 02  | [The map's query](./02-map-query.md)         | ✔️ done | `crates/engine/src/{engine,lib,results}.rs` |

In order, after plans 01 and 02.

## Conclusions

Built on `feat/012-storage-map` on 2026-11-17, task by task, each commit through the hooks and reviewed.

- **What is there now**: the place view with its descent (90 %, 8 levels) and the map's three queries.
- **Changed from the design while building**: the queries answer from one snapshot of the current phase, with the
  committed volumes kept beside the committed state; places are refused while a scan runs; the volumes are read before
  `begin()` and stored with it; volumes are read through `CleanupPorts` so plan 007's `compose.rs` stays untouched.
  `Descent` (`through` or `stay`) came with 04/05.
- **Measured**: the budget on the generated LARGE tree, release: 3.9 ms and 625 226 heap bytes for `Shares::of`
  (50 ms and 5 MB allowed). The developer's home is measured in 06/02.
