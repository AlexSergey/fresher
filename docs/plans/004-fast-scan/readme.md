# 004 — Fast scan and search

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** core, macos, scanner, store,
engine, testing, app

## Goal

The real disk walk: one parallel pass over the scan roots that reads each folder in one system call, matches every
module's interests with one lookup per entry, keeps a compact tree in memory, and streams results from the first
second. Every later scan reads only the folders macOS says changed, with a full scan on any doubt. A cache of a few
megabytes — capped, never deleting anything, safe to lose — lets the last result show at launch. Scan time, memory and
cache size are measured and held by tests and benches. The design: [`00-design.md`](./00-design.md).

Branch: `feat/004-fast-scan`, squash-merged into `main` once when done ([`../../workflow.md`](../../workflow.md)).
Depends on plan [002](../002-base-architecture/readme.md) (ports, events, `fresher-store`). Independent of plan
[003](../003-base-ui/readme.md): the two can run in parallel. Fits plans [006](../006-permissions/readme.md) and
[009](../009-system-knowledge/readme.md) through the interfaces in the design, section 12.

## Plans

| #   | Plan                                                        | Status   | Depends on       | What                                                                                          |
| --- | ----------------------------------------------------------- | -------- | ---------------- | --------------------------------------------------------------------------------------------- |
| 01  | [Measure first](./01-measure-first/readme.md)               | ✔️ done    | —                | throwaway probes: the bulk read, the change journal, volumes, links, clones, prompts; the numbers into the design |
| 02  | [Contracts](./02-contracts/readme.md)                       | ✔️ done    | 01               | the core's ports and types: walk request and commit, sizes and markers, OS ports, disk state, cache |
| 03  | [macOS layer](./03-macos-layer/readme.md)                   | ✔️ done    | 02               | `fresher-macos`: the unsafe boundary, IO policy, bulk read, volumes, the change journal        |
| 04  | [Tree and matcher](./04-tree-and-matcher/readme.md)         | ✔️ done    | 02               | the arena, folding, links and totals, the compiled matcher                                    |
| 05  | [Walker](./05-walker/readme.md)                             | ✔️ done    | 03, 04           | large trees and benches, boundaries, workers, errors, claims and commit, the app wired        |
| 06  | [Cache](./06-cache/readme.md)                               | ✔️ done    | 02 (06/02: 04)   | the store's cache slots and envelope, the tree codec, fitting the cap                         |
| 07  | [Rescan and restore](./07-rescan/readme.md)                 | ✔️ done    | 03, 05, 06       | the change set, patching, the check and fallbacks, restore at launch and scan modes            |
| 08  | [Queries and finish](./08-queries-and-finish/readme.md)     | ✔️ done        | 07            | the disk state, search, the budget run, docs and version 0.3.0                                 |

### Order and parallel work

```
01 ── 02 ──┬── 03 ──────────────────┬── 05 ──┐
           ├── 04 ──────────────────┤        ├── 07 ── 08 ── conclusions
           └── 06/01 ── 06/02–03 ───┘ (06/02 also after 04)
```

- **Wave 1**: 01, its steps in order (probes, then the numbers written into the design).
- **Wave 2**: 02, its steps in order. The core is frozen after it, as in plan 002: a contract missing in a later plan
  stops that plan, and the change goes into `crates/core` in the main session.
- **Wave 3, in parallel**: 03, 04 and 06/01. Each runs in its own worktree branched from `feat/004-fast-scan` and is
  merged back when its plan (or step) is done; two agents never share a worktree.
- **Wave 4, in parallel**: 05 (after 03 and 04) and 06/02–06/03 (after 04 and 06/01).
- **Wave 5**: 07.
- **Wave 6**: 08, then the conclusions.

Which files each plan touches, so parallel plans stay apart:

