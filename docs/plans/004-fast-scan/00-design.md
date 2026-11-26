# 004 — Fast scan and search: design

**Status:** ✅ approved · **Version:** 0.3.0 · **Date:** 2026-11-15 · **Modules:** core, macos, scanner, store, engine,
testing, app

## Intent

The user's task (translated): "Create a draft of plan 004 for the fast scan and search. Pick an efficient algorithm for
it, given our ideas and goals. Add caching, but a super lightweight one — so that a program fighting disk clutter does
not clutter the disk itself."

Then, on expanding the draft: "Take the recommended decisions yourself." Every such decision is listed under
"Agent's calls (to check)" at the end.

## Goals

1. The first scan is as fast as the disk allows; results appear from the first second.
2. Every later scan reads only what changed, and falls back to a full scan on any doubt.
3. One walk serves every module: matching costs one hash lookup per entry; no path string is built to match.
4. The cache is a few megabytes, capped by construction, never deletes anything, and is safe to lose.
5. Scan time, memory and cache size are measured numbers with a bar each (rule 4 of [`rules.md`](../../rules.md));
   what can be checked deterministically is checked by a test, the rest by a bench run by hand.
6. The core, the engine and the modules know nothing about macOS: every OS call is an adapter behind a port.

## Non-goals

- Watching the disk while the app is open, history points and comparing saved states: plan
  [010](../010-history/readme.md), built on the two generations kept here.
- The duplicate search (its own module; it reuses the sizes, links and clone ids of the tree).
- Per-process attribution ([live storage attribution](../../ideas/live-storage-attribution.md)).
- Any screen: plan [003](../003-base-ui/readme.md) renders the scan, plan [012](../012-storage-map/readme.md) the
  storage map. This plan gives them the engine API they read.
- The permission model and its prompts: plan [006](../006-permissions/readme.md). This plan reports what it could not
  read and accepts a list of places it must not open.
- Clearing the app's own data (the "clear" button): plan [005](../005-settings/readme.md).

## Design

### 1. Where the code lives

```
fresher-core      ports: DiskWalk (changed), FolderReader, ChangeJournal, Volumes, ScanCache, DiskState (new)
fresher-macos     the macOS adapters of FolderReader, ChangeJournal, Volumes; the IO policy; all `unsafe` in `ffi/`
fresher-scanner   the walk, the tree, the matcher, the cache codec, the rescan — OS-neutral, over the ports
fresher-store     the cache files: slots, generations, envelope, checksum, cap — behind ScanCache
fresher-engine    claims, commit, restore, scan modes — over DiskWalk and DiskState only
app/tauri         composes them: the scanner gets the macOS adapters and the store's cache
```

- **The OS boundary.** Reading a folder, the change journal and volume facts are ports of the core, written in
  OS-neutral words (a folder, an entry, a journal position). `fresher-macos` implements them; a later Windows crate
  would implement them again. The scanner depends on `fresher-core` only, so it runs in tests over fakes and over a
  portable `std::fs` reader, and in the app over the macOS adapters.
- **Dependencies change** in [`scripts/check-crate-deps.ts`](../../../scripts/check-crate-deps.ts): `fresher-macos`
  may depend on `fresher-core` (it implements core ports, as plan 006 also expects for `AccessProbe`). The scanner's
  row stays `core, macos`, but `fresher-macos` is only its dev-dependency (benches and integration tests).
- **`unsafe`.** Cargo does not let a crate take the workspace's lints and override one of them, and the workspace
  forbids `unsafe_code`. The workspace level becomes `deny`; every crate except `fresher-macos` puts
  `#![forbid(unsafe_code)]` at its root, and the crate guard fails a crate root without it. Inside `fresher-macos`
  only the module `ffi` carries `#[expect(unsafe_code, reason = "…")]`; each unsafe block holds one operation and a
  `// SAFETY:` comment stating the invariant. Everything outside `ffi` — parsing the kernel's buffers included — is
  safe Rust.

### 2. The ports

All in `fresher-core`, with no IO; the fakes live in `fresher-testing`.

| Port            | What it answers                                                                                   | Implemented by                      |
| --------------- | ------------------------------------------------------------------------------------------------- | ----------------------------------- |
| `DiskWalk`      | `walk(request, cancel, on_match, on_progress)` → summary; `commit(claims)` → the `DiskState`      | `fresher-scanner` (`Scanner`)       |
| `FolderReader`  | one folder's entries with their attributes, in as few system calls as the OS allows               | `fresher-macos` (bulk); `fresher-scanner` (`StdReader`, portable) |
| `ChangeJournal` | the folders changed on a volume since a journal position, or why that cannot be told              | `fresher-macos` (FSEvents)          |
| `Volumes`       | which volume a path is on: its id, device, whether names are case-sensitive, used space            | `fresher-macos`; plan 009 extends it |
| `ScanCache`     | load and save the cache of one scan scope, two generations, within a cap                          | `fresher-store`                     |
| `DiskState`     | the read-only tree of the last committed scan: a node, its children, the largest files, search     | `fresher-scanner`                   |

The `DiskWalk` request grows from three arguments to one struct:

```rust
pub struct WalkRequest<'a> {
    pub roots: &'a [PathBuf],
    pub interests: &'a InterestSet,
    /// Places the walk must not open (they would raise a system prompt); each is reported as not scanned.
    pub avoid: &'a [PathBuf],
    pub mode: WalkMode, // Full | Auto (a rescan when the cache allows it) | Restore (the cache only, no disk)
}
```

`WalkSummary` gains `kind: ScanKind` (`Full`, `Rescan { changed_folders }`, `Restored { taken }`) and
`fallback: Option<FullScanReason>` (why a rescan became a full scan; logged, never shown with a path). `commit`
takes the resolved claims of the scan and returns the `Arc<dyn DiskState>` the engine keeps; a cancelled scan is not
committed.

### 3. The first scan

1. **Roots.** Each root is canonicalized once (the settings check already refused protected folders); a root inside
   another root is dropped and reported; a root that is not a folder stops the walk (`RootUnreadable`, as today).
   Each root's volume is identified through `Volumes`.
