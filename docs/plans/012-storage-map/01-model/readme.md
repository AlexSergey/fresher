# 012/01 — The model

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
scanner, testing, engine

The map's types, the tree's owned nodes, the shares that put every owned byte in one place, and the by-meaning groups
with the attribution to apps. The design: [`../00-design.md`](../00-design.md), sections 1, 4 and 5.

## Steps

| #   | Step                                                  | Status                                   | Shared files                                                                                    |
| --- | ----------------------------------------------------- | ---------------------------------------- | ----------------------------------------------------------------------------------------------- |
| 01  | [The map's types](./01-types.md)                      | ✔️ done | `crates/core/src/lib.rs`, `crates/core/locales/en.json`, `crates/core/tests/typescript.rs`      |
| 02  | [Owned nodes and lookup](./02-owned-nodes.md)         | ✔️ done | `crates/core/src/disk_state.rs`, `crates/scanner/src/state/mod.rs`, `crates/testing/src/fixed_disk_state.rs` |
| 03  | [The shares](./03-shares.md)                          | ✔️ done | `crates/engine/src/lib.rs`                                                                      |
| 04  | [By meaning and the apps](./04-meaning.md)            | ✔️ done | `crates/core/src/finding.rs`, `crates/core/src/system/{ids,running}.rs`, `crates/engine/src/{results,session}.rs` |

In order. 01 may run beside 02/02 and 04/03; 02 beside 02/01; 03 beside 02/03.

## Conclusions

Built on `feat/012-storage-map` on 2026-11-17, task by task, each commit through the hooks and reviewed.

- **What is there now**: the map's types and texts in the core, `DiskState::owned` and `find`, `Shares::of` (one place
  for every owned byte) and the by-meaning groups with the attribution to apps.
- **Changed from the design while building**: `explained(state, id)` takes the state, and a folder inside a counted
  owner is wholly explained (01/03); an ignored inner owner is subtracted from the outer one, its bytes go to your
  folders (01/03); the meaning is scoped to one volume's roots (03/02); an ignored finding that names an app stays in
  the app's data (final review); the largest folders are only folders.
- **Measured**: the properties hold: every owned byte is counted once, the leaves of the groups add up to the roots
  (3000 cases, mutation-checked). On the generated LARGE tree, release: `Shares::of` 3.9 ms, 625 226 heap bytes.