| Plan | Owns (only this plan edits it during its wave)                                                                 | Shared files it touches                                                                                           |
| ---- | -------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| 01   | `NO_COMMIT/probes/004/` (never committed)                                                                      | `docs/plans/004-fast-scan/` (01/04 only)                                                                          |
| 02   | `crates/core/` (its locales included)                                                                          | `crates/rules/src/cachedir.rs`, `crates/testing/` (fakes, `NaiveWalk`), `crates/engine/` (adapts to the port), `app/tauri/src/adapters.rs`, `app/ui/src/ipc/bindings.generated.ts`, `docs/hacks/README.md`, `docs/architecture/README.md` |
| 03   | `crates/macos/`                                                                                                | root `Cargo.toml` (lints), `Cargo.lock`, every crate root (one `#![forbid(unsafe_code)]` line, 03/01), `scripts/check-crate-deps.ts` and its test, `CLAUDE.md`, `docs/quality.md` |
| 04   | `crates/scanner/src/tree/`, `crates/scanner/src/matcher/`                                                      | `crates/scanner/Cargo.toml`, `crates/scanner/src/lib.rs`, `Cargo.lock`                                            |
| 05   | `crates/scanner/src/walk/`, `crates/scanner/src/scanner.rs`, `crates/scanner/src/state/`, `crates/scanner/benches/`, `crates/testing/src/large_tree.rs`, `crates/engine/src/claims.rs` | `crates/scanner/Cargo.toml`, `crates/scanner/src/lib.rs`, `crates/testing/src/fixture_tree.rs`, `crates/engine/src/session.rs`, `app/tauri/src/compose.rs`, `app/tauri/src/adapters.rs`, `package.json`, `Cargo.lock`, `docs/hacks/README.md`, `docs/testing.md` |
| 06   | `crates/store/src/scan_cache.rs`, `crates/store/src/envelope.rs`, `crates/scanner/src/codec/`                   | `crates/store/src/lib.rs`, `crates/store/Cargo.toml`, `crates/scanner/src/lib.rs` (06/02), `Cargo.lock`, `docs/architecture/README.md` ("Data") |
| 07   | `crates/scanner/src/rescan/`                                                                                   | `crates/scanner/src/walk/`, `crates/scanner/src/scanner.rs`, `crates/scanner/src/state/`, `crates/scanner/benches/`, `crates/engine/`, `app/tauri/src/compose.rs`, `app/ui/src/ipc/bindings.generated.ts`, `crates/e2e/`, `fixtures/` |
| 08   | `crates/scanner/src/state/largest.rs`, `crates/scanner/src/state/search.rs`, `crates/engine/src/disk.rs`      | `crates/scanner/src/state/mod.rs`, `crates/scanner/benches/`, `crates/engine/`, all docs, `CLAUDE.md`, the version files |

`crates/scanner/src/lib.rs` and `Cargo.toml` are touched by 04 and 06/02 in the same wave: each adds its own `mod`
line and nothing else; the second merge takes both. `Cargo.lock` is never merged by hand: on a conflict take either
side, regenerate it (`cargo metadata --format-version 1 > /dev/null`) and run the checks.

## Ideas this touches

[Scanning](../../ideas/architecture/scanning.md) and
[snapshots and watch mode](../../ideas/architecture/snapshots-and-watch-mode.md) (this plan takes proposed decisions
1–8, 11 and replaces 9: a compact file instead of an SQLite `items` table), [macOS specifics](../../ideas/architecture/macos.md)
(cloud files, clones, the Full Disk Access error), [storage history](../../ideas/storage-history.md) (built later on
the cache generations), the [large files finder](../035-large-files/readme.md) (the kept file records), the
[duplicate finder](../../ideas/duplicate-finder.md) (sizes, links and clone ids are already in the tree), the
[storage explainer](../../ideas/storage-explainer.md) (one claim per path).

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 %.
- In the app (`npm run tauri dev`), "Scan" walks the configured roots with the real scanner; findings stream from the
  first second; a second "Scan" reads only the changed folders; after a restart the last result shows at once,
  labelled with its date.
- The scanner gives the same matches as `NaiveWalk` on every fixture home and on random trees.
- The deterministic bars of the design (section 13) are held by tests; the timing bars by `npm run bench`, with the
  numbers written in the conclusions.
- The data folder stays within its 50 MB budget after repeated scans of the `large` tree; no file in it is ever
  deleted by the app.