2. **The IO policy.** Before the first read, the macOS adapter sets the process's policy so that a cloud file is never
   downloaded (`setiopolicy_np` with `IOPOL_TYPE_VFS_MATERIALIZE_DATALESS_FILES` off) and, if 01 confirms the policy
   exists and works, that reading leaves access times alone. Without the first, a scan of a cloud folder would break
   the no-network rule.
3. **One system call per batch of entries.** The macOS `FolderReader` opens a folder by path with
   `O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC`, checks through the handle that it is the folder the parent's
   listing announced (same device and file id — a folder swapped for a link or another folder between the listing and
   the open is refused as `Replaced`), then reads it with `getattrlistbulk`: name, kind, device, file id, link count,
   modification time, flags, logical size, allocated size and, with `FSOPT_ATTR_CMN_EXTENDED`, the clone id and the
   extended flags; the private size only for a file flagged `EF_MAY_SHARE_BLOCKS`, by one more call (asking it in
   the bulk read doubles the walk — measured in 01, call 20). A path past `PATH_MAX` is opened in steps with
   `openat`; a folder whose handle has a link count of 0 after the read was removed (call 14). A per-entry error (`ATTR_CMN_ERROR`) marks that entry, not the folder. The buffer is parsed by safe
   code with bounds checks; a malformed record fails the folder as `Failed`, never panics.
4. **Workers and one aggregator.** A pool of worker threads (`std::thread::scope`) takes folder jobs from a shared
   queue with two lanes — the interesting lane first — reads the folder, matches every entry against the compiled
   matcher, folds the small files, and sends one `FolderBatch` to the aggregator. The aggregator runs on the calling
   thread (the engine's scan thread): it alone owns the tree, so nothing is locked, and it calls `on_match` and
   `on_progress`, which are not `Send`. The queue is a `Mutex` and a `Condvar` over two stacks (depth-first per lane
   keeps the queue short); `crossbeam-deque` is taken only if 01 or the benches show the lock to matter (01: it does
   not). The pool has one thread per core, at most 16: more threads made the walk slower in 01 (call 13).
5. **Interesting places first.** A folder whose matcher state is inside an anchored interest (on the way to
   `Library/Caches/*`, say) goes to the interesting lane, so categories fill in while the long tail is still read.
6. **Boundaries.** A folder on another device than its root's is not entered (a mount point: reported as skipped, not
   as unreadable); a link is listed, never followed; an avoided place is listed as `NotScanned::Avoided` and not
   opened; nothing under a protected path is entered (the same list as
   [`restrictions/deletion-safety.md`](../../restrictions/deletion-safety.md)).
7. **Errors by kind, never fatal.** `EPERM` on a folder is `Guarded` (macOS privacy protection), `EACCES`
   is `NoPermission`, a vanished or replaced folder is `Failed`; `EINTR` is retried up to three times. The folder
   becomes an unreadable node; its ancestors' sizes become `Partial` (`42 GB+`).
8. **Cancellation.** Workers check the token before each folder; the aggregator drains and stops; the walk returns
   `Cancelled` and nothing is committed: the cache keeps the previous state.
9. **Progress.** Entries and bytes, as today, throttled by the engine.

### 4. Sizes, links and clones

- **Three sizes per item**: logical (what Finder shows), allocated (what it takes on disk) and reclaimable (what
  removing it frees: the private size, not shared with a clone or another hard link). `Sizes` gains `reclaimable`;
  the UI shows allocated as the headline and reclaimable as "freed if removed" (plan 012).
- **Hard links** (a file whose link count is above 1; a folder's link count is its child count on macOS and is
  ignored) are kept in a `(device, file id)` set owned by the aggregator. Their bytes are attributed after the walk to
  the first path of the group in depth-first name order, so totals do not depend on which worker came first.
- **Clones**: the shared part of a clone group (allocated minus private) is counted once per `(device, clone id)`, the
  private part per file. How often clones and links occur in a real home is measured in 01: 4.3 % of the home's
  bytes are hard-linked, 10.3 % shared by clones. A clone is a file flagged `EF_MAY_SHARE_BLOCKS`; a small
  hard-linked or cloned file is folded like any small file, and the aggregator's link table keeps its identity, its
  folder and its size (call 21).
- **Cloud files** whose data is not local count their logical size and no allocated size; they are never read.

### 5. The tree in memory

- **Folders are nodes; small files are folded into their folder** as a count and the sums of their three sizes. A
  file is kept as its own record when it is large (1 MiB and up, confirmed in 01) or matched by an interest; a small
  hard-linked or cloned file is folded and only listed in the link table (01: keeping them would double the nodes).
- **An arena**: nodes in one `Vec`, 32-bit indices for the parent, the first child and the next sibling; names in one
  shared byte buffer (offset and length); flags in one byte (folder, kept file, link, unreadable kind, avoided,
  collapsed, opaque). The node also holds the folded count and sums, the subtree totals, the modification time and the
  index of its first interest hit. About 96 bytes a node; the measured home of 4.7 million entries is 556 thousand
  nodes — about 53 megabytes.
- **Interest hits** are kept per node: which interest matched, whether a marker file was valid, which sibling names
  were present. A rescan or a restore re-emits the matches from the tree without reading the disk.
- **Opaque subtrees**: a folder claimed `Whole` is read to the end for its sizes, links and errors, but its insides
  are not kept as nodes and nothing inside it is matched — the same as the reference walk.
- **Determinism**: when the walk ends, every folder's children are sorted by name and the totals are computed
  bottom-up in one pass. The same disk gives the same tree, byte for byte, whatever the thread count.
- The small files of a folder are not kept; when a screen opens a folder, they are read live (one bulk read).

### 6. Matching

- The interests of **every registered module, enabled or not**, are compiled once per scan into one matcher; the
  engine drops the matches of disabled modules. Turning a module on or off then never invalidates the cache.
- The compiled matcher: a hash map of exact folder names, one of marker file names, one of extensions, a set of
  sibling names, and a trie of anchored paths. The trie state travels with each folder job, so a folder's children are
  matched by one lookup in the current trie node — no path is built or compared.
