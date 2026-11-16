# 008/05 — The journal

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** store, e2e

`fresher-store`'s `FileCleanupJournal` implements the core's `CleanupJournal` port: an append-only JSON-lines file in
the data folder, written ahead of every act, read back into the history, kept within its share of the footprint by
rotation, and never written when it comes from a newer version or cannot be read. The store still deletes nothing. The
design: [`../00-design.md`](../00-design.md), section 8; the store's policy:
[`architecture/README.md`](../../../architecture/README.md), "Data".

Depends on 02. Runs in parallel with 03, 04 and 06/01–06/03. Every test runs on a temporary data folder.

## Steps

| #   | Step                                                         | Status   | Shared files                    |
| --- | ------------------------------------------------------------ | -------- | ------------------------------- |
| 01  | [Records and the file](./01-records-and-file.md)             | ✔️ done    | `crates/store/src/lib.rs`       |
| 02  | [Reading and versions](./02-reading-and-versions.md)         | ✔️ done    | —                               |
| 03  | [Rotation and the footprint](./03-rotation-and-footprint.md) | ✔️ done    | `crates/e2e/tests/footprint.rs` |

In order.

## Conclusions

The three steps are built as written, with the calls each step's conclusions give; `npm run pre-push` passes on the
last commit. `fresher-store` exports `FileCleanupJournal::open(&DataFolder)`, which never fails: the app passes it to
the engine as its `CleanupJournal` (06/04).

- **The file**: `cleanups.jsonl`, `0600`, a header `{"format":"cleanup-journal","version":1}` and one compact JSON
  record per line (`begin`, `item`, `end`, `put-back`), in forms of the file's own; a non-UTF-8 path is
  `{"bytes":"<hex>"}`. One change from the design's sketch: an `item` keeps `outcome` and `trashed` (original, landed,
  identity) apart, as the frozen `ItemRecord` does.
- **Crash safety**: `begin` is one line, synced (with the folder of a new file) before it returns; a torn last line is
  skipped and counted when reading and closed with a newline before the next record; earlier records are never
  rewritten (`a_torn_last_record_never_corrupts_the_records_before_it` cuts the last record at every byte). A failed
  write drops the open file and answers the error, `NoSpace` for a full disk, so the engine stops; no error holds a
  path.
- **Fail closed**: a newer version or an unreadable file is read-only (`JournalState::ReadOnly`), never written; a file
  that is not a journal is renamed aside at the next write.
- **The footprint**: two generations rotated by rename at 4 MiB; a hundred cleanups of a hundred items take 6.9 MB with
  the scans' cache, the folder stays at 7.0 MB of the 50 MB. The store still deletes nothing.
