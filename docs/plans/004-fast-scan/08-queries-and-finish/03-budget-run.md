# 004/08/03 — The budget run

**Status:** ✔️ done (the run by hand is [deferred](../../../deferred/fast-scan-home-run.md)) · **Version:** 0.3.0 ·
**Date:** 2026-11-16 · **Modules:** scanner, docs

Every bar of the design, measured on the final code.

## Files

- `docs/plans/004-fast-scan/readme.md` — "Conclusions", section "Measurements": a table with every row of the
  design's targets (section 13 as 01/04 set it), the measured value, the machine, and pass or fail.
- `docs/plans/004-fast-scan/00-design.md` — a bar that fails is not quietly raised: the step stops, the cause is
  investigated (a profile, the counting reader), and either the code is fixed in a follow-up step of the plan or the
  bar is changed as a decision recorded with the user.

## What is run

1. `npm run pre-push` — the deterministic bars (folders read, bytes per node, the cap, the footprint) pass.
2. `FRESHER_BENCH_TREE=large npm run bench` — every group, on a quiet machine on power, after one warm-up run; the
   medians and the peak memory.
3. By hand, the release build (`npm run build`), the developer's home folder, read-only: the first scan, a rescan
   after a minute, a rescan after a day of ordinary use, a restore at launch; the time, the peak memory from Activity
   Monitor, the cache file sizes and `DataFolder::footprint()` (shown by a debug log line). No path is recorded.
4. A cold first scan once after a reboot, by hand, recorded with no bar (it depends on the disk, not on the code).

## Produces

The measured numbers of the conclusions; plan 010 starts from the cache sizes measured here.

## Checks

- Every target row has a measured value; every failing bar has a recorded decision.
- The footprint after ten scans of `LARGE` with changes between them stays within `FOOTPRINT_BUDGET`, and the data
  folder holds only the settings file and the four slot files (plus any stale temporary file of the
  [known issue](../../../known-issues/fixed/store-stale-temporary-files.md), counted and noted).
- A partly rewritten APFS clone gets a new clone id while it still shares most of its blocks (measured outside the
  plan, 2026-11-15). On a real tree with such clones, the totals equal the allocated bytes on the volume within the
  bar, so no shared block is counted twice and none is lost.

## Conclusions

The automated part is measured; the runs by hand (3 and 4 of "What is run", and the clone check) are the user's, in
the checklist below. The step is done when they are filled in and the failing bars below have a decision.

**Added for the run**: a `scan/restore/<tree>` bench (the cache decoded and the matches emitted, no folder read: the
restore at launch without the engine); `crates/e2e/tests/footprint.rs`, ten scans of a generated tree through the
real scanner and `FileScanCache` on a temporary data folder, a scripted journal naming the changed folder (so scans
2–10 are rescans that load the saved cache), a new folder of `round × 100` subfolders added and the one of two rounds
before moved out of the scanned roots between scans. On `SMALL` it runs in the hooks (≈ 2 s); on `LARGE` it is
ignored and run by hand: `cargo nextest run -p fresher-e2e --release --run-ignored only --no-capture
ten_scans_of_the_large`.

**The machine**: as in 01 (Apple M3 Pro, 12 cores, 18 GiB, internal SSD, APFS, macOS 27.2.1), on power, release
builds, `kern.maxvnodes` 263 168. Other work ran on the machine: the load average is given with each run. Criterion
warms up before it samples; medians below.

### Measurements

