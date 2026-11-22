# 012 — The storage map: design

**Status:** ✅ approved (by delegation, 2026-11-17) · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
macos, scanner, engine, testing, sandbox, app, app/ui

Expanded on 2026-11-17 from the one-file draft of 2026-11-15. The user asked the agent to work on its own and accept
the recommended decisions: every open question is answered with the recommended option and recorded under
"Decisions"; the agent's calls wait for the user's check.

## Intent

The first screen must answer "where did my space go" in one look ([`goal.md`](../../goal.md), sections 1 and 6). That
needs one model of the whole disk into which every module's findings fit, whose parts add up to what the Mac itself
reports, and which has no "Other".

## Goals

1. Every byte of the volume sits in exactly one named place; the parts add up to the space the volume shares.
2. Two views of the same numbers: by meaning (apps and categories) and by place (folders).
3. Space no module recognizes is shown as its real folders.
4. Space that is not files of the scan — the system's volumes, swap and the sleep image, the purgeable space — is
   named and explained, never offered as junk.
5. What the app could not measure is said plainly, with the reason.
6. The volume facts of plan 009's [deferred item](../../deferred/volume-facts.md): each volume's kind, free and
   purgeable space, with this plan as their first consumer.

## Non-goals

- The treemap and other rich charts (the [large files finder](../035-large-files/readme.md)); this plan's
  place view is the list such a chart would sit on.
- Cleaning anything from the map's system parts; the map adds no category and no removal.
- Listing local snapshots with their dates and sizes, and thinning them (decision 5): the
  [system data analyzer](../../ideas/system-data-analyzer.md)'s own module, later.
- Growth since the last scan: plan [010](../010-history/readme.md) adds it to the by-meaning view when it exists.
- What waits in the Trash from this app's own cleanups (the journal's share): the
  [Trash idea](../037-trash/readme.md); this plan only names the Trash's folder as a place.

## What already exists (plans 003, 004, 008, 009)

