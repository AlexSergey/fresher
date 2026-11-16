# 004/02 — Contracts

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** core, rules, testing, engine, app

The core's side of the plan, written once so that 03, 04 and 06 can run in parallel against fixed contracts: the
third size and the marker check, the OS-neutral ports with their fakes, the disk state and cache ports, and the new
shape of `DiskWalk`. Every crate that uses a changed type is adapted in the same step, so each commit builds and
passes the hooks. The design: [`../00-design.md`](../00-design.md), sections 1, 2, 4, 6 and 11.

Depends on 01 (the attribute set and the journal's answers shape the ports). The core is frozen when this plan ends.

## Steps

| #   | Step                                                              | Status   | Shared files                                                                          |
| --- | ----------------------------------------------------------------- | -------- | ------------------------------------------------------------------------------------- |
| 01  | [Sizes and the marker check](./01-sizes-and-marker.md)            | ✔️ done | `crates/rules/`, `crates/testing/src/naive_walk.rs`, bindings, `docs/hacks/README.md` |
| 02  | [The OS ports](./02-os-ports.md)                                  | ✔️ done | `crates/testing/src/fakes.rs`                                                         |
| 03  | [Disk state and the cache port](./03-disk-state-and-cache.md)     | ✔️ done | `crates/testing/src/fakes.rs`                                                         |
| 04  | [The walk request and commit](./04-walk-request-and-commit.md)    | ✔️ done | `crates/engine/`, `crates/testing/`, `app/tauri/src/adapters.rs`, bindings, `docs/architecture/README.md` |

In order.

## Conclusions

- The core's contracts for the rest of plan 004 are in place, each step one commit that builds and passes the hooks:
  the third size (`Sizes::reclaimable`) and `Match::marker` with the pure `CACHEDIR.TAG` check (the rules no longer
  read a file, and their hack entry is gone); the OS ports `FolderReader`, `ChangeJournal` and `Volumes`; the
  `DiskState` and `ScanCache` ports with `stable_hash` and the two new command errors; and the new `DiskWalk` with
  `WalkRequest`, `WalkMode`, `commit` and `ScanKind`. `Unreadable::Guarded` is the name shared with plan 006.
- Fakes in `fresher-testing`: `FakeVolumes`, `FakeChangeJournal`, `MemoryScanCache`, `FixedDiskState`; `NaiveWalk`
  checks the tag, honours the avoid list, refuses `Restore` and commits a `FixedDiskState`.
- Deviations, each recorded in its step: `ScanCache` takes `ScopeKey` by value (Clippy refuses an 8-byte `Copy` by
  reference); `FixedDiskState` has its own file; `crates/rules/src/cachedir.rs` is removed rather than emptied; the
  engine does not commit a walk that stopped with an error.
- The core is frozen from here: a contract missing in plans 03–08 goes into `crates/core` in the main session.
