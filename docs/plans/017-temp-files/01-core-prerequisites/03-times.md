# 017/01/03 — The times of an item and of its inside

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
core, macos, scanner

Design section 5: an archive tool keeps old modification times, a folder's own times do not move when a file deep
inside is written, and access times are lazy. The walk gives the status-change time and the newest change of a whole
folder, gathered while it sums the folder.

## Files

- `crates/core/src/folder.rs` — `RawEntry` gains `changed: Option<Timestamp>` (status change) and
  `accessed: Option<Timestamp>`.
- `crates/core/src/inside.rs` (new), `lib.rs` — `Inside { newest_change, newest_file_access, special, foreign,
  complete }` (design section 5) and `Inside::join(&mut self, entry)`: the newest of `modified` and `changed`; the
  newest `accessed` of files only; `special` for an `Other` entry; `foreign` for an entry not owned by the user or
  with any lock or system flag (`EntryAccess` against the scan's user); `complete` false on an unreadable or
  not-entered folder.
- `crates/core/src/interest.rs` — `Match` gains `changed`, `accessed` (the entry's own) and `inside: Option<Inside>`
  (`Some` for a `Claim::Whole` folder, `None` for anything else).
- `crates/macos/src/bulk/` — the bulk read asks `ATTR_CMN_CHGTIME` and `ATTR_CMN_ACCTIME`; `parse.rs` fills the two
  times (no allocation per entry; the attribute buffer's layout test follows).
- `crates/scanner/src/tree/` — a whole folder's `Inside` is joined in the same pass that sums its sizes; a rescan
  patches it with the folder's sizes; `codec/` stores the three new values (format version up); `rescan/` and the
  restore hand them back unchanged.
- `crates/scanner/src/naive*` — the reference walk computes the same from `lstat` (`st_ctime`, `st_atime`).

## Produces

`Match.changed`, `Match.accessed`, `Match.inside`, for the module's age, kind and owner rows.

## Checks

- Negative first: a folder whose files carry 2019 modification times set with `utimes` has a `newest_change` of now
  (their status change); a folder with one old-looking file changed yesterday deep inside has yesterday's; a folder's
  own access time never counts; a folder holding an unreadable folder is not `complete`; a folder holding a pipe is
  `special`; a file or a link has no `inside`.
- Positive: over a fake `FolderReader` with given times, a tree's `newest_change` is the newest of its entries'
  modification and status-change times, and `newest_file_access` its newest file's access time; the bulk reader and the reference walk agree on every
  value (the property test over random trees, times included); a cache round trip and a restore keep them; a rescan
  after one file changes moves its folder's `newest_change`.
- Rule 4: the scan bench (`npm run bench`) before and after; the two attributes and the join must not move the scan's
  time by more than 5 % (written in the conclusions).
- `cargo nextest run -p fresher-core -p fresher-macos -p fresher-scanner`; every existing golden unchanged or
  regenerated with the new fields, reviewed.

## Conclusions

Built (6efc57d, review fixes e289fc0). The scan bench (small tree, before and after interleaved three times) moved
`scan/full` by −1 %, +1.4 %, +2 % and +4.6 % at 1, 12, 24 and 36 threads, within the 5 %: the two attributes are always
asked. The tree keeps the times of the nodes with hits only (`[i64; 2]` per node, about 36 bytes); on the bench tree,
where an extension hits one file in five, its memory grew from 2.0 MB to 2.7 MB (+35 %; the first layout took 4.3 MB).
The cache body stays at 25 bytes a node.

Calls taken while building:

- `Match.accessed` is `None` for a folder: its own access time moves when the walk lists it, which made the thread-count
  and rescan comparisons flaky.
- `Inside::join` takes the scan's user; `foreign` uses the same lock set as `EntryAccess`. `Inside` is gathered for
  whole-claimed subtrees only; a folder not entered (avoided, protected, another volume) or unreadable makes it
  incomplete.
- A time before the epoch is unknown, in the bulk read and the reference walk alike.
- `fresher_testing::settle_access_time`: a file written across a second keeps an access time older than its
  modification, and the first walk that reads it moves it; the fixture builders settle every file.

The review found that a rescan could hand back cached access and change times older than the disk's; the ruling
(decision 21) is that the walked places are always read again in full, built in 01/04. A folder a rescan copies without
reading its parent again still keeps its cached own times, like its modification time; the places are not affected. The
cache's tree format went to 4 (5 in 02/02).

Seen while building, not caused here: the sandbox autotest `storage_map` flakes under the whole workspace's load (a 48
MiB file's allocated size off by 48 blocks), on the base commit too; and the rescan property test fails about once in
1,500 cases on `newest_file_access` ([known issue](../../../known-issues/scanner-rescan-property-access-time.md)).
