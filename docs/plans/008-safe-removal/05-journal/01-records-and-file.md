# 008/05/01 — Records and the file

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** store

## Files

- `crates/store/src/journal/mod.rs` (new) — `FileCleanupJournal::open(&DataFolder) -> Self` (never fails: a file that
  cannot be used makes the journal read-only, with the reason) and the port's writing half.
- `crates/store/src/journal/records.rs` (new) — the on-disk forms of the core's records (design, section 8): the
  header `{"format":"cleanup-journal","version":1}`, then one JSON object per line with a `type` tag (`begin`, `item`,
  `end`, `put-back`); serde forms of their own, so a change of an IPC type never changes the file.
- `crates/store/src/journal/path_text.rs` (new) — a path as a string when it is UTF-8, as `{"bytes":"<hex>"}`
  otherwise; read back to the same bytes.
- `crates/store/src/journal/file.rs` (new) — appending: the file opened with `O_APPEND` and mode `0600`, created with
  its header line when missing; if the last byte is not a newline (a torn line), a newline is written first. `begin`
  takes the next `CleanupId` (one more than the highest in the file, read at open) and is synced (`sync_data`) before
  it returns; `item` appends without a sync; `sync` syncs (the engine calls it every 50 items and every second);
  `end` and `put_back` append and sync.
- `crates/store/src/lib.rs` — `mod journal;` and `pub use journal::FileCleanupJournal;`.

## Produces

The journal the app writes (06/04): every cleanup recorded before its first act.

## Checks

- Negative (first): `a_missing_file_starts_with_its_header`, `a_torn_last_line_gets_a_newline_before_the_next_record`,
  `begin_is_synced_before_it_returns` (a test hook counts the syncs), `the_file_is_private_to_the_user` (mode `0600`),
  `a_record_never_holds_a_field_the_design_does_not_name` (the serialized forms against a fixed list).
- Positive: `ids_continue_from_the_highest_in_the_file`, `records_append_in_order`,
  `a_non_utf8_path_round_trips` (property: any bytes), `a_cleanup_of_10000_items_writes_one_begin_line`.

## Conclusions

- Built as written: `journal/{mod,records,path_text,file,read}.rs` and `journal/tests.rs`, `FileCleanupJournal::open`
  exported from `fresher-store`. `proptest` 1.12.0 is a dev-dependency of the store now (`docs/dependencies.md`).
- **The format**: `{"format":"cleanup-journal","version":1}`, then compact JSON lines tagged by `type`. The forms are
  the file's own (`records.rs`): the core's field-less enums are mirrored by a small `mirror!` macro whose matches are
  exhaustive both ways, so a new variant in the core does not compile until the file has its form; the reason is
  `{"key", "args"}` with each argument tagged (`text`, `count`, `size`, `date`, `path`).
- Changed from the design's sketch, each the plainer form of the frozen contract: an `item` record holds
  `"outcome":{"kind":…}` and a separate `"trashed":[{"original","landed","identity"}]` (the contract's `ItemRecord`),
  not `landed` inside the outcome; a `put-back` record's outcome is tagged too (`{"kind":"put-back"}`), since
  `Refused` carries its skip code.
- **Durability**: `begin` writes its one line and `sync_data`s before it returns; a new file gets its header synced
  and then its folder synced, so the name is durable before any record relies on it (a failure there refuses the
  `begin`: fail closed). `item` appends only; `sync`, `end` and `put_back` sync. A failed write drops the open file;
  the next write reopens it and, when its last byte is not a newline, closes the torn line first. The id of a `begin`
  is spent even when its write fails, so a line that reached the disk is never followed by another of the same id.
- The line reader came with this step, not 02: the next id is read at open (the highest id in any record, plus one).
  Reading is streamed (`read_until` into one reused buffer).
- No error holds a path: an I/O error keeps only its kind; `StorageFull` and `QuotaExceeded` are `NoSpace`.
- Checks as listed, plus `a_torn_last_record_never_corrupts_the_records_before_it` (the last record cut at every byte:
  the earlier cleanup reads the same and the next `begin` lands on a line of its own),
  `a_failed_write_is_reported_and_writes_nothing_after_it`, and the record forms' own tests in `records.rs`,
  `path_text.rs` and `file.rs`.