- No crate but `fresher-macos` contains `unsafe`; the crate guard fails a crate root without `#![forbid(unsafe_code)]`.
- The hacks "No disk walk until plan 004" and "CACHEDIR.TAG read by the rules" are gone; the stand-in avoid list is
  recorded until plan 006.

## Conclusions

The plan is ✔️ done: everything is built, tested and measured by the agent. On 2026-11-16 the user asked to merge it
without the run by hand, which is [deferred](../../deferred/fast-scan-home-run.md); the checklist below stays for it. The overview's `found` total adds each finding's counted size, like the category
totals, so a path claimed twice or nested in another finding counts once (decided when plan 003 was taken in; 003/05/01
said "every finding summed").

### What was built

- **The OS boundary** (03): `fresher-macos`, the only crate with `unsafe`, all of it in `mod ffi`, one operation per
  block with its `// SAFETY:` comment; the workspace lint `unsafe_code = "deny"`, `#![forbid(unsafe_code)]` at every
  other crate root, checked by the crate guard. It gives the bulk folder read (`MacFolderReader`), volumes
  (`MacVolumes`), the change journal on FSEvents (`MacChangeJournal`), the IO policy (cloud files never downloaded),
  the peak memory, and the standing list of places that prompt.
- **The core's contracts** (02): the third size (`Sizes::reclaimable`), `Match::marker` with the pure `CACHEDIR.TAG`
  check (the rules no longer read a file), the OS ports, `DiskState`, `ScanCache`, the new `DiskWalk` with
  `WalkRequest`, `WalkMode`, `commit` and `ScanKind`; their fakes in `fresher-testing`.
