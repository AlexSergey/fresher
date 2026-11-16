# 004/07/03 — The check and the fallbacks

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner

The journal is trusted only as far as a sample confirms it.

## Files

- `crates/scanner/src/rescan/check.rs` — `check(tree, change_set, reader, seed) -> Result<(), FullScanReason>`:
  picks `CHECK_SAMPLE` folders (32; tuned here) that the change set does not hold, by a seeded generator over the
  folder nodes — the seed is the scan id, so a test fixes it — weighted towards large subtrees and claimed folders
  (where a missed change costs the most); reads each and compares its entry names, kinds, file ids and folded sums
  with the node; one difference → `CheckFailed`, logged with the count of differences and no path.
- `crates/scanner/src/scanner.rs` — the check runs after the patch and before `emit`; a failure throws the patch
  away and runs a full scan (`fallback: CheckFailed`). The user asking for a full scan (`WalkMode::Full`) skips
  everything and sets `fallback: AskedByUser`.
- `crates/scanner/benches/scan.rs` — groups on `LARGE`, with the macOS reader and a scripted `FakeChangeJournal`
  (the benches measure the scanner, not the system's journal): `scan/rescan-unchanged`, `scan/rescan-day` (1 000
  changed folders, the size of an ordinary day from 01/02), `scan/rescan-limit` (changed folders from 1 % to 50 %,
  against a full scan) — the crossing point sets `RESCAN_LIMIT`.

## Produces

The last guard of the rescan; `RESCAN_LIMIT` and `CHECK_SAMPLE` as measured constants.

## Checks

- Negative: `a_change_the_journal_missed_fails_the_check` (the fixture changed, the fake journal reports nothing;
  with a fixed seed whose sample holds the changed folder), `a_failed_check_falls_back_to_a_full_scan_with_its_reason`,
  `a_full_scan_asked_by_the_user_reads_every_folder`.
- Positive: `an_honest_journal_passes_the_check`, `the_sample_is_the_same_for_the_same_seed`.
- **Bars** (deterministic, counting reader on `SMALL`): `a_rescan_with_nothing_changed_reads_only_the_sample_and_the_unreadable`
  (reads = `CHECK_SAMPLE` + unreadable folders), `a_rescan_reads_each_changed_folder_once`.
- Bench numbers in the notes against the design's bars (rescan unchanged ≤ 0.5 s, a day ≤ 2 s on `LARGE`), and the
  chosen `RESCAN_LIMIT`.

## Conclusions

- `rescan/check.rs`: `check(tree, unchanged, walk, seed, cancel)` reads `CHECK_SAMPLE` (32) folders the patch copied
  without reading, drawn by `sample(tree, unchanged, seed)`: a seeded weighted draw (SplitMix64, keys `ln(u) / w`)
  over the readable folders (not opaque, collapsed, avoided, on another volume or unreadable), the weight growing with
  the log of the folder's allocated size and doubled for a folder with a hit. Each folder is read and folded by the
  full scan's worker and compared with its node: its kept entries' names and kinds, its kept files' logical and
  allocated sizes, its folded bytes (as `Tree::apply` counts them) and whether an entry failed; a folder that fails
  to read or is now claimed whole is a difference. One difference: `CheckFailed`, logged with the count, no path.
- **Changed from the step**: the tree keeps no file id, so the check does not compare file ids (a file replaced by
  another of the same size and name is left to the journal); and the seed is the time of the rescan from the
  injected clock, because the scanner does not know the engine's scan id (a test fixes it through the clock).
- The check runs after the patch and before the emit; a failure throws the patch away and runs a full scan with
  `fallback: CheckFailed`. `WalkMode::Full` sets `fallback: AskedByUser` and never touches the cache or the journal.
  `ScannerOptions::rescan_limit_percent` carries `RESCAN_LIMIT_PERCENT` (the benches raise it to measure past it).
- **Bench numbers** (`LARGE`, release, the macOS reader, a scripted journal, 12 threads, 113 453 folders; the
  interests of the cache bench): `scan/rescan-unchanged` **32 ms** (bar 0.5 s), `scan/rescan-day` (1 000 folders
  named, read with their parents) **254 ms** (bar 1 s in the design, 2 s in this step). `scan/rescan-limit`, folders
  named by the journal: 1 % 279 ms, 2 % 684 ms, 5 % 2.6 s, 10 % 7.7 s, 20 % 8.7 s, 30 % 12.1 s, 50 % 19.2 s; a
  full scan 25.0 s (cold: `LARGE` does not fit the kernel's metadata cache, as 05 found). Half the folders named still
  beat a full scan, so **`RESCAN_LIMIT_PERCENT` is 50** (it counts the folders read again with their parents and every
  folder below a node walked again, so it bites before half the tree is named). `CHECK_SAMPLE` stays 32: the sample
  is part of the 32 ms of an unchanged rescan. The bench process peaked at 351 MB after every group (the scans, the
  cache group's copies and criterion's samples), not a single scan.
- Tests: the step's checks; the bars run on `SMALL` with a counting reader: an unchanged rescan reads exactly the 32
  sampled folders and the 3 unreadable ones, and a rescan naming one folder in 50 reads none twice.