| Need                         | What exists today                                                                                                     | This plan                                                                         |
| ---------------------------- | --------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| The tree of a scan           | `DiskState` (`crates/core/src/disk_state.rs`): nodes with `Sizes` (logical, allocated, reclaimable, completeness), `owner`, `not_scanned`; `children` by size; `unreadable` | two queries more: the owned nodes, a node by its path (01/02)                     |
| One owner per byte           | the engine's claims (`crates/engine/src/claims.rs`): the deepest finding owns its subtree; the tree's `owner` per node | the map's shares follow the same rule on the tree's own sizes (01/03)             |
| Hard links and clones once   | the scanner's aggregator counts each once (plan 004, section 4)                                                       | reused: every size of the map comes from the tree                                |
| Volume facts                 | the `Volumes` port: `identify` (id, device, case, this volume's `used`), `available`; `MacVolumes`, `FakeVolumes`     | `Volumes::space` (02/01): kind, capacity, free, purgeable, the volumes sharing it |
| Every mounted volume         | `fresher-macos`'s `ffi::volume::mounts` (`getfsstat`, no volume asked)                                                | reused to find the volumes that share one disk (02/03)                           |
| A Foundation call            | `objc2`, `objc2-foundation` (`NSURL`, `NSFileManager`) in `fresher-macos`'s `ffi::trash`                               | one more call: two resource values of the volume (02/02)                          |
| Installed apps, known folders| plan 009's `SystemFacts` (`cx.system()`), one per scan                                                                | the app's data folders attributed to it (01/04)                                   |
| The overview's numbers       | `get_overview` (totals per category, `found`), refreshed at most 4 times a second during a scan (`state/refresh.ts`)  | the map is asked on the same beat (04/02)                                         |
| Bars and lists               | `UsageBar`, `StorageRing`, `Segment` (danger levels only), `TreeRow`, `ListRow`, `VirtualList`, `CategoryGrid`        | `Segment` gains neutral tones (04/03); the views reuse the rest                  |

## Design

### 1. The parts of a volume

The map of one volume is drawn over the **space that volume shares**: its capacity (`f_blocks`), which on APFS is the
whole container. Measured on the developer's Mac: every volume of the container reports the same capacity and the same
free space, and its own used space (`/` 13 GB, Preboot 9.5 GB, Data 381 GB, of 483 GB with 77 GB free), so the
volumes beside the scanned one are measured, not guessed.

| Part                     | What it is                                                                                         | From                                   | Cleanable                   |
| ------------------------ | -------------------------------------------------------------------------------------------------- | -------------------------------------- | --------------------------- |
| Explained                | the findings of the modules, grouped app → category → item                                          | the tree, the owners                   | by its danger level         |
| Your folders             | everything scanned that no module claimed, as the real folder tree                                  | the tree                               | no suggestion               |
| Not scanned yet          | during a scan only: this volume's used space the scan has not reached                               | the volume, the findings so far        | —                           |
| The remainder            | this volume's used space the scan did not measure, with its reasons (section 3)                     | the volume minus the tree              | —                           |
| macOS                    | the system's volumes that share the disk: the system, Preboot, Update and the like                  | the neighbours' used space             | never                       |
| Swap and sleep image     | the virtual memory volume: swap files and the sleep image                                          | the neighbour's used space             | never; information          |
| Other volumes            | another volume of the user's on the same disk, by its own name                                      | the neighbour's used space             | — (scan it to see inside)   |
| The disk's own space     | capacity minus free minus every mounted volume's used: unmounted volumes (Recovery) and bookkeeping | arithmetic, never below zero           | never                       |
| Free                     | the space the user can still write (`f_bavail`)                                                    | the volume                             | —                           |

- **Purgeable space is not a part** (decision 6, call 7): it lies inside the used space and overlaps files of the
  scan (cached cloud files, some caches) and local snapshots. It is shown with Free: "Free 77 GB · macOS can free about
  30 GB more on its own when space runs low". Never offered as junk.
- **One claim per byte.** The owners of plan 004 decide every folder's place; a nested claim is subtracted from the one
  around it (the deepest owns, as the engine's claims); hard links and clones are counted once by the tree. A property
  test holds the invariant: the leaves add up to the total, in both views (01/03, 01/04, 02/04).
- **Which number.** The headline is the space taken on disk (`allocated`). "Freed if removed" is a separate number, from
  the private size (`reclaimable`), because clones and snapshots make the two differ. The logical size is detail on
  demand.
- **Incomplete is marked**: a part with unreadable folders inside shows `42 GB+` (the tree's completeness).
- **Ignored findings are your folders** (decision 7): the engine leaves them out of every answer, so the map shows
  their bytes as unclaimed folders, without a suggestion; restoring them moves them back without a rescan. Amended
  2026-11-17: an ignored finding that names an app stays in that app's data (unclaimed, not offered), since the app's
  folder still holds it.
- **Findings that are not about a path** (a tool's object, a Docker image) are not added to the map: their bytes sit
  inside a file the tree already counts. Their category lists them as detail (call 6).

### 2. The volumes of a map

- **One map per volume that holds a scan root**, the home's volume first; roots on one volume share one map. Two
  volumes of one disk that both hold a root each get their own map and show the other as a neighbour (call 23).
- **Neighbours**: every other mounted volume whose device is a slice of the same disk (`/dev/disk3s1s1` and
  `/dev/disk3s5` share `disk3`), with a role told by its mount point in the adapter: `/` is the system, the volume at
  `/System/Volumes/VM` holds swap and the sleep image, any other under `/System/Volumes/` is the system's (Preboot,
  Update), anything else is another volume of the user's (call 9). A volume on another disk is not a neighbour.
- **Kind**: `Network` when the mount is not local (`MNT_LOCAL` off), `Internal` when the system says the volume is
  internal, `External` otherwise; a mounted disk image reads as `External` (call 10). The kind names the map ("Macintosh
  HD · internal") and says when purgeable space is not asked (a network volume).
- **When it is read**: at the start of a scan, at its commit and at the launch's restore — never per query (call 18);
  the volume facts change from moment to moment, and the map is asked four times a second during a scan.

### 3. The remainder and its reasons

The remainder is this volume's used space minus what the scan measured on it: never folded into another part, never
hidden.

- **Its name depends on the roots** (call 12). When the scan roots on the volume do not cover its mount point (the
  default root is `~`), it is **"Outside the scanned folders"**: "apps, the system's data and other users' files, which
  the scan did not look at. Add the disk in Settings → Scan to see inside." When they cover it, it is **"Not
  measured"**.
- **Its reasons are listed under it**, each a plain line, none a category: the folders the app could not read (their
  count, and the permission that would show them, from `unreadable` and the access status of plan 006); local
  snapshots and the file system's own bookkeeping; what changed on the disk during the scan.
- **A negative remainder** (the scan counted more than the volume reports: files written during the scan) is shown as
  zero, and the map says once that the disk changed during the scan and a new scan gives exact numbers; the parts are
  then drawn over their own sum (call 12). Only a complete scan says so: during a scan the findings so far may run past
  the used space without anything having changed.

### 4. Two views

- **By meaning** (01/04): the apps, largest first — an app's line is the findings attributed to it plus its data
  folders that no module claims (by its bundle id under the known folders of app data: caches, support, containers,
  group containers, preferences, logs, saved state) and its bundle when a root holds it; then the explained categories
  not tied to an app (the existing category cards, with their selection); then the named places of your folders (the
  Trash, call 15) and the largest of your folders; then the system's parts, the remainder and Free.
- **By place** (03/01): the folder tree, the largest first, from a root. Each row shows how much of it is explained
  ("12 GB, 9 GB of it explained"). Opening a folder **descends automatically** through a folder that is almost entirely
  one child (90 % of its allocated size, at most 8 levels), shown as one path ("Library › Containers ›
  com.example.app"). A node of another state is `StaleNode`, as every disk query.
- **Switching the view never changes the total**: both are built from one `Shares` of the state (section 5).

### 5. Where the code lives

- **`fresher-core`** (`crates/core/src/storage/`, new): the map's types — `StorageMap`, `VolumeMap`, `MapPart`,
  `PartKind`, `RemainderReason`, `MapPhase`, `MeaningGroup`, `PlaceView`, `PlaceRow` — and their texts in
  `locales/en.json` (`core.storage.*`). In `volume.rs`: `MountKind`, `VolumeSpace` is taken by the journal, so
  `DiskSpace`, `Neighbour`, `NeighbourRole`, and `Volumes::space`. In `disk_state.rs`: `DiskState::owned` (every owned
  node with its category) and `DiskState::find` (a node by its path). `NewFinding::for_app` (a finding names the app it
  belongs to).
- **`fresher-scanner`**: `owned` and `find` on `TreeState` (the tree's `lookup` exists).
- **`fresher-macos`**: `ffi::volume::resources` (the Foundation call) and `MacVolumes::space`.
- **`fresher-engine`** (`crates/engine/src/storage/`, new): `shares.rs` (the owned bytes per category and per folder,
  once per state and ignore lists), `meaning.rs` (the by-meaning groups), `volume.rs` (a volume's parts and the
  remainder), `place.rs` (the by-place view and the descent), `query.rs` (`Engine::storage_map`,
  `Engine::storage_meaning`, `Engine::storage_place`). The map reads the `Volumes` the engine already holds in `CleanupPorts` (call 11): no change
  to the composition root.
- **`fresher-testing`**: `FakeVolumes::with_space`; `FixedDiskState` answers `owned` and `find`.
- **`fresher-app`**: `app/tauri/src/storage_commands.rs` (new): `get_storage_map`, `get_storage_meaning`,
  `get_storage_place`; one `mod` and three handler lines in `lib.rs`, the grants in `build.rs` and the capability
  (call 19).
- **`app/ui`**: `Segment` tones (`design/data/segment.ts`), `product/storage-map/` (the bar, its legend and parts),
  `product/meaning-list/`, `product/place-list/`, `state/storage.ts`, and the overview screen.
- **`fresher-sandbox`** (plan 007): an optional `[case.map]` expectation; `fixtures/sandboxes/storage-map.toml`.

```rust
pub struct StorageMap { pub phase: MapPhase, pub volumes: Vec<VolumeMap> }
pub enum MapPhase { NoScan, Scanning, Complete { state: StateId } }
pub struct VolumeMap {
    pub name: String, pub kind: MountKind,
    pub capacity: ByteSize,                 // the whole the parts are drawn over
    pub parts: Vec<MapPart>,                // in a fixed order; their sizes add up to `capacity`
    pub purgeable: Option<ByteSize>,        // shown with Free; not a part
    pub roots: Vec<NodeId>,                 // where the place view starts (empty before a commit)
    pub disk_changed: bool,                 // a negative remainder: the disk changed during the scan
}
pub struct MapPart { pub kind: PartKind, pub sizes: Sizes }   // `reclaimable` is "freed if removed"
pub enum PartKind {
    Explained, YourFolders, NotScannedYet,
    Remainder { outside_scan: bool, reasons: Vec<RemainderReason> },
    MacOs, SwapAndSleep, OtherVolume { name: String }, DiskOwn, Free,
}
pub enum RemainderReason { Unreadable { folders: u64, needs: Option<Capability> }, SnapshotsAndBookkeeping, ChangedDuringScan }
```

### 6. Live filling

The tree exists only after the commit, so the map has three phases (call 17):

- **`NoScan`** (before any state): each volume of the configured roots, its used space as "Not scanned yet", the
  neighbours, the disk's own space and Free.
- **`Scanning`**: "Explained" is the scan's findings so far, by the engine's counted totals, each finding on the volume
  of its root; "Not scanned yet" is the rest of the volume's used space; the volume parts as above. The bar fills as the
  findings stream in, and the sum always holds.
- **`Complete`**: the full map of the committed state. The restore at launch commits the cached state within a second,
  so the full map is there at once on every launch after the first.

No new event: `findings-added` and `scan-finished` already mark the overview stale, and the refresher asks the map
with the overview, at most 4 times a second.

### 7. Speed and memory (rule 4)

- `Shares` is built once per state and ignore lists: one pass over the owned nodes (thousands, not millions) and their
  ancestors; kept beside the state in the engine and dropped with it. Its build time and heap are measured on the
  developer's home in 03/02 and written in the conclusions; the budget is 50 ms and 5 MB.
- A place page is one `children` call (at most 500 rows) plus one lookup per row in `Shares`.
- The volume facts are three system calls and one Foundation call, read at most once per scan phase.

### 8. The sandbox case file

Plan [007](../007-safe-development/readme.md) (built in parallel) asks every feature for a case file
`fixtures/sandboxes/<feature>.toml`. The map is a core feature, like `cleanup-guards`: its file is
`storage-map.toml`, on the `trial` module. A case may carry an optional `[case.map]` expectation (05/01):

```toml
[[case]]
name = "a folder no module claims is shown as your folder, never explained"
kind = "negative"
[[case.file]]
path = "home/Movies/holiday.mov"
size = 41943040
[case.map]
part = "your-folders"          # your-folders · explained
at = "home/Movies"             # the folder the place view shows it under
```

The harness checks, after the scan and before the cleanup, that the place view puts the path in that part, and that
the map's leaves add up; the golden report gains the map with the fake volume's numbers. Built after plan 007 is merged
into `main` (decision 8).

### 9. Interfaces with other plans

| Plan | 012 needs from it                                                            | 012 offers it                                                       |
| ---- | ---------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| 004  | the tree, the owners, the three sizes, the unreadable places                 | the first screen that reads the tree                                |
| 006  | the access status: which permission shows an unreadable folder               | the remainder's reasons name it                                     |
| 007  | the case format, the harness, the rule of a case file per feature            | `[case.map]`, `storage-map.toml`                                    |
| 009  | `SystemFacts`: known folders, installed apps; the deferred volume facts       | the first consumer of the facts and of `InstalledApp::owns` for data |
| 010  | —                                                                            | the by-meaning groups to compare between scans                      |
| 014  | —                                                                            | "the scan fills the overview live" (section 6)                      |

## Open questions

None left: every question was delegated and decided below.

## Decisions

Taken by delegation on 2026-11-17 (the user's instruction: accept the recommended decisions):

1. **The plan is approved** as designed here, with the agent's calls below still to check.
2. **"Not readable" and "Not measured" are shown as honest lines with their reasons** (the draft's call 1): nothing is
   classified into them, they are not a category, and they are never offered for cleaning. The unreadable folders are
   one of the remainder's reasons, with their count and the permission that would show them; their bytes are in the
   remainder, since an unread folder has no size.
3. **No version bump**: the version stays 0.1.0 ([`../README.md`](../README.md), "Versions"); this replaces the
   draft's call 6 (0.5.0).
4. **The volume facts of plan 009 are taken here** ([deferred](../../deferred/volume-facts.md)): kinds `Internal`,
   `External`, `Network`; free; purgeable from the Foundation resource value; the capacity and the neighbours added,
   because the map's whole needs them. A disk image is not told apart (no plain call says it; nothing needs it yet).
5. **Local snapshots are not listed in this plan**: their sizes need administrator rights and their list needs a tool;
   the map names them among the remainder's reasons and in the purgeable note. Listing and thinning them stay with the
   [system data analyzer](../../ideas/system-data-analyzer.md). This replaces the draft's "each its own named line"
   for snapshots.
6. **Purgeable space is a note on Free, not a part**: it overlaps the scanned files and the snapshots, so adding it
   would count bytes twice. This replaces the draft's "Kept by macOS" part; swap and the sleep image keep their own
   line, measured from their volume.
7. **Ignored findings count as your folders** in the map, without a suggestion. Amended 2026-11-17: one that names an
   app stays in that app's data, unclaimed, as the by-meaning view groups the app's folders whole.
8. **The case file step (05) is built after plan 007 is merged into `main`**: this branch merges `main` first; until
   then the other plans are built and the map is checked by the engine's tests.

## Agent's calls (to check)

1. **The headline is the space taken on disk** (`allocated`); "freed if removed" (`reclaimable`) is separate; the
   logical size is detail (the draft's call 2).
2. **Two views over one model**, with the same total (the draft's call 3).
3. **Unclaimed space carries no cleanup suggestion** (the draft's call 4).
4. **The system's parts are information only** (the draft's call 5).
5. **After the commit every size comes from the tree**, not from the findings' own sizes; during a scan "Explained"
   is the engine's counted totals, replaced at the commit.
6. **Findings not about a path are not added to the map**: their bytes are inside a file the tree counts.
7. **Purgeable is shown with Free** (decision 6), as "about": its answer changes from moment to moment.
8. **The map's whole is the capacity the volume shares**, so the system's volumes beside it are measured parts.
9. **Neighbours by device and roles by mount point** in the adapter (`fresher-macos`), never in the engine.
10. **`MountKind`** is the new enum's name: `VolumeKind` is already the access capability's (removable, network).
11. **The map reads the `Volumes` already in `CleanupPorts`**: no change to `compose.rs`; an engine without cleanup
    ports (some tests) answers the map without volume facts (the scanned parts only, `capacity` their sum).
12. **The remainder is named by the roots** ("Outside the scanned folders" or "Not measured"), lists its reasons, and
    a negative remainder is shown as zero with a note.
13. **Auto-descent at 90 %**, at most 8 levels per opening.
14. **App attribution**: a finding may name its app (`NewFinding::for_app`); an unclaimed first-level folder of an app
    data known folder belongs to the installed app that owns its name (`InstalledApp::owns`); an app's bundle counts
    when a root holds it. Worked out at the commit with the scan's `SystemFacts` and kept with the results. This
    overrides plan 009's "a fact is gathered only when something asks": the installed apps are listed once per scan, at
    the commit, even when no module asked.
15. **The Trash is a named place** of your folders (`KnownFolder::Trash`), with "you get this space back when you empty
    the Trash"; never cleanable.
16. **`Shares` once per state and ignore lists**, kept in memory with the state, measured (section 7).
17. **Three phases and no new event**: the overview refresher asks the map too.
18. **Volume facts read at scan start, commit and restore**, not per query.
19. **The commands in a new file** `storage_commands.rs`, so plan 007's `commands.rs` and `compose.rs` stay untouched.
20. **By meaning reuses `CategoryGrid`** for the explained categories, with their selection; the app groups come
    first.
21. **`Segment` gains neutral tones** (`system`, `free`, `plain`, `unknown`) beside the danger levels.
22. **`[case.map]` is an optional table** of 007's case format, not a second format.
23. **One map per volume holding a root**; two volumes of one disk show each other as neighbours.

## Conclusions

Not written yet.