| What (design, section 13)                     | Bar           | Measured                                                                 | Load   | Result |
| --------------------------------------------- | ------------- | ------------------------------------------------------------------------ | ------ | ------ |
| full scan, warm, `large` (3 M entries)        | ≤ 10 s        | `LARGE` cannot be warm here (3 M entries past the kernel's 263 k vnodes); warm `SMALL` (100 k): 289 ms at 12 threads, 345 000 entries/s → 8.7 s for 3 M | 6–14 | pass, projected |
| full scan, cold, `large`                      | none (05)     | 22.8 s at 12 threads (one per core, the default), 15.7 s at 24 (two per core) | 6–14 | recorded |
| full scan, `small`, by threads                | —             | 1 thread 380 ms; 12: 289 ms; 24: 373 ms; 36: 357 ms; 48: 369 ms           | 7–14   | recorded |
| rescan, nothing changed, `large`              | ≤ 0.5 s       | 32 ms                                                                    | 5–7    | pass   |
| rescan after 1 000 changed folders, `large`   | ≤ 1 s         | 231 ms                                                                   | 5–7    | pass   |
| restore: cache → matches emitted, `large`     | ≤ 0.2 s       | 12.7 ms (the engine's part is not in it)                                 | 5–7    | pass   |
| search by name, `large`                       | ≤ 50 ms       | common 4.0 ms, rare 4.5 ms, one letter 12.9 ms; the first search, which builds the 16.2 MB lowercase copy, 35 ms | 5–7 | pass |
| largest files, `large`                        | —             | 9.1 ms                                                                   | 5–7    | recorded |
| peak memory of a full scan, `large`           | ≤ 200 MB      | 143 MB at 12 threads (a process that ran only that scan); **402 MB at 24 threads**; 226 MB for a scan plus its tree kept, encoded and fitted (the cache bench) | 6–14 | pass at the default |
| tree heap per node                            | ≤ 96 B        | the test on its synthetic tree passes; **104 B** on `LARGE` scanned with every `*.tmp` file matched (587 637 nodes, 61.2 MB): a matched file is a 72-byte node, its name and a 24-byte hit | 5–7 | **fails on that tree** |
| cache body per node, uncompressed             | ≤ 32 B        | 24 B (`LARGE`, 114 478 nodes, body 2.77 MB)                              | 5–7    | pass   |
| one cache file / the whole cache              | ≤ 8 / 32 MiB  | 1.73 MB a file; `LARGE` with every `*.tmp` kept does not fit and is not saved (07) | —  | pass   |
| data folder after ten scans of `large` with changes | ≤ 50 MB (`FOOTPRINT_BUDGET`) | 1.73 MB after the first scan, 3.46 MB after each next (the settings file and `scan-1.bin` with `scan-1.previous.bin`; the file grew from 1 725 480 to 1 726 607 B over the rounds); no stale temporary file; 26 s for the ten | 8–12 | pass |
| folders read by a rescan with nothing changed | sample + unreadable | held by the tests of 07/03 in `npm run pre-push`                   | —      | pass   |

**For the user — decisions** (a failing bar is not raised quietly):

1. **Threads**: one per core is the faster on a warm tree (`SMALL`: 289 ms against 373 ms) and keeps the peak at
   143 MB; two per core is the faster on a cold one (`LARGE`: 15.7 s against 22.8 s) and takes the peak to 402 MB,
   twice the bar. The recommendation: keep one per core (at most 16), as 05 took it; confirm it on the home folder
   (checklist, item 2).
2. **Tree heap per node on `LARGE`**: 104 B against 96 B, on a tree where every `*.tmp` file is matched and kept. The
   bar's test (synthetic names, few hits) passes. The recommendation: keep the bar on the test, record the
   generated tree's 104 B as what a tree of many matched files costs, and compare it with the home folder's number
   (item 1) before deciding; no code change in this plan.

Both recommendations are taken (2026-11-16, the user asked for the recommended calls); the home-folder numbers of the
run by hand confirm or reopen them.

### By hand (the user)

Read-only, the release build (`npm run build`, then the built app), the developer's home folder. No path is recorded;
counts, times and sizes only. The data folder's footprint is shown by the debug log line of `DataFolder::footprint()`.

1. First scan of the home folder: time ____ s; peak memory (Activity Monitor) ____ MB; cache files ____ B and ____
   B; footprint ____ B; tree heap per node, if logged ____ B.
2. The same scan with two threads per core, if the user wants it settled (a build with the thread count changed by
   hand): time ____ s; peak memory ____ MB.
3. A rescan after a minute: time ____ ms; changed folders ____; fallback, if any ____.
4. A rescan after a day of ordinary use: time ____ ms; changed folders ____; fallback, if any ____.
5. Quit and start the app (the restore at launch): the last result shows in ____ ms, labelled with its date.
6. Clones: on the home folder, the scan's total allocated bytes ____ against the volume's used bytes for the same
   places (`du -sk` of the roots, read-only) ____: within the bar, so no shared block is counted twice or lost.
7. Once after a reboot, before anything else reads the disk: the first scan, cold, ____ s (recorded with no bar).
