# 008/05/02 — Reading and versions

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** store

## Files

- `crates/store/src/journal/read.rs` (new) — the port's reading half:
  - `cleanups(offset, limit)`: summaries newest first (date, method, count, sizes moved and deleted, counts by
    outcome, ended or interrupted); `cleanup(id)`: every item with its outcome, its landed path and its put backs;
  - a line that does not parse is skipped and counted (a torn last line after a crash; nothing else is expected); a
    `begin` with no `end` is `Interrupted`, its items without an outcome `Unknown`;
  - reading is streamed line by line, never the whole file into one string.
- `crates/store/src/journal/mod.rs` — `state()`:
  - **Ready**: the file is missing or its header is this version's;
  - **ReadOnly { NewerVersion }**: the header names a newer version — read what can be read, never write;
  - **ReadOnly { Unreadable }**: the file exists and cannot be opened or read — never write (a passing error must not
    lead to a new file over the user's history, as for the settings, plan 002 call 26);
  - a file whose first line is not a journal header: moved aside to `cleanups.broken.jsonl` (a rename, replacing an
    older broken file) on the next `begin`, then a new file starts — the store's existing "broken" rule.
- Migration: version 1 has none; the store's `migrate` machinery is wired so version 2 adds one function, with a test
  that a version-1 file reads under the current code.

## Produces

The history the Cleanups screen shows (06/03, 07/04), and the fail-closed state the engine checks before a cleanup.

## Checks

- Negative (first): `a_newer_version_is_read_only_and_never_written` (bytes unchanged after a refused `begin`),
  `an_unreadable_file_is_read_only` (mode `000`), `a_broken_header_is_moved_aside_on_the_next_begin`,
  `an_unparsable_line_is_skipped_and_counted`, `a_begin_without_an_end_reads_as_interrupted`.
- Positive: `summaries_come_newest_first_and_paged`, `a_cleanup_reads_back_with_its_outcomes_and_put_backs`,
  `a_version_1_file_reads` (a fixture file in `fixtures/journal/v1.jsonl`, reviewed like a golden file).

## Conclusions

- Built as written: `read.rs` reads line by line into `Collected` (every cleanup, or one by id for `cleanup(id)`),
  `state()` comes from what the first line of `cleanups.jsonl` said at open, and a file whose first line is not a
  journal header is renamed over `cleanups.broken.jsonl` by the next write. `fixtures/journal/v1.jsonl` is the golden
  version 1 file: three cleanups (completed, cancelled with a permanent delete, interrupted with a non-UTF-8 path) and
  two put backs, one refused.
- **Versions**: each line of an older file is parsed to a JSON value, taken through `migrate` with the journal's chain
  (`MIGRATIONS`, empty for version 1), then read; a newer file's lines are read as far as this build understands them.
  `an_older_version_is_migrated_line_by_line` runs a version 2 build with one test migration over the version 1
  fixture.
- Calls taken where the step said nothing:
  - an empty file is a file whose header was never written (a crash right after it was created): it is treated as
    missing and gets its header, never moved aside;
  - the first line is read at most 4 KiB: a longer one is not a journal header;
  - a broken file shows no history until it is moved aside, and is never read as a journal;
  - an unreadable file is read-only, and reading it answers `JournalError::Failed` (the history shows the error, not an
    empty list);
  - a record of a cleanup whose `begin` is not in the file is ignored (its generation was dropped), and a `begin` whose
    reason key the core refuses is skipped and counted like a line that does not parse;
  - writing to an older file comes with the rotation (03): it is renamed into the previous generation at the next
    `begin`, so a file never mixes two versions.
- Checks as listed, plus `an_unknown_cleanup_reads_as_none` and `an_older_version_is_migrated_line_by_line`.
