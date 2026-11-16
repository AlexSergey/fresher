# Scanning

**Since:** 2026-11-15

**Taken by plan [004](../../plans/004-fast-scan/readme.md)** (2026-11-16): the bulk read per batch of entries, the
three sizes (the private size read only for files that may share blocks), hard links and clones counted once, one
device per root with nested roots dropped, errors collected by kind, and the arena tree (72 bytes a node). Changed by
measurement: one `Mutex` queue instead of work stealing, and one thread per core (at most 16) instead of 3 × the cores,
which was slower on a warm tree.

## Parallel traversal

- **Work-stealing beats a sorted parallel walk.** One leading Rust analyzer replaced a rayon-based sorted walker with
  its own work-stealing walker on `crossbeam-deque` and reports up to 40 % faster scans.
- **Thread count**: about 3 × the cores, capped at 64. More threads help a cold cache (the disk waits); too many hurt a
  warm one. Another tool limits concurrency with a semaphore of 2 × the cores.
- **Results flow to one aggregator** over a channel; workers never share the tree.

## Reading a folder in one syscall

- On macOS, `getattrlistbulk(2)` reads a batch of directory entries with their attributes in one call: name, device,
  file id (inode), link count, `ATTR_FILE_ALLOCSIZE`, `ATTR_FILE_DATAALLOCSIZE`, and with `FSOPT_ATTR_CMN_EXTENDED`
  the extended attributes `ATTR_CMNEXT_CLONEID` and `ATTR_CMNEXT_PRIVATESIZE`. No separate `lstat` per entry.
- Pitfalls:
  - entries are packed with 4-byte alignment inside 8-byte-aligned records;
  - `ATTR_CMN_ERROR` reports a per-entry error inside the buffer — check it before reading the rest;
  - the device id comes back signed.

## Sizes

- **Logical size** (`st_size`) — what the user expects from Finder.
- **Allocated size** (`st_blocks × 512`) — what the file takes on disk; small files take a whole block, sparse files
  take less than their logical size. Cap an absurd block count (a known quirk of some foreign filesystems).
- **Private (exclusive) size** — `ATTR_CMNEXT_PRIVATESIZE`: the bytes not shared with a clone or a snapshot, freed
  immediately if the file is deleted. This is the honest "you will get back" number.
- Every size is marked complete, partial or unknown; an unreadable folder makes its parent's size partial (`42 GB+`),
  never silently smaller.

## Hard links and clones

- Track `(device, inode)` only when a file's link count is above 1: the set stays small.
- On Darwin a **directory's link count is its child count**, not a link count — do not treat directories as hard links
  by that number.
- APFS clones: count a data fork once per `(device, clone id)`; resource forks stay private.

## Errors

- Collect errors by kind (permission denied, not found, interrupted) and keep walking; retry `Interrupted` with a cap.
- Show them as a result, not a failure: "312 folders could not be read — grant Full Disk Access to see them".

## Boundaries

- Stay on the root's device by default: compare each entry's device with the root's.
- Reject nested or overlapping roots: on macOS `/` and `/System/Volumes/Data` overlap through firmlinks.
- Never follow symlinks.

## Memory layout

- An arena of nodes of about 64 bytes: 32-bit indices for parent, first child and next sibling; names in one shared
  byte buffer; no `PathBuf` per node, no general graph library. Paths are rebuilt on demand.
- Millions of entries fit in a few hundred megabytes at most.