- **The semantics are the reference walk's** (`NaiveWalk` in `fresher-testing`): exact names, a folder name matches a
  folder or a link, an extension a file or a link, a marker file a folder holding a regular file of that name, `*` one
  name. The scanner is tested against it on every fixture and on random trees; a change of semantics changes both.
- **Anchors.** An anchored interest is compiled against an absolute folder: today `Pattern::UnderHome` against the
  home folder the app injects; after plan [009](../009-system-knowledge/readme.md), a known folder resolved through
  `KnownFolders`. The scanner takes a resolver function, so 009 changes only the composition.
- **`CACHEDIR.TAG`** is checked by the walk: when a marker file named `CACHEDIR.TAG` is found, the reader reads its
  first 43 bytes (opened without following a link, regular file only) and the match carries
  `marker: Some(MarkerCheck::Valid | Invalid)`. The signature check is a pure function of the core;
  `fresher-rules` keeps only the decision, and its own read (the hack "CACHEDIR.TAG read by the rules") goes.
- The fingerprint of the compiled interest set (a stable 64-bit hash of our own over the canonical interests) is
  stored in the cache; a different fingerprint means a full scan.

### 7. Claims: one owner per path

Two modules may report the same path, or one finding may sit inside another (a cache inside a project's
`node_modules`). The engine resolves this on the findings, after every module has finished:

- the deepest finding owns its subtree; an outer finding's counted size is its size minus the nested ones;
- two findings of the same path: the higher danger level owns it, then the module registered first;
- category totals add counted sizes only, so no byte is counted twice (a property test holds it);
- the engine hands the resolved claims (path, category) to `DiskWalk::commit`, which records the owner category on the
  nodes — so an old state stays attributable after the rules change (plan 010) — and saves the cache.

### 8. Rescans: read only what changed

macOS keeps, per volume, a record of the folders that changed (the FSEvents database). No helper of ours runs in the
background: the app reads that record when the user starts a scan.

1. The cache of the scope is loaded; it holds each volume's id, the journal's id and the last journal position —
   the event id taken before that scan's walk began (01: the newest ~100 ms of changes may arrive after the
   history ends, always with a larger id).
2. **Any doubt means a full scan**: no cache, a broken or older cache, another fingerprint, other roots or another
   avoid list, another volume or journal id, a gap in the journal (it was reset, dropped events, its ids wrapped, a
   root was moved), or more changed folders than a full scan would read faster (the threshold is measured in 07), or
   a position too old or a journal read too long to beat a full scan (01: one developer day of history took 77 s to
   read; see call 1).
3. The journal gives the changed folders since the position (`ChangeJournal::changes_since`), flushed up to now, and
   the subtrees it could only name as a whole.
4. Each changed folder that is in the tree is read again (one bulk read): its folded sums are recomputed, new
   subfolders are walked in full by the same workers, removed ones are dropped. A changed path not in the tree is
   handled at its nearest ancestor in the tree. A change inside an opaque or collapsed subtree re-reads that subtree.
5. **Unreadable and skipped folders are always read again**: Full Disk Access may have been granted since.
6. **A check**: a deterministic sample of folders the journal says did not change (32, tuned in 07, seeded by the
   scan id) is read again and compared; one difference means a full scan, with the reason logged.
7. The totals are recomputed bottom-up, the matches re-emitted from the tree, and every module runs again over all of
   them: modules are pure and fast, and a module's `finish` (the newest version, say) needs every match.

Folder modification times are never used to skip a folder: they do not change when a file grows in place — exactly
the case the product exists to find.

### 9. Restore at launch

At launch the engine runs a scan in `Restore` mode: the walk loads the current generation and emits its matches
without touching the disk; the modules with interests run over them; the result is labelled with the date of the
state (`ScanKind::Restored { taken }`). Modules with no interests (an external tool such as the Docker CLI) do not run
until the user starts a scan: the app runs nothing the user did not start ([`goal.md`](../../goal.md), section 8).
With no cache, the restore ends at once with nothing, and the first-run screens of plan 014 apply.

### 10. The cache

The user's requirement: super lightweight, capped, safe to lose.

- **What is kept**: one file per scan scope (the canonical roots and their volumes), two generations — `current` and
  `previous`. A scope that is not the last two scopes used is overwritten. At most 2 scopes × 2 generations = 4
  files.
- **The cap**: 8 MiB a file, so at most 32 MiB in all, inside the app's 50 MB footprint budget (`FOOTPRINT_BUDGET`),
  leaving room for settings, the cleanup journal and history points. The cap holds by construction; the footprint is
  measured by `DataFolder::footprint()` in a test.
- **Nothing is ever deleted**: the files are fixed slots (`scan-1.bin` … `scan-2.previous.bin`), written by the
  store's atomic write and rotated by renaming `current` over `previous`. No deleting function is called, so the
  deletion rule needs no exception. Clearing the cache on demand is plan 005's, with the store's stale temporary files
  ([known issue](../../known-issues/fixed/store-stale-temporary-files.md)).
