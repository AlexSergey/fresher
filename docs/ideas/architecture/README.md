# Architecture ideas

**Since:** 2026-11-15

How the best open-source disk analyzers, duplicate finders, cleaners and uninstallers are built, and the decisions we
propose to take from them. Clean room: techniques only, no project named, no code taken
([`../../restrictions/repository.md`](../../restrictions/repository.md), "Clean room"). Collected on 2026-11-15 from
their source and documentation; items marked **unverified** come from platform documentation or memory and must be
checked by the plan that takes them up.

These are ideas, not decisions: each becomes a decision only in the design of the plan that implements it.

| Topic                                                             | From  | What                                                                                                                 |
| ----------------------------------------------------------------- | ----- | -------------------------------------------------------------------------------------------------------------------- |
| [Scanning](./scanning.md)                                         | user  | the walker, sizes, hard links, APFS clones, errors, memory layout                                                    |
| [Snapshots and watch mode](./snapshots-and-watch-mode.md)         | user  | the SQLite snapshot shape, diffing two scans, fast rescans                                                           |
| [Duplicates](./duplicates.md)                                     | user  | the staged hashing pipeline, hash cache, clones, similar images                                                      |
| [macOS specifics](./macos.md)                                     | user  | dataless files, purgeable space, snapshots, Full Disk Access, the Trash, leftovers                                   |
| [Rules as data](./rules-as-data.md)                               | user  | declarative rules, project artifacts, cache markers, protected paths                                                 |
| [Crates](./crates.md)                                             | user  | candidate dependencies and their licences                                                                            |
| [Safety in the types](./safety-in-types.md)                       | agent | additions to plans 002 and 008: evidence on findings, a consumed `Verified`, a token for permanent removal           |
| [Change detection and hashing](./change-detection-and-hashing.md) | agent | measured walk timings and clone behaviour; rescan and duplicate options with their verdicts; md5 nowhere             |

## Proposed decisions

1. **A macOS-only walker on `getattrlistbulk(2)`** in `fresher-scanner`: one syscall per batch returns name, device,
   inode, link count, allocated sizes and the APFS clone id — no `lstat` per entry ([scanning](./scanning.md)).
   Measured: asking for the clone id and private size makes the walk 2–3 × slower (cold 24.9 s → 44.4 s+) — see
   [change detection and hashing](./change-detection-and-hashing.md).
2. **Work-stealing scheduling** (`crossbeam-deque`), about 3 × the cores, capped at 64 workers; `rayon` is acceptable for
   a first version ([scanning](./scanning.md)).
3. **Turn off materialisation of dataless files at startup** (`setiopolicy_np`), so walking iCloud Drive or other
   cloud folders never downloads anything — required by our no-network rule ([macOS](./macos.md)).
4. **Three sizes per file**: logical, allocated, and private (exclusive) size; "freed if deleted" comes from the
   private size ([scanning](./scanning.md)).
5. **Hard links counted once**: track `(device, inode)` only when a file's link count is above 1 ([scanning](./scanning.md)).
6. **APFS clone groups counted once** per `(device, clone id)` in totals ([scanning](./scanning.md)); a partly
   rewritten clone gets a new clone id — see [change detection and hashing](./change-detection-and-hashing.md).
7. **Stay on one device by default** and reject nested or overlapping roots (firmlinks) ([scanning](./scanning.md)).
8. **An arena tree in memory**: 32-bit indices, one shared name buffer, about 64 bytes per node ([scanning](./scanning.md)).
9. **Snapshots in SQLite** as an `items` table keyed by parent and name, bulk-inserted ([snapshots](./snapshots-and-watch-mode.md)).
10. **Watch mode diffs two snapshots by path**: added, removed, changed, with the size change rolled up to folders
    ([snapshots](./snapshots-and-watch-mode.md)).
11. **Scan errors collected by kind** and shown honestly: "N folders unreadable — grant Full Disk Access"
    ([scanning](./scanning.md)).
12. **Full Disk Access detected by probing** a file only readable with it, re-checked when the window gains focus
    ([macOS](./macos.md)).
13. **Duplicates in stages**: size → drop hard links and identical clone ids → 4 KiB prefix → 4 KiB suffix → full
    BLAKE3 ([duplicates](./duplicates.md)).
14. **A hash cache in SQLite** keyed by `(device, inode, size, mtime, range)` ([duplicates](./duplicates.md)); the
    agent proposes adding `ctime` ([change detection and hashing](./change-detection-and-hashing.md)).
15. **BLAKE3 for content, `twox-hash` for xxh3** — not `xxhash-rust` (BSL-1.0, outside our licence list)
    ([crates](./crates.md)); the agent proposes measuring BLAKE3 against the hardware SHA-256 of Apple Silicon first
    ([change detection and hashing](./change-detection-and-hashing.md)).
16. **Similar images optional**: perceptual hashes with a BK-tree over Hamming distance ([duplicates](./duplicates.md)).
17. **The cleaner trashes through `NSFileManager trashItemAtURL`** (via `objc2`), records the resulting path in our own
    undo log, and never drives Finder by Apple Events ([macOS](./macos.md)).
18. **Developer-artifact rules are data**: a marker file, a sibling or contents condition and a minimum age;
    `CACHEDIR.TAG` honoured ([rules](./rules-as-data.md)).
19. **App leftovers matched by bundle id, team id and the container metadata plist**, with each app's `Info.plist`
    as the primary source and Spotlight only as a supplement ([macOS](./macos.md)).
20. **Protected paths checked on physical paths** (resolved first), protecting the ancestors of protected items too,
    plus security-agent and device-management caches and an explicit list of critical system bundles
    ([rules](./rules-as-data.md)).