- **The scan** (04, 05): a compact arena tree (72 bytes a node, files under 1 MiB folded into their folder, hard links
  and clones counted once), a compiled matcher (one lookup an entry for every module's interests), a worker per core
  (at most 16) on one queue with one aggregator, boundaries (one device per root, nested roots dropped, links never
  followed), errors kept by kind, the claims and the committed `TreeState`. It gives the same matches and summary as
  the reference walk on every fixture home and on random trees, with the portable and with the macOS reader. The app
  scans with it.
- **The cache** (06): four fixed slot files in the data folder (two scopes, two generations, rotated by rename, never
  deleted), a checked envelope around the tree body compressed with deflate level 1, a file capped at 8 MiB on disk,
  small unclaimed subtrees collapsed to fit; a broken or foreign file means one full scan, never a wrong result.
- **Rescan and restore** (07): a scan reads only the folders the journal names (with their parents and the unreadable
  ones), copies the rest, reads a sample of 32 copied folders, and scans in full on any doubt, with the reason in
  `WalkSummary::fallback`. At launch the last state is restored from the cache without reading a folder.
- **Queries** (08): the largest files and search by name in memory, and the engine's read-only API over the last
  state for plans 003, 006 and 012; no IPC command yet.
- **Measuring**: the large-tree generator, `npm run bench` (criterion, by hand), the deterministic bars as tests in the
  hook, the footprint test of ten scans.

### Measurements

From the budget run ([08/03](./08-queries-and-finish/03-budget-run.md), 2026-11-16). The machine: Apple M3 Pro, 12
cores, 18 GiB, internal SSD, APFS, macOS 27.2.1, on power, release builds, `kern.maxvnodes` 263 168. Other work ran on
the machine: "Load" is the load average during the run. Criterion medians. `LARGE` is the generated tree of 3 million
entries, `SMALL` of 100 000.

| What (design, section 13)                     | Bar                 | Measured                                                                                                                       | Load | Result                 |
| --------------------------------------------- | ------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ---- | ---------------------- |
| full scan, warm, `large` (3 M entries)        | ≤ 10 s              | `LARGE` cannot be warm here (3 M entries, 263 k vnodes); warm `SMALL`: 289 ms at 12 threads, 345 000 entries/s → 8.7 s for 3 M | 6–14 | pass, projected        |
| full scan, cold, `large`                      | none (05)           | 22.8 s at 12 threads (the default), 15.7 s at 24                                                                               | 6–14 | recorded               |
| full scan, `small`, by threads                | —                   | 1: 380 ms; 12: 289 ms; 24: 373 ms; 36: 357 ms; 48: 369 ms                                                                      | 7–14 | recorded               |
| rescan, nothing changed, `large`              | ≤ 0.5 s             | 32 ms                                                                                                                          | 5–7  | pass                   |
| rescan after 1 000 changed folders, `large`   | ≤ 1 s               | 231 ms                                                                                                                         | 5–7  | pass                   |
| restore: cache → matches emitted, `large`     | ≤ 0.2 s             | 12.7 ms (without the engine's part)                                                                                            | 5–7  | pass                   |
| search by name, `large`                       | ≤ 50 ms             | common 4.0 ms, rare 4.5 ms, one letter 12.9 ms; the first (builds the 16.2 MB copy) 35 ms                                      | 5–7  | pass                   |
| largest files, `large`                        | —                   | 9.1 ms                                                                                                                         | 5–7  | recorded               |
| peak memory of a full scan, `large`           | ≤ 200 MB            | 143 MB at 12 threads; **402 MB at 24**; 226 MB with the tree kept, encoded and fitted                                          | 6–14 | pass at the default    |
| tree heap per node                            | ≤ 96 B              | the test passes; **104 B** on `LARGE` with every `*.tmp` file matched (587 637 nodes)                                          | 5–7  | **fails on that tree** |
| cache body per node, uncompressed             | ≤ 32 B              | 24 B (`LARGE`, 114 478 nodes, body 2.77 MB)                                                                                    | 5–7  | pass                   |
| one cache file / the whole cache              | ≤ 8 / 32 MiB        | 1.73 MB a file; `LARGE` with every `*.tmp` kept does not fit and is not saved                                                  | —    | pass                   |
| data folder after ten scans of `large`        | ≤ 50 MB             | 1.73 MB after the first scan, 3.46 MB after each next; no stale temporary file                                                 | 8–12 | pass                   |
| folders read by a rescan with nothing changed | sample + unreadable | held by the tests of 07/03 in `npm run pre-push`                                                                               | —    | pass                   |

Earlier numbers worth keeping: the matcher 28 ns an entry idle (04), ≈ 50 ns under load (05); a full scan of `LARGE`
with one thread ≈ 245 s (05); rescanning half the folders of `LARGE` 19 s against a 25 s full scan (07); the
developer's home in 01: 4.7 million entries in a 22 s warm full scan, a tree of 556 000 nodes, a cache body of 13.6 MB
(5.5 MB compressed). The home-folder numbers of the final code come from the user's run below.

### Decisions

| Date       | Decision                                                                                          | Status                                     |
| ---------- | ------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| 2026-11-15 | a super lightweight cache: small, capped, safe to lose                                            | the user's                                 |
| 2026-11-15 | macOS first, Windows not ruled out: every OS call behind a port of the core                       | the user's                                 |
| 2026-11-15 | `unsafe` only in `fresher-macos`'s `mod ffi`; `deny` in the workspace, `forbid` elsewhere         | the user's                                 |
| 2026-11-15 | the plan approved with every agent's call of the design                                           | the user's                                 |
| 2026-11-15 | the cache compressed with deflate level 1 (`miniz_oxide`)                                         | the user's                                 |
| 2026-11-16 | the 8 MiB cap is on the compressed file on disk, the body bounded at 64 MiB before compression    | the agent's reading, confirmed by the user |
| 2026-11-16 | the 10 s full-scan bar is on a warm tree; a cold full scan is recorded with no bar (05)           | taken as recommended, to check             |
| 2026-11-16 | one thread per core, at most 16: faster warm, within 200 MB; two per core took 402 MB (05, 08/03) | taken as recommended, to check             |
| 2026-11-16 | the tree-heap bar (96 B) stays on its test; 104 B a node on a tree of many matched files recorded | taken as recommended, to check             |

The three "to check" are confirmed or reopened by the home-folder numbers of the checklist (items 1, 2 and 7); whether
a cold-scan target is wanted is the user's question.

### Deviations worth knowing

Each is recorded in its step as an agent's call.

- **From measurement (01)**: one thread per core instead of 3 × the cores; the private size read only for a file that
  may share blocks (asking it for every file doubled the walk); small hard-linked and cloned files folded, their
  identity kept in the link and clone tables; a journal replay bounded by age (14 days) and time (11 s), because a day
  of history took 77 s to replay.
- **The tree** (04): a node holds one set of sums (its own bytes until `freeze`, its subtree's after) to stay under the
  heap bar; a clone group counts the largest shared part of its members.
- **The scanner depends on the core alone**: it reads through `FolderReader`; the app injects the macOS reader.
- **The cache** (06, 07): the cap is on the compressed file, with a separate body limit; a tree whose matched files are
  kept everywhere may not fit and is then not cached (one full scan next time, never a wrong result).
- **The rescan** (07): the parent of a changed folder is read too (the tree keeps no time per folder); the check's
  seed comes from the clock.
- **The reference walk** (05) was fixed: it compared marker names through the file system (`cachedir.tag` taken for
  `CACHEDIR.TAG` on a volume that ignores case).
- **The warm-scan bar** is projected from `SMALL`: `LARGE` does not fit the kernel's metadata cache on this machine.
- **The step's name check** (08/04) still finds the `NoWalk` test stubs and the reference walk's `holds_valid_tag`:
  test code only, kept.

### Known issues

- [A FIFO is read as a regular file](../../known-issues/fixed/core-fifo-read-as-file.md) (open): the core's entry kinds have
  no "other", so a named pipe with a marker's name counts as that marker for the matcher (the walker's tag read
  refuses it, so a `CACHEDIR.TAG` FIFO is never `Valid`). The fix is a core contract, for a later plan.
- [Stale temporary files in the data folder](../../known-issues/store-stale-temporary-files.md) (open, from plan 002):
  the cache writes the same way; the footprint run found none.

### By hand (the user)

Read-only, on the developer's home folder; counts, times and sizes only, no path. Items 1–7 are 08/03's (fill them in
there); items 8–9 are the plan's acceptance.

1. `npm run build`, the built app, the first scan of the home: time, peak memory (Activity Monitor), the two cache file
   sizes, the footprint (`DataFolder::footprint()` debug log line), the tree heap per node if logged.
2. If the thread count is to be settled: the same scan with two threads per core (a build changed by hand): time,
   peak memory.
3. A rescan after a minute: time, changed folders, fallback if any.
4. A rescan after a day of ordinary use: time, changed folders, fallback if any.
5. Quit and start the app: the last result shows at once, labelled with its date; the time.
6. Clones: the scan's total allocated bytes against `du -sk` of the same roots: within the bar, no shared block
   counted twice or lost.
7. Once after a reboot, before anything else reads the disk: the first scan, cold (recorded with no bar).
8. `npm run tauri dev`: "Scan" walks the configured roots with the real scanner and findings stream from the first
   second; a second "Scan" reads only the changed folders; after a restart the last result shows at once.
9. The decisions above marked "to check": confirm or overrule.

### Left to later plans

- **006 (permissions)**: filling `Match::facts` (the facts 006 adds to a match: owner, a writable parent, the file
  flags) from the bulk read; the guarded places, which replace the standing avoid list (hack "Places that prompt are
  avoided until plan 006") and feed the request's `avoid`; the access probe (`UnknownAccess` until then); the runs 01
  could not do from a terminal (which places prompt, cloud files, removable and network volumes, the journal with Full
  Disk Access and after a sleep, macOS 13 to 26); `unreadable_places` as its screen's data.
- **003 and 012**: the IPC commands for the queries with the screens that use them; `ScanKind` shown ("restored from
  …").
- **009**: `KnownFolders` for anchored interests (the home is the only anchor until then); the `Volumes` port extended.
- **010**: history and the diff of two scans, on the cache's two generations.
- **005**: the cache in the settings' footprint and clearing it.
- **Later**: an "other" entry kind in the core (the FIFO issue); a cold-scan target, if the user wants one; a
  cancellable `CommandRunner` with the first module that calls a tool.
