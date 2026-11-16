# 008/05/03 — Rotation and the footprint

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** store, e2e

## Files

- `crates/store/src/journal/rotate.rs` (new) — at `begin`, when `cleanups.jsonl` is 4 MiB or more
  (`JOURNAL_GENERATION = 4 MiB`), it is renamed over `cleanups.previous.jsonl` and a new file starts with its header;
  ids continue (the next id is read before the rename). A single `begin` larger than a generation (a plan of 10 000
  items with long paths) is written whole into a fresh file; the cap is a threshold, not a split.
- `crates/store/src/journal/read.rs` — reads the previous generation, then the current one; a cleanup's records never
  span the two (a rotation happens only at a `begin`).
- `crates/e2e/tests/footprint.rs` — the existing ten scans plus a hundred cleanups of a hundred items each through the
  real journal: the data folder stays within `FOOTPRINT_BUDGET` and holds only the settings file, the cache's slot
  files and at most the two journal generations (and `cleanups.broken.jsonl` only in the test that plants one).
- `docs/architecture/README.md` ("Data") — the journal's files, its budget (8 MiB of the 50 MB) and its rotation.

## Produces

A journal that cannot outgrow its share of the footprint, and the store still deleting nothing.

## Checks

- Negative (first): `a_file_under_the_threshold_is_not_rotated`, `rotation_never_happens_inside_a_cleanup`
  (`item` and `end` past the threshold append to the same file), `rotation_replaces_only_the_previous_generation`.
- Positive: `a_file_past_the_threshold_rotates_at_the_next_begin`, `history_reads_across_both_generations`,
  `ids_continue_across_a_rotation`, the footprint test.

## Conclusions

- Built as written: `rotate.rs` (`JOURNAL_GENERATION` = 4 MiB, `cleanups.previous.jsonl`), read before the current
  file; the size check runs only in `begin`, after the file is open and before the line is written, so a cleanup's
  `begin`, items and `end` are always in one file. A rotation that cannot rename refuses the `begin` (fail closed).
- Calls taken where the step said nothing:
  - the threshold is a field of the journal (`JOURNAL_GENERATION` in the app), so the store's tests rotate at a
    hundred bytes instead of writing megabytes;
  - a `put-back` of a cleanup from the previous generation is appended to the current file, and reading attaches it
    by id across the two files (a `put-back` of a cleanup whose generation dropped off is ignored);
  - an older version's file becomes the previous generation at the next write, by the same rename (02's call).
  - at open each generation is read on its own for the next id, so a previous file that cannot be read does not hide
    the ids of the current one (`ids_continue_from_the_current_file_when_the_previous_cannot_be_read`); reading the
    history with such a file answers `JournalError::Failed`.
- **Footprint** (`crates/e2e/tests/footprint.rs`, the small tree): after the ten scans the folder takes 127 KB; a
  hundred cleanups of a hundred items each (paths of ~70 characters, every item moved with its landed path) add
  about 69 KB a cleanup, 690 bytes an item; the journal rotated once, between the 60th and 70th cleanup, and ends at
  6.9 MB in its two files, holding all hundred cleanups; the folder ends at 7.0 MB, within `FOOTPRINT_BUDGET`, with
  exactly `cleanups.jsonl`, `cleanups.previous.jsonl`, `scan-1.bin`, `scan-1.previous.bin` and `settings.json`. The
  test also checks the journal's two files stay within its 8 MiB share.
- Checks as listed, plus `a_begin_larger_than_a_generation_is_written_whole`, the test above and
  `an_older_version_becomes_the_previous_generation_at_the_next_write`. `docs/architecture/README.md` ("Data")
  describes the journal's files, format, write-ahead, versions and rotation.
