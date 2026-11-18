# 017/01 — Core prerequisites

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
core, macos, system, scanner, engine, cleaner, testing, sandbox, app

What the research found the core is missing before a temporary-files module can be safe: the two places as known
folders with their protection, a fourth entry kind, the status-change time and the newest change of a whole folder,
places walked outside the scan roots, and a sandbox that states the facts it cannot make. The design:
[`../00-design.md`](../00-design.md), sections 1 to 6.

## Steps

| #   | Step                                                          | Status                                  | Shared files                                                                                                   |
| --- | ------------------------------------------------------------- | --------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| 01  | [The temporary places and their protection](./01-places.md)   | ✔️ done                                | `crates/core/src/{known_folder.rs,settings/folders.rs}`, `crates/macos/src/{protection.rs,ffi/}`, `crates/system/src/known_folders.rs`, `crates/testing/src/known_folders.rs`, `app/tauri/src/compose.rs` |
| 02  | [A fourth entry kind](./02-entry-kind.md)                     | ✔️ done                                | `crates/core/src/{interest,folder}.rs`, `crates/macos/src/bulk/`, `crates/scanner/src/{matcher,codec,naive*}`, `crates/cleaner/src/guard/` |
| 03  | [The times of an item and of its inside](./03-times.md)       | ✔️ done                                | `crates/core/src/{interest,folder,inside,lib}.rs`, `crates/macos/src/bulk/`, `crates/scanner/src/{tree,codec,rescan,naive*}` |
| 04  | [Places outside the scan roots](./04-walked-places.md)        | ✔️ done                                | `crates/core/src/walk.rs`, `crates/scanner/src/scanner.rs`, `crates/engine/src/`, `crates/cleaner/`, the map's goldens |
| 05  | [The sandbox's stated facts](./05-stated-facts.md)            | ✔️ done                                | `crates/testing/src/`, `crates/sandbox/src/`, `app/tauri/src/sandbox.rs`, the harness, `docs/testing.md`       |

01 and 02 in parallel; 03 after 02; 04 after 01; 05 after 03 and 04.

## Conclusions

The core gained what the module needs, each part with its tests: the two places and their protection (01), a fourth
entry kind that fixed the FIFO known issue (02), the status change, the access and the inside of a whole folder, within
the scan's 5 % (03), the places walked outside the scan roots and always read from the disk (04, decision 21), and the
sandbox's stated facts (05). The cache's tree format went from 2 to 5 over the plan (02, 03 and 02/02's lock
names; 04's flags fit format 4). What changed from each brief, and why, is in its step.
