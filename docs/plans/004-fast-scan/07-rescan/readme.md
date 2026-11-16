# 004/07 — Rescan and restore

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner, engine, app, e2e

Every later scan reads only what changed: the cache is loaded and checked, the change journal names the folders that
changed since the cached position, only those are read again, a sample of the others is checked, and any doubt
means a full scan. At launch the last state is restored without touching the disk. The design:
[`../00-design.md`](../00-design.md), sections 8 and 9.

Depends on 03 (the journal), 05 (the walker) and 06 (the cache). One plan at a time from here: it owns
`crates/scanner/src/rescan/` and edits the walker, the engine and the app in sequence. The rescan logic is tested over
`FakeChangeJournal` (deterministic); the FSEvents adapter's own tests are in 03/04; the end-to-end tests use
`FakeChangeJournal` too, so no test waits on the system's journal.

## Steps

| #   | Step                                                                  | Status   | Shared files                                       |
| --- | --------------------------------------------------------------------- | -------- | -------------------------------------------------- |
| 01  | [Save, load and the change set](./01-save-load-change-set.md)         | ✔️ done    | `crates/scanner/src/scanner.rs`, `crates/scanner/src/state/`, `crates/scanner/benches/scan.rs` |
| 02  | [Patching the tree](./02-patching.md)                                 | ✔️ done    | `crates/scanner/src/walk/`                         |
| 03  | [The check and the fallbacks](./03-check-and-fallbacks.md)            | ✔️ done    | `crates/scanner/benches/scan.rs`                   |
| 04  | [Restore at launch and scan modes](./04-restore-and-modes.md)         | ✔️ done    | `crates/engine/`, `app/tauri/`, bindings, `crates/e2e/`, `fixtures/` |

In order.

## Conclusions

- Every later scan reads only what changed: the cache saved at each commit is loaded and checked, the journal's
  changed folders are read again with their parents, the rest of the tree is copied, a sample of 32 copied folders is
  read and compared, and any doubt is a full scan with its reason in `WalkSummary::fallback` (`NoCache`,
  `BrokenCache`, `OtherRoots`, `OtherVolume`, `OtherAvoidList`, `OtherFingerprint`, `JournalGap(..)` including
  `TooOld` past 14 days and `RootChanged` when a root's identity changed, `TooManyChanges` past 50 %, `CheckFailed`,
  `AskedByUser`). At launch the app restores the last state without reading a folder.
- **Numbers** (`LARGE`, release): rescan unchanged 32 ms, 1 000 changed folders 254 ms, half the folders 19 s
  against a full scan of 25 s; the cache body 2.8 MB (24 B a node), 1.7 MB on disk, encode 15 ms, decode 10 ms,
  fit 51 ms. A tree whose matched files are kept everywhere (the scan bench's `*.tmp`) does not fit the 8 MiB cap
  and is not cached.
- **Deviations**, each in its step: the parent of a changed folder is read too (the tree keeps no file id and no
  time but its parent's listing; 01); no `replace` field in the job (02); no file id in the check and the seed from
  the clock (03); the e2e cannot count only the changed folder, and no separate restore bench (04).
- **For the user**: the by-hand run in `npm run tauri dev` (07/04).
