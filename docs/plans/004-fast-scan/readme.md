# 004 — Fast scan and search

**Status:** 📝 draft · **Version:** 0.3.0 · **Date:** 2026-11-15 · **Modules:** macos, scanner, store, engine

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "Create a draft of plan 004 for the fast scan and search. Pick an efficient algorithm for
it, given our ideas and goals. Add caching, but a super lightweight one — so that a program fighting disk clutter does
not clutter the disk itself."

## Goals

1. The first scan is as fast as the disk allows; results appear from the first second.
2. Every later scan reads only what changed.
3. One walk serves every module: matching costs almost nothing per entry.
4. The cache is a few megabytes, capped, and safe to lose.
5. Scan time, memory and cache size are measured numbers (rule 4 of [`rules.md`](../../rules.md)).

## Non-goals

- Watch mode, history and comparing saved states: their own plan, built on this one. This plan keeps only the current
  and the previous state.
- The duplicate search (its own module; it reuses the tree built here).
- Per-process attribution ([live storage attribution](../../ideas/live-storage-attribution.md)).

## Design

### 1. The first scan

- **One syscall per batch of entries**: `getattrlistbulk` returns names, kinds, inode, link count, logical, allocated
  and private sizes and the clone id — no `lstat` per file.
- **Work-stealing workers** (`crossbeam-deque`), about 3 × the cores, capped; they send batches to **one aggregator**
  that owns the tree, so nothing is locked.
- **Interesting places first**: the folders the modules care about are queued before the rest, so categories fill in
  within the first second while the long tail is still being read.
- **Boundaries**: stays on the root's device, follows no link, rejects overlapping roots; cloud files are never
  downloaded (`setiopolicy_np` at start); every scan can be cancelled and leaves a result marked partial.
- **Honest sizes**: three sizes per item; hard links and clone groups counted once; an unreadable folder makes its
  parents partial (`42 GB+`); errors are collected by kind and shown.

### 2. The tree in memory

- **Folders are nodes; small files are folded into their folder** as a count and three sums. A file is kept as its own
  record only when it is large (1 MiB and up, to be tuned), hard-linked, or matched by a module.
- An arena with 32-bit indices and one shared name buffer.
- Result: a home of 3 million files is about 400 thousand nodes — tens of megabytes of memory, not hundreds — and
  nearly all the bytes are still attributed to named files. The small files of one folder are read live when the user
  opens it (one syscall).
- **Opaque subtrees**: a folder a module claims whole (`node_modules`, a cache) is summed but not indexed inside.

### 3. Matching (the "search" of the scan)

- Module interests are compiled once per scan into one matcher: a hash table of exact names, a table of extensions, and
  a trie of anchored paths that is walked in step with the folder descent — its state travels with the folder, so no
  path string is ever built or compared.
- Cost per entry: one hash lookup. No regular expressions, no globbing in the walk.
- **One claim per path**: when two modules want the same item, the more specific anchor wins, then the higher danger
  level; nested claims subtract from the outer one, so nothing is counted twice.

### 4. Rescans: read only what changed

macOS itself keeps a history of changed folders per volume (FSEvents); no helper of ours runs in the background.

1. The cache stores the volume's identity and the last event id seen.
2. On the next scan the app asks for the folders changed since that id, re-reads only those folders (one syscall each),
   patches the tree and pushes the differences up to the parents.
3. Modules receive only the changed matches; the UI receives only the changed findings.
4. **Any doubt means a full scan**: the history was reset or dropped events, the volume changed, the cache is missing,
   corrupt or from another version, or the patched total disagrees with the volume's used space beyond a tolerance.

Folder modification times are not used to skip folders: they do not change when a file grows in place — exactly the
case the product exists to find.

At launch the last result is shown immediately, labelled with its date; the user starts the rescan.

### 5. The cache

- **One compact binary file** per scanned volume in the app's data folder: folders in depth-first order with child
  counts (so no ids and no parent pointers are stored), names, variable-length integers, a checksum, a format version.
  Totals of subtrees are recomputed on load, not stored.
- Written atomically after a scan; no journal, no temporary files left behind.
- **Two generations only**: the current one and the previous one (for "what changed since the last scan").
- **A budget**: about 50 MB for the whole data folder by default; the oldest data goes first. The size is shown in
  Settings with a button to clear it.
- It is a cache: a missing or broken file costs one full scan, never a wrong result.

### 6. Search and queries

- Search by name runs over the in-memory name buffer — milliseconds, no index on disk.
- The tree answers through a read-only port: children of a folder, the largest items, the items of a category. Modules
  and the UI use this port; nobody reads the cache file directly.

### Targets

Placeholders until the first plan measures them on a generated tree and on a real home folder; then they become the
budget that benches hold.

| What                                   | Target            |
| -------------------------------------- | ----------------- |
| first scan, warm, 2 million entries    | ≤ 5 s             |
| rescan, nothing changed                | ≤ 0.5 s           |
| rescan after an ordinary day           | ≤ 2 s             |
| launch to the last result on screen    | ≤ 0.2 s           |
| peak memory, 3 million entries         | ≤ 200 MB          |
| cache on disk, 3 million entries       | ≤ 15 MB           |

## Plans (after approval)

| #   | Plan                 | What                                                                                   |
| --- | -------------------- | -------------------------------------------------------------------------------------- |
| 01  | Measure first        | throwaway probes in `NO_COMMIT/`: the bulk call and its attributes, the change history and its failure modes, counts of links and clones; the numbers go into the design |
| 02  | macOS layer          | `fresher-macos`: safe wrappers for the bulk read, the IO policy, volume info, the change history |
| 03  | Walker               | workers, boundaries, errors, cancellation, progress events                             |
| 04  | Tree and matcher     | the arena, folding, opaque subtrees, the compiled matcher, claims                      |
| 05  | Cache                | the file format, atomic write, checksum, two generations, the budget                   |
| 06  | Rescan               | changed folders, patching, the fallbacks, the total check                              |
| 07  | Queries and benches  | the read port, search by name, criterion benches holding the budget                    |

Depends on plan 002 (ports, events, `fresher-store`). Independent of plan 003: the two can run in parallel.

## Ideas this touches

[Scanning](../../ideas/architecture/scanning.md) and
[snapshots and watch mode](../../ideas/architecture/snapshots-and-watch-mode.md) (this plan takes proposed decisions
1–8, 11 and replaces 9: a compact file instead of an SQLite `items` table),
[storage history](../../ideas/storage-history.md) (built later on the cache generations), the
[large files finder](../../ideas/large-files-finder.md) (the kept file records), the
[duplicate finder](../../ideas/duplicate-finder.md) (sizes, links and clone ids are already in the tree).

## Decisions

None yet.

## Agent's calls (to check)

1. **Rescans driven by the system's change history**, with a full scan on any doubt; folder modification times are not
   trusted. To verify in 004/01: how far back the history reaches and how it behaves without Full Disk Access.
2. **Small files folded into their folder**; the threshold starts at 1 MiB.
3. **A compact binary cache, two generations, a 50 MB budget** — no SQLite (see plan 002, call 1).
4. **A compiled matcher and one claim per path.**
5. **Interesting places are scanned first.**
6. **No watching while the app is open**: a rescan starts only when the user asks.
7. **Search by name stays in memory**, with no index on disk.
8. **Compression of the cache is decided by measurement** in 004/01 (none, or a pure-Rust one).
9. **Target version 0.3.0.**

## Conclusions

Not written yet.
