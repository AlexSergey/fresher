# 004/05/03 — Workers and the aggregator

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner

## Files

- `crates/scanner/src/walk/queue.rs` — `WorkQueue`: a `Mutex` over two `Vec<Job>` stacks (`interesting`, `rest`)
  and a `Condvar`; `push(job, lane)`, `pop() -> Option<Job>` (the interesting lane first; `None` when both are empty
  and no worker is busy, or when cancelled); a busy counter so the last idle worker ends the walk. `Job { node:
  NodeIndex, path: PathBuf, identity: FolderIdentity, state: TrieState, opaque: Option<NodeIndex> }`.
- `crates/scanner/src/walk/worker.rs` — the worker loop: pop a job, check the cancel token, `reader.read` into its
  own reused `FolderListing`, `matcher.folder` and `matcher.entry` for every entry, `head` of a `CACHEDIR.TAG` marker
  (43 bytes, `is_valid_cachedir_tag`), `fold` into a `FolderBatch`, send it to the aggregator over
  `std::sync::mpsc::sync_channel` (bounded, so a slow aggregator slows the workers instead of growing memory), push
  the subfolders the `Boundary` lets in — to the interesting lane when their trie state is live. Inside an opaque
  subtree the worker sums instead of keeping nodes and reports the sums against the opaque root.
- `crates/scanner/src/walk/aggregator.rs` — runs on the calling thread: receives batches, `Tree::apply`, calls
  `on_match` for each hit (a `Whole` claim only once its subtree is complete: a pending-subfolder count per opaque
  root), `on_progress` with entries and bytes; when the workers are done, `freeze`.
- `crates/scanner/src/walk/mod.rs` — `Walk::run(roots, matcher, boundary, reader, threads, cancel, on_match,
  on_progress) -> Result<(Tree, WalkSummary), WalkError>`, with `std::thread::scope` for the workers.
- `crates/scanner/src/scanner.rs`, `crates/scanner/src/lib.rs` — `pub struct Scanner` (built from `Arc<dyn
  FolderReader>`, `Arc<dyn Volumes>`, an anchor resolver, a thread count) implements `DiskWalk::walk` for
  `WalkMode::Full` and `Auto` (a full scan until 07); `Restore` → `NothingToRestore` until 07. `ScannerOptions`
  carries the thread count (default from 01: 3 × the cores, at most 48) and the folding threshold.
- `crates/scanner/benches/scan.rs` — group `scan/full`: a full scan of the generated tree with the macOS reader at 1,
  the cores, 2 ×, 3 × and 4 × the cores; the median time and the peak memory.

## Produces

The full scan behind `DiskWalk` for the engine (05/05, 05/06) and the base the rescan (07) extends.

## Checks

- Negative: `a_cancelled_walk_stops_and_returns_cancelled` (cancel from `on_progress` after the first batch; the walk
  returns within one folder per worker), `a_whole_claim_is_reported_only_when_its_subtree_is_read`,
  `the_queue_ends_when_the_last_busy_worker_finds_it_empty`.
- Positive: `every_folder_is_read_exactly_once` (counting reader on `SMALL`: reads = folders), `interesting_folders_are_read_before_the_rest`
  (counting reader with an order log: every folder on an anchored path is read before any folder of the long tail
  of the same depth or deeper), `one_thread_and_many_threads_build_the_same_tree` (the frozen trees are equal),
  `a_bulk_read_per_folder` (with a reader whose buffer holds every entry, one `read` per folder).
- Bench: `scan/full` on `LARGE`; the best thread count and the median time against the design's bar, and the peak
  memory, written in the notes; if the queue's lock shows in a profile (more than 5 % of the time waiting), the notes
  say so and the agent's call 13 is revisited with the user.

## Conclusions

- `walk/queue.rs`, `walk/worker.rs`, `walk/aggregator.rs`, `walk/mod.rs` (`Walk::run`), `scanner.rs` (`Scanner`,
  `ScannerOptions`, `AnchorResolver`, `home_anchor`) as written; `walk/errors.rs` (the retry of an interrupted read
  and the kinds) is created here because the worker needs it, and 04 checks it.
- **Deviation — the aggregator queues the subfolders, not the worker.** A subfolder's job carries its node, and only
  the tree's single owner makes nodes; so the worker sends, with its batch, one `Child` per kept folder (path,
  identity, trie state, the boundary's decision, whether a whole claim hit it) and the aggregator queues them after
  `Tree::apply`. The queue therefore counts *outstanding* jobs (queued, being read, or read but not yet applied)
  instead of busy workers: `pop` answers `None` once none is left. The jobs of one report are queued under one lock
  (`push_all`), so a worker never takes one of them before its siblings are in.
- **Deviation — the thread count.** The step says "3 × the cores, at most 48"; the design's measured call 13 says one
  thread per core, at most 16, and that is the default of `ScannerOptions` (the bench shows why it is revisited, below).
- Not spelled out, taken the simplest way: an opaque subtree is read by the same workers in parallel; each folder of
  it reports plain sums, shared files and the match sizes, and the aggregator keeps a pending count per opaque root,
  applies `apply_opaque` and reports the whole claims when it reaches zero. A folder becomes opaque by a whole name hit
  in its parent's listing, or by a whole marker hit in its own (then its own listing is summed). A whole claim's match
  sizes count every file once per name, as the reference walk does; the tree counts a hard link or a clone's shared
  part once. The scan roots are never matched themselves (as in the reference walk); `~` and `~/…` roots are filled in
  with the home anchor (the engine passes the roots as the settings write them).
- The summary counts what the reference walk counts: entries and unreadable folders outside whole claims, an entry on
  another device not at all; the progress also counts the entries inside whole claims, so it moves while a large
  `node_modules` is summed.
- `Scanner::walk` answers `NothingToRestore` for `Restore`; a cancelled walk returns `Cancelled` and keeps nothing.
- Checks: the step's tests in `walk/queue.rs` and `walk/tests.rs` (cancel within one folder per worker, a whole claim
  reported after its subtree, every folder of `SMALL` read once, interesting folders first with one worker, one and
  eight threads giving the same frozen tree and matches, one read per folder), plus an avoided root, a folder on
  another device (a reader that reports another device), an avoided folder with the counting reader.
- **Measured** (`scan/full`, the macOS reader, release, the developer's machine, other work running):
  - `SMALL` (100 000 entries): 1 thread 470 ms.
  - `LARGE` (3 000 000 entries): 1 thread ≈ 245 s (criterion's estimate of one run; 10 runs were not waited for);
    12 threads **23.4 s** (22.5–24.5); 24 threads **15.4 s** (14.9–15.9, the best); 36 threads 16.7 s; 48 threads
    18.0 s. Peak memory of the process: **160 MB** after the 12-thread runs (bar 200 MB), 333 MB after the 24–48-thread
    runs (more reports and listings in flight).
  - **The 10 s bar is not met.** A profile (`sample`, 12 threads) puts 92 % of the samples in `getattrlistbulk` and
    0.1 % waiting for the queue's lock (13 of about 10 000), with the process at 226 % CPU: the walk waits on the disk.
    The tree's 3 million entries are far more than the kernel keeps (`kern.maxvnodes` 263 168 here), so every scan of
    `LARGE` reads its metadata cold, while 01's probe tree and home were measured warm. So call 13 (the queue) is
    not revisited; the thread count and the bar are, with the user: on a cold tree more threads help (24 threads beat
    12), unlike the warm trees of 01.