- **The envelope** (the store's): a magic, the container format and version, the scope key, the body's length and a
  CRC-32 (`crc32fast`, already in the lock file); a newer, older or broken file is discarded (`VersionPolicy::Discard`)
  and costs one full scan, never a wrong result.
- **The body** (the scanner's): its own tree format version; the volume and journal ids and positions; the time it
  was taken; the roots, the avoid list, the interest fingerprint; the owner categories as a table; then the folders in
  depth-first order, each with its child count (so no ids and no parent pointers are stored), its name front-coded
  against the previous sibling (siblings are sorted), its flags, its folded count and sums, its modification time,
  its hits and claim, and its kept files — all as variable-length integers. Subtree totals are recomputed on load.
- **Fitting the cap**: if a body would exceed the cap, the smallest subtrees without hits, claims or unreadable
  folders are collapsed into their root folder (its totals and counts only), with the threshold raised until the
  body fits. A rescan that meets a change inside a collapsed subtree re-reads that subtree.
- **Compression**: the body is compressed with deflate level 1 (`miniz_oxide`) inside the store's envelope, which
  checks the compressed bytes; the cap applies to the file on disk. 01 measured −60 % (13.6 MB → 5.5 MB, 49 ms) on the
  home's body, so a home like it fits without collapsing (the user's decision of 2026-11-15).
- How many generations and scopes are kept is the store's policy, not the format's: plan 010 changes the policy and
  thins older states inside the same budget.

### 11. Queries and search

The engine keeps the last committed `Arc<dyn DiskState>`; modules never see it (they get matches), the UI reads it
through engine methods that later plans expose as commands:

- `node(id)`, `children(id, order, offset, limit)` — by size or by name, paged like findings;
- `largest_files(limit)` — the kept file records, the largest first;
- `search(query, limit)` — names containing the query, case ignored, the largest first, over the in-memory name buffer
  (a lowercase copy is built on the first search); no index on disk;
- `unreadable(limit)` — the top-most unreadable or avoided folders with their reason and how many folders each hides
  (for plan 006);
- `path(id)` — the full path, rebuilt from the parents.

A `NodeId` carries the state's id, so an id from an older state is refused, like a `FindingId`.

### 12. Interfaces with other plans

- **006 (permissions)** — 004 offers: unreadable folders by kind (`NoPermission`, `Guarded`, `Failed`)
  in `WalkSummary` and, with their places, through `DiskState::unreadable`; the `avoid` list of the walk request;
  unreadable folders re-read on every rescan. 004 needs from 006: the list of places not to open while their
  permission is not asked. Until 006, `fresher-macos` gives a stand-in list of the home's folders that raise a
  system prompt (Desktop, Documents, Downloads, other apps' containers; confirmed in 01), recorded in
  [`../../hacks/`](../../hacks/README.md).
- **009 (system knowledge)** — 004 needs: `KnownFolders` to resolve anchored interests (the scanner takes a resolver;
  until 009 the home folder is the only anchor). 004 offers: the `Volumes` port with the methods it needs (identify a
  path, case sensitivity, used space), which 009 extends with kinds, free and purgeable space; the shared walk for
  the `Projects` service's marker files (an interest set owner is just an index to the walk).
- **010 (history)** — 004 offers: two generations in depth-first order (comparable in one merge pass), the owner
  category on every node, the store's retention as a policy. History points are 010's.
- **005 (settings)** — the cache's size in the footprint; clearing it is 005's.
- **007 (safe development)** — 004 adds the large-tree generator and the fakes of its new ports to
  `fresher-testing`; 007 extends them into scenarios and "the source of disk changes".
- **012 (storage map)** — the claims, the three sizes, the unreadable places and the tree queries.
- **003 (base UI)** — `ScanKind` in `LastScan` and `ScanFinished` ("restored from 14 November"); the commands for the
  queries come with the screens that use them.
- **External tools** — the `CommandRunner` port has no cancel token; the walk runs no command, and the real runner
  already kills a program after 60 s. A cancellable run belongs to the plan of the first module that calls a tool.

### 13. Measuring

Rule 4 asks for measured numbers. Two kinds:

- **Deterministic proxies, held by tests** (run in the pre-push hook): one bulk read per folder of up to a buffer's
  entries (a counting reader); a rescan with nothing changed reads only the check sample and the unreadable folders;
  the tree's heap bytes per node; the cache's bytes per node; the cache within its cap; the data folder within its
  budget after scans of the generated tree.
- **Timings and peak memory, held by benches** run by hand (`npm run bench`, criterion, release build) on generated
  trees; each step that can move them records the numbers before and after in its notes. Peak memory comes from
  `getrusage` through `fresher-macos`.

Generated trees come from `fresher-testing` (`LargeTree`): a seeded shape (counts, depth, the share of large files,
hard links, `node_modules`-like deep trees, cache folders), sparse files so sizes cost no disk, built once under
`target/bench-trees/` and removed by `cargo clean`. Specs: `small` (100 thousand entries, used by tests) and `large`
(3 million entries, benches). A real home folder is measured by hand only, read-only, in 01 and in 08; no test touches
it.

The bars the benches and tests hold, set from the numbers of plan 01 (section "Measured (01)"): a measured value plus
30 %, rounded. A bar whose path 01 could not run end to end is derived from the measured parts; the step that builds
that path measures it and writes the number next to the bar in its notes.

| What                                                    | Bar       | Source (01)                                                                                     | Held by                             |
| ------------------------------------------------------- | --------- | ----------------------------------------------------------------------------------------------- | ----------------------------------- |
| full scan, warm, `large` (3 M entries)                  | ≤ 10 s    | probe tree: 971 551 entries in 2.38 s at 8 threads (≈ 409 000 entries/s) → 7.3 s for 3 M         | bench `scan/full/large`             |
| rescan, nothing changed, `large`                        | ≤ 0.5 s   | derived, for a position seconds old: `FlushSync` 6–20 ms; 32 sample folders ≈ 7 ms; body decode ≈ the 22 ms encode (a real home pays more: 30 minutes of history took 3.1 s) | bench `scan/rescan-unchanged/large` |
| rescan after 1 000 changed folders, `large`             | ≤ 1 s     | derived: 1 000 folder reads at ≈ 4 700 folders/s = 0.21 s, plus the unchanged rescan              | bench `scan/rescan-day/large`       |
| restore: cache → matches emitted, `large`               | ≤ 0.2 s   | derived: the home's body (556 k nodes) encodes in 22 ms, deflate-1 in 49 ms                      | bench `scan/restore/large`          |
| search by name, `large`                                 | ≤ 50 ms   | derived: the home's names are 6.3 MB, one linear pass                                            | bench `state/search/large`          |
| peak memory of a full scan, `large`                     | ≤ 200 MB  | the home walk alone peaked at 133 MB (4.7 M entries, one folder of 408 k entries); its tree adds ≈ 53 MB (556 k nodes × 96 B); the probe tree peaked at 7 MB | bench report (`getrusage`, bytes)   |
| tree heap per node                                      | ≤ 96 B    | the arena layout of section 5 (01 does not build the tree; 04/01 measures)                       | test on `small`                     |
| cache body per node, uncompressed                       | ≤ 32 B    | the home's body: 13 639 663 B / 555 862 nodes = 24.5 B (26.0 B per folder)                       | test on `small`                     |
| one cache file / the whole cache                        | ≤ 8 / 32 MiB | the store's cap; the home's body is 13.6 MB uncompressed (collapsed to fit), 5.5 MB with deflate-1 | the store's cap, tests              |
| folders read by a rescan with nothing changed           | sample + unreadable | design                                                                                | test with a counting reader         |

### 14. Testing

- Every scanner test runs on a `FixtureTree` or a generated tree in a temporary folder; nothing reads the real home.
- **Against the reference**: on every fixture home and on random trees (proptest), the scanner with `StdReader` gives
  the same matches as `NaiveWalk`; with the macOS reader (an integration test in the scanner, `fresher-macos` as a
  dev-dependency) the same again.
- **The bulk reader** is tested against `std::fs` on fixture trees; its buffer parser by unit tests on made-up buffers
  and a property test (never panics on any bytes).
- **The journal** has a fake (`FakeChangeJournal`: scripted changes and gaps) for the rescan logic; the FSEvents
  adapter has its own integration tests on a temporary folder (create, grow in place, remove, then
  `changes_since`).
- **The cache**: round-trip, corruption (every truncation and a flipped byte are refused), the cap, rotation, a
  newer version kept untouched.
- **The engine**: claims (nested, same path, property: no double count), restore, modes, commit only when not
  cancelled — over `NaiveWalk` and fakes.
- **Core e2e** (`crates/e2e`): scan a fixture home with the real scanner, commit, rescan after a scripted change,
  restore in a new engine — against golden reports.

## Measured (01)

Throwaway probes of plan [01](./01-measure-first/readme.md), run on 2026-11-15. Counts and timings only; no name or
path from a home folder.

**The machine**: MacBook Pro, Apple M3 Pro (12 cores: 6 performance, 6 efficiency), 18 GiB, internal SSD, APFS,
macOS 27.2.1 (26B131) — the only macOS version available; 13.x–26.x were not run. The probes ran from a terminal
without Full Disk Access (it reads Desktop, where the repository lives). Release builds, warm caches (a configuration
run once untimed, then timed); no cold numbers (no reboot). Other work was running on the machine, so the timings carry
some noise.

**The trees**: the *probe tree* — generated in a temporary folder: 971 551 entries (11 250 folders, 960 201 sparse
files, 100 hard links, 100 clones, 100 symlinks, a folder of 10 000 entries, a 200-deep chain past `PATH_MAX`); the
*home* — the developer's home folder, read-only, without the places that could raise a prompt (Documents, Downloads,
Pictures, Movies, Music, `Library/Containers`, `Library/Group Containers`, `Library/Mobile Documents`,
`Library/CloudStorage`).

### The bulk read (01/01)

| What                                                        | Value                                                                    |
| ----------------------------------------------------------- | ------------------------------------------------------------------------ |
| attributes returned for files, folders and links            | all requested (masks: common 0x8204040b, folder 0x2f, file and link 0x607, extended 0x1308) |
| `ATTR_CMN_ERROR` when the entry has no error                | not in the returned mask, takes no bytes                                 |
| record alignment / attribute packing                        | every record 8-byte aligned; attributes packed on 4 bytes in group order (common, folder, file, extended), each group in bit order; 8-byte values read unaligned |
| record length                                               | file 120 B (144 B with extended attributes), folder 136 B                |
| folder link count                                           | 1 (not the child count); file attributes not returned for folders        |
| private size                                                | returned for every file: equals allocated for a plain file, 0 for a fully shared clone or sparse file, the diverged blocks for a modified clone |
| marks a clone                                               | `EF_MAY_SHARE_BLOCKS` in `ATTR_CMNEXT_EXT_FLAGS` (a diverged clone keeps it, but gets its own clone id and a reference count of 1) |
| device id                                                   | positive (0x1000011 …); stored as the `i32` bits                         |
| buffer 16 / 64 / 256 KiB, 1 thread                          | 43.0 / 43.1 / 43.2 entries per call; 11.6–11.9 / 11.8–12.0 / 11.4–11.7 s — no difference; 64 KiB taken |
| std walk (`read_dir` + `symlink_metadata`), 1 thread        | 12.8–14.0 s (system 6.8–7.0 s)                                           |
| bulk walk, 1 thread, extended attributes without private size | 9.0 s (system 3.3 s)                                                   |
| bulk walk, 1 thread, with private size                      | 11.4–12.0 s (system 5.3 s)                                               |
| bulk walk by threads, without private size                  | 4: 3.35 s · 6: 2.62 s · **8: 2.38 s** · 12: 2.49 s · 24: 3.17 s · 48: 3.31 s |
| bulk walk by threads, with private size                     | 4: 4.59 s · **6: 3.74 s** · 8: 4.06 s · 12: 6.63 s · 24: 7.37 s · 48: 7.75 s · 64: 7.08 s |
| cost of each extended attribute (6 threads)                 | none 2.46 s · clone id 2.45 s · flags 2.56 s · clone count 2.58 s · **private size 3.58 s** |
| folders per second, best                                    | ≈ 4 700 (probe tree, 8 threads); ≈ 24 000 (home, 8 threads; its folders are smaller) |
| home walk, 4.7 M entries, without private size              | 1: 96.6 s · 4: 29.6 s · 6: 22.9 s · **8: 21.8–22.6 s** · 12: 23.2 s · 24: 23.6 s · 48: 25.4 s |
| home walk with private size                                 | 6: 44.4 s · 8: 43.8 s · 12: 48.0 s — the private size doubles the walk              |
| CPU against wall at the best count                          | probe tree 8 threads: 6.8 s system in 2.4 s; home: 62 s system in 22 s; user time ≈ 2 % |
| time waiting for the `Mutex` at the best count              | 0–1 ms (probe tree), 0.7–1.3 s summed over 8 threads in 22 s (home); it grows only past the core count (48 threads: 124 s summed) |
| errors                                                      | mode `000`: `EACCES` on open; swapped for a link: `ENOTDIR` (not `ELOOP`); swapped for another folder: open succeeds, another inode; removed before open: `ENOENT`; removed after open: the read returns 0 entries and no error; past `PATH_MAX`: `ENAMETOOLONG` |
| IO policy                                                   | dataless materialization of a process started from the terminal is ON (inherited); setting it OFF succeeds. Access-time OFF succeeds; reading did not move access times on this volume either way |
| totals                                                      | bulk = std on the probe tree (960 201 files, 2 722 536 845 375 logical bytes, 2 284 371 968 allocated), except the `PATH_MAX` folder, which both fail |

### The change journal (01/02)

| What                                                        | Value                                                                    |
| ----------------------------------------------------------- | ------------------------------------------------------------------------ |
| **a file growing in place** (append, overwrite, `truncate`) | **recorded**: an event for its folder, every time                        |
| create, remove, rename within, chmod                        | the folder                                                               |
| rename across folders                                       | both folders                                                             |
| new or removed subtree                                      | the parent and every folder of the subtree (removed ones named too)      |
| hard link added                                             | the new link's folder and the source folder's parent — not the source's folder |
| the watched root moved                                      | an event on its parent only; no `RootChanged` in a history replay        |
| `FlushSync`                                                 | changes 0.1 s old or more always in the history; changes of the last ~100 ms may arrive after `HistoryDone`, with a larger id — never lost if the stored position is the id taken before the walk |
| a position older than the journal                           | no flag: the history starts at the oldest kept event (a silent gap)      |
| `FSEventsGetLastEventIdForDeviceBeforeTime`                 | returns 0 for every time on this macOS: unusable                         |
| journal reset (disk image)                                  | the journal UUID changes; event ids are global and keep growing; no gap flag |
| gap flags seen                                              | `MustScanSubDirs` with `UserDropped` (large replays), `KernelDropped` (once in the whole history), `Mount`/`Unmount`; `EventIdsWrapped`, `RootChanged` not provoked |
| a burst of 100 000 creations in 1 000 folders               | no drop; 2 075 folder events for 1 002 folders, delivered in 0.92 s      |
| without Full Disk Access                                    | events under protected folders are reported (folder paths)               |
| volumes                                                     | APFS, HFS+ and FAT images mounted normally have a journal; mounted `-nobrowse`, none (UUID NULL, no events) |
| retention                                                   | ≈ 3.5 weeks of history on this Mac (ids mapped to time through folder modification times; bounded by the journal's size, so heavier use means shorter) |
| cost of `since` on the home (folder events)                 | 30 minutes: 3.1 s, 15 016 events, 4 958 folders · 1 day: 77 s, 655 100 events, 191 200 folders · 4.5 days: 201 s · 1 week: 470 s, 705 712 folders, with 6 373 `MustScanSubDirs` + `UserDropped` · the whole journal of a quiet path: ≥ 40 s (the journal is scanned whatever the path) |
| an ordinary day                                             | ≈ 7 500 folders outside the developer's repositories; ≈ 194 000 with them (96 % under the folder holding the repositories: build output, packages, worktrees) |
| folder against file events                                  | the 100 000-file burst: 2 075 folder events, 101 001 file events — folder-level streams are enough |

### Volumes, links, clones, prompts (01/03)

| What                                                        | Value                                                                    |
| ----------------------------------------------------------- | ------------------------------------------------------------------------ |
| volume identity                                             | the UUID (`ATTR_VOL_UUID` on the mount point): present for APFS, HFS+ and FAT; device numbers are per mount (`stat("/")` and `statfs("/")` even disagree) |
| volume attribute packing                                    | not bit order (`SPACEUSED` came before `CAPABILITIES` and `UUID`)        |
| case sensitivity                                            | not case-sensitive (data volume, system volume, all images)              |
| home: entries / folders / files / links                     | 4 665 830 / 525 119 / 4 129 394 / 11 313; deepest folder 22; largest folder 408 495 entries; names 21 bytes on average |
| home: allocated, hard links once                            | 286.15 GiB; `du -skx` of the same set 286.15 GiB (agree within 0.01 %)   |
| hard links                                                  | 639 405 files in 157 843 groups; 13.7 GiB once (4.3 % of allocated)     |
| clones                                                      | 67 245 files, 56 139 clone ids; 32.8 GiB allocated, of it 28.4 GiB shared (10.3 %) |
| cloud files                                                 | 0 in the walked part (iCloud Drive was not opened)                       |
| unreadable folders                                          | 53, all `EPERM`; no `EACCES`; no per-entry error; no mount point         |
| files under 256 KiB / 1 MiB / 4 MiB                         | 96.8 / 99.3 / 99.8 % of files, 15.2 / 30.1 / 41.9 % of allocated bytes  |
| nodes (folders + kept files) at 256 KiB / 1 MiB / 4 MiB     | 656 781 / **555 844** / 535 063                                          |
| … if every small hard-linked or cloned file were kept too   | 1 304 757 / 1 256 818 / 1 240 133                                       |
| cache body at 1 MiB (in memory)                             | 13 639 663 B: 24.5 B per node, 26.0 B per folder; encoded in 22 ms       |
| compressed                                                  | deflate level 1 (`miniz_oxide`): 5.48 MB, −59.8 %, 49 ms; level 6: 4.53 MB, −66.8 %, 302 ms; zstd fastest (`ruzstd`): 6.03 MB, −55.8 %, 105 ms |
| `ru_maxrss`                                                 | bytes (100 MiB touched → +104 873 984)                                   |

Prompts, from a terminal (whose grants apply to the probe; a fresh app's cannot be seen this way):

| Place                                                        | Outcome                                                  |
| ------------------------------------------------------------ | -------------------------------------------------------- |
| `Library/Mail`, `Library/Messages`, `Library/Safari`, `Library/Cookies`, `~/.Trash` | `EPERM` on open, no prompt (Full Disk Access class) |
| Desktop                                                      | read (the terminal holds a grant)                        |
| Documents, Downloads, iCloud Drive, `Library/CloudStorage`   | not opened: a "Files and Folders" prompt is documented  |
| `Library/Containers`, `Library/Group Containers`             | not opened: the "data from other apps" prompt (macOS 14 and later) |
| Pictures, Movies, Music, `Library/Calendars`, the address book folder | not opened: not known whether they prompt      |
| removable and network volumes                                | not tested                                               |

**Needs a by-hand run from Finder** (plan 006/01, a freshly signed app with no grant): every "not opened" row above;
Desktop for a fresh app; a removable and a network volume; a cloud folder with a dataless file (which flag, which
sizes, and that nothing downloads with materialization off); the journal with Full Disk Access and after a sleep of
the Mac; `EventIdsWrapped` and a live `RootChanged`; the size of a day on a non-developer Mac; every macOS version
from 13.

## Open questions

None. Open question 1 (compressing the cache) is answered under Decisions.

## Decisions

- 2026-11-15 — **Super lightweight cache** (the user's task): a program fighting disk clutter must not clutter the
  disk; the cache is small, capped and safe to lose.
- 2026-11-15 — **macOS first, Windows not ruled out** (the user): OS calls — bulk folder reads, volume, file id and
  APFS specifics, the change journal — are adapters behind ports, implemented in the macOS crate; the core, the engine
  and the modules know nothing about the OS.
- 2026-11-15 — **The agent takes the recommended decisions** (the user, on expanding the draft); they are listed
  below, each waiting for a confirm or an overrule.
- 2026-11-15 — **`unsafe` for the macOS calls** (the user): the workspace lint `unsafe_code` becomes `deny`, every
  crate but `fresher-macos` keeps `#![forbid(unsafe_code)]` at its root (checked by the crate guard), and `unsafe` lives
  only in `fresher-macos`'s `mod ffi` (agent's call 11); `CLAUDE.md` and `quality.md` follow in step 03/01.
- 2026-11-15 — **The plan is approved, with every agent's call below confirmed** (the user).
- 2026-11-15 — **The cache is compressed** (the user, answering open question 1 after the 01 measurements): deflate
  level 1 through `miniz_oxide` (pure Rust; MIT OR Zlib OR Apache-2.0; already in `Cargo.lock`), in the store's
  envelope (step 06/01); agent's call 8.

## Agent's calls (to check)

From the draft, kept or refined:

1. **Rescans driven by macOS's change journal (FSEvents)**, a full scan on any doubt; folder modification times are
   never trusted. To verify in 01: how far back the journal reaches, whether a file growing in place is recorded, how
   it behaves without Full Disk Access.
   **01: confirmed.** A file growing in place is recorded for its folder; events under protected folders are reported
   without Full Disk Access. Retention cannot be asked (`GetLastEventIdForDeviceBeforeTime` returns 0) and a
   position older than the journal gives a silent gap, so the sampled check (call 25) is load-bearing, not a
   formality. Two details for 03/04 and 07: the position stored is the event id taken *before* the walk (changes of
   the last ~100 ms may arrive after `HistoryDone`); a new hard link is not reported on its source's folder, so a
   folder holding a member of a known link group is re-read when any member's folder changes. **Changed: a replay is
   not cheap** — on the developer's home one day of history took 77 s and a week 470 s, against a full warm scan of
   22 s; a position far back costs at least 40 s whatever the path. So the rescan bounds the journal read by the age
   of the position and by a time budget, and falls back to a full scan when either is passed: 07 sets both (a start:
   a position older than 2 weeks, retention being ≈ 3.5 weeks, or a read longer than half the last full scan).
   Full-scan flags: `MustScanSubDirs` (that subtree; a full scan at or above a root), `UserDropped`,
   `KernelDropped`, `EventIdsWrapped`, `RootChanged`, a journal UUID changed or missing (a `-nobrowse` mount has no
   journal), no `HistoryDone` within the budget, a root's parent reported (check the root's identity).
2. **Small files folded into their folder**; files of 1 MiB and up kept (the threshold is tuned in 01).
   **01: confirmed at 1 MiB**, changed for links: at 1 MiB, 99.3 % of the home's files fold, 30 % of its bytes, and
   the tree has 556 k nodes for 4.7 M entries (256 KiB: 657 k; 4 MiB: 535 k). Keeping every small hard-linked or
   cloned file as its own record would more than double that (1.26 M), so such a file is folded like any small file
   and only its identity goes to the link table (call 21).
3. **A compact binary cache of our own format**, no SQLite (plan 002, decision 1); two generations.
4. **A compiled matcher and one claim per path.**
5. **Interesting places are scanned first** (two lanes in the work queue).
6. **No watching while the app is open**: a rescan starts only when the user asks.
7. **Search by name stays in memory**, a linear scan with no index on disk.
8. **Compression of the cache decided by measurement** in 01: none unless a pure-Rust compressor saves more than a
   third, and then it is asked about as a new dependency.
   **01: compression pays** (deflate level 1: −60 %, 49 ms on the home's body); the user approved it (Decisions).
9. **Target version 0.3.0.**

New, made while expanding the draft:

10. **OS-neutral ports in the core** (`FolderReader`, `ChangeJournal`, `Volumes`, `ScanCache`, `DiskState`),
    implemented by `fresher-macos`, `fresher-store` and `fresher-scanner`; `fresher-macos` depends on `fresher-core`;
    the scanner depends on the core alone (`fresher-macos` as a dev-dependency only).
11. **`unsafe_code` becomes `deny` in the workspace**, every crate but `fresher-macos` keeps `#![forbid(unsafe_code)]`
    at its root and the crate guard checks it; inside `fresher-macos` only `mod ffi` is allowed `unsafe`.
    `CLAUDE.md` and [`quality.md`](../../quality.md) are updated in the same step.
12. **Our own FFI declarations** for the few functions `libc` lacks (FSEvents, libdispatch, `getattrlist` volume
    attributes, `setiopolicy_np` if missing); no new binding crate. A crate already in `Cargo.lock` under a permissive
    licence (`objc2-core-foundation`) is used for CoreFoundation types only if 01 shows it saves real work.
13. **A `Mutex` + `Condvar` work queue first**; `crossbeam-deque` only if measured contention justifies a new
    dependency. Thread count from 01 (starting at 3 × the cores, at most 48).
    **01: the queue confirmed, the thread count changed.** The lock costs 0–1 ms at the best count; it shows only past
    the core count. More threads than cores make the walk slower (the kernel's own contention: system time × 4–10),
    not faster: best 8 on 12 cores, 6–12 within 10 %, 24 and 48 worse on both trees. The count is the number of
    cores, at most 16.
14. **Open by path with an identity check** (`O_NOFOLLOW | O_DIRECTORY`, then device and file id against the
    parent's listing) rather than keeping parent handles open: no file-descriptor limit to raise, and a swapped folder
    is refused.
    **01: confirmed, with two cases.** A swapped link gives `ENOTDIR` (not `ELOOP`); a swapped folder opens and only
    the identity check sees it. A path past `PATH_MAX` fails with `ENAMETOOLONG`: such a folder is opened in steps
    with `openat` from a shorter ancestor (each step `O_NOFOLLOW | O_DIRECTORY`). A folder removed after the open
    reads as empty without an error: after the read the handle's link count is checked (0 → `Failed`).
15. **The IO policy**: cloud files never downloaded; access-time updates off for the process if 01 confirms the policy.
    **01: confirmed.** A process started from a terminal inherits materialization ON, so setting it OFF is required.
    Access-time OFF is accepted (no visible effect on this volume, which did not update access times on read); the
    policy types exist since macOS 10.14/10.15, below the minimum 13.0. The cloud-file check itself needs a by-hand
    run (006/01).
16. **`EPERM` → `Guarded`, `EACCES` → `NoPermission`**; plan 006 maps them to its capabilities.
    **01: confirmed.** Guarded places fail the open with `EPERM` (53 folders of the home, no `EACCES`); `EACCES` came
    only from a mode `000` folder.
17. **Until plan 006, the walk avoids the places that would raise a system prompt** (a stand-in list in
    `fresher-macos`, recorded in `docs/hacks/`) and reports them as not scanned.
    **01: the list, until 006/01's by-hand run says otherwise**: Desktop, Documents, Downloads, Pictures, Movies,
    Music, `Library/Containers`, `Library/Group Containers`, `Library/Mobile Documents` (iCloud Drive),
    `Library/CloudStorage`, `Library/Calendars` and the address book folder under `Library/Application Support`;
    removable and network volumes are outside the roots. The Full Disk Access places (Mail, Messages, Safari, …) do
    not prompt and are read as `Guarded`. Which of the list really prompt could not be seen from a terminal.
18. **Nested roots are dropped** (scanned through the outer root) and reported; mount points under a root are skipped
    and reported.
19. **`CACHEDIR.TAG` validity is read by the walk** and carried by `Match::marker`; the signature check moves to the
    core; the rules' own read is removed with its hack entry.
20. **`Sizes` gains `reclaimable`** (the private size).
    **01: confirmed, read differently.** Asking the bulk read for the private size doubles the walk (home: 22 s →
    44 s). The bulk read asks for the clone id and the extended flags (free); a file without `EF_MAY_SHARE_BLOCKS`
    has reclaimable = allocated (01 saw private = allocated on plain files); only a file with the flag (1.4 % of the
    home's files) gets its private size by one `getattrlist` call.
21. **Hard links attributed to their first path in depth-first name order** after the walk; clones' shared part
    counted once per clone id.
    **01: confirmed, with folded members** (call 2): the link table holds (device, file id) → the folders holding
    members and the allocated size; clones are found by `EF_MAY_SHARE_BLOCKS`, not by a reference count above 1. On
    the home: 639 k hard-linked files in 158 k groups (4.3 % of bytes), 67 k cloned files (10.3 % of bytes shared).
22. **The walk matches the interests of every registered module**, enabled or not; the engine filters, so the cache
    survives settings changes.
23. **Claims are resolved in the engine on the findings**: the deepest owns, then the higher danger, then the module
    registered first; nested sizes are subtracted from category totals.
24. **A rescan re-runs every module over all matches** and streams every finding again; no findings diff in this plan
    (plan 010 compares states).
25. **A sampled check** of unchanged folders (32 by default) on every rescan, instead of the draft's comparison with
    the volume's used space: the roots are folders, not volumes; plan 012 reconciles the volume.
    **01: confirmed and load-bearing**: the journal gives no flag when the position is older than its history (call
    1). A changed or missing journal UUID means a full scan.
26. **Unreadable and avoided folders are read again on every rescan**; an avoid list that changed means a full scan.
27. **A cancelled scan is not committed**; a partial one is.
28. **Restore at launch** runs only the modules with interests over the cached matches; tool modules wait for the
    user's scan.
29. **The cache is keyed by scan scope** (canonical roots and their volumes), not by volume; at most 2 scopes × 2
    generations; 8 MiB a file, 32 MiB in all; fixed slots rotated by rename, so nothing is ever deleted.
30. **Over the cap, small unclaimed subtrees are collapsed** until the body fits.
31. **`crc32fast` for the checksum** (already in `Cargo.lock`); our own stable FNV-1a for the interest fingerprint.
32. **No new IPC commands for the queries** in this plan: the engine API is tested here; the commands come with the
    screens (plans 003 and 012). `ScanKind` joins `LastScan` and `ScanFinished`.
33. **`CommandRunner` gets no cancel token or timeout variant here**: the walk runs no command; the first module that
    calls a tool takes it on.
34. **criterion** as a dev-dependency of `fresher-scanner` (named in [`testing.md`](../../testing.md); MIT OR
    Apache-2.0), default features off; timings held by benches run by hand, deterministic proxies by tests.
35. **A large-tree generator in `fresher-testing`** with its own seeded generator (no `rand`); trees live under
    `target/bench-trees/` and go with `cargo clean`.
36. **The probes of 01 read the developer's own home folder, read-only**, and record only counts and timings — never a
    name or a path; they stay in `NO_COMMIT/`.
37. **The `Volumes` port is started by this plan** with what the scan needs; plan 009 extends it.
38. **No history points here**: plan 010 writes them from the totals the engine already has.
