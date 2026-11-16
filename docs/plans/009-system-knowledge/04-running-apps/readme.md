# 009/04 — Running apps

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** macos, system, cleaner, engine, app

The running apps by bundle id, and the cleaner skipping an item that belongs to one. The design:
[`../00-design.md`](../00-design.md), section 4.

## Steps

| #   | Step                                                  | Status      | Shared files                          |
| --- | ----------------------------------------------------- | ----------- | ------------------------------------- |
| 01  | [The running apps](./01-running-apps.md)              | ✔️ done     | `crates/macos/src/open_files.rs`      |
| 02  | [Ownership in the cleaner](./02-cleaner-ownership.md) | ✔️ done     | `crates/cleaner/src/lib.rs`, compose  |

In order.

## Conclusions

Both steps built as written. The cleaner now skips an item a running app owns by bundle id as `InUse`, the same
reason and text as an open file; a failed listing acts on nothing. The known issue of other users' processes is
narrowed, not closed.
