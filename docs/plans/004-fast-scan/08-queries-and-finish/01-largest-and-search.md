# 004/08/01 — Largest files and search

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner

## Files

- `crates/scanner/src/state/largest.rs` — `largest_files(limit)`: the kept file records (every file of 1 MiB and up
  is one), the largest allocated size first, ties by path order; a bounded binary heap of `limit` (at most 500, like
  findings), so the cost is one pass and no sort of everything. Hard-link records with no bytes of their own are
  skipped.
- `crates/scanner/src/state/search.rs` — `search(query, limit)`:
  - an empty or whitespace-only query answers nothing; a query over 255 bytes is cut to 255;
  - the query and the names are compared case-insensitively: a lowercase copy of the name buffer (Unicode lowercase
    of `std`) is built on the first search of a state and kept for the next searches of the same state; no Unicode
    normalization (it would need a new crate; a name typed in another normal form than it is stored in is a known
    limit, written in the notes if 01 found such names);
  - a linear byte-substring scan over that copy, with no index and no new crate;
  - matches ranked by allocated size, the largest first, at most `limit` (≤ 500); folders and kept files both.
- `crates/scanner/src/state/mod.rs` — `TreeState` answers `largest_files` and `search` through them.
- `crates/scanner/benches/scan.rs` — groups `state/largest` and `state/search` on `LARGE` (a common query, a rare
  one, a one-letter one).

## Produces

The two queries plans 003 and 012 show, and the large files finder idea builds on.

## Checks

- Negative: `an_empty_query_finds_nothing`, `a_query_matches_no_folded_small_file` (small files are not in the tree:
  the answer says so by construction, and the doc comment says so), `a_hard_link_without_bytes_is_not_among_the_largest`.
- Positive: `search_ignores_case`, `search_ranks_by_size`, `search_finds_folders_and_kept_files`,
  `largest_files_are_in_size_order`, `the_lowercase_copy_is_built_once` (`heap_bytes` grows on the first search only).
- Bench numbers in the notes: search on `LARGE` ≤ 50 ms (the design's bar), the lowercase copy's size.

## Conclusions

- `TreeState::largest_files(limit)` (`state/largest.rs`): one pass over the tree in path order (depth first, children
  by name), the kept files offered to a binary heap of at most `limit` (capped at 500, `FindingsQuery::MAX_LIMIT`);
  ties go to the earlier path. A kept file flagged `LinkCopy` (its hard link group counted at another path) is skipped.
- `TreeState::search(query, limit)` (`state/search.rs`): the query is cut to 255 bytes at a character boundary and
  lowercased; empty or whitespace only, it finds nothing and builds nothing. The first search builds the lowercase
  copy (`OnceLock` in the state): the names of the folders and kept files in path order, each lowercased with
  `str::to_lowercase` (a name that is not UTF-8 read lossily) and ended by a NUL, plus the start and the node of each
  name. One `match_indices` pass over it (the standard library's substring search, no new crate) maps each match to
  its name, once per name, into the same bounded heap. `TreeState::heap_bytes` counts the copy once built.
- **Calls taken** (unspecified by the step): the scan roots are not searched (their name is their whole path, so a
  query would match a part of the path above the root); symbolic links are not searched (the step names folders and
  kept files); ties in search go to the earlier path, as in `largest_files`. 01 did not count names stored in another
  Unicode normal form than they are typed, so no known-issue file is written; the limit stays in the doc comment.
- The tests build trees in memory (`state/query_tests.rs`) and through the scanner on fixture trees.
  `bench::QueryBench` gives the benches the state; `committed` of the state's tests is shared with them.
- **Numbers** (`LARGE` scanned with the scan bench's interests, so every `*.tmp` file is a node; release; load
  average 15–31 from other work on the machine): 587 637 nodes, the tree 61.2 MB on the heap, **the lowercase copy
  16.2 MB** (27.6 B a node), the first search (building the copy) 36 ms; `state/largest/large` 22 ms;
  `state/search/large` **12 ms** for a common query (`cache`), 9.5 ms for a rare one, 15.7 ms for one letter — the
  bar is 50 ms.
