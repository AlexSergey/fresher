# 004/01/01 — The bulk read

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-15 · **Modules:** macos (probe)

## Files

- `NO_COMMIT/probes/004/bulk-read/` — a Cargo project outside the workspace (its own `Cargo.toml`, `libc` only):
  - `generate` — builds a tree of 1 million entries in a temporary folder (sparse files, a few hard links, a few
    clones made with `clonefile`, a folder of 10 000 entries, a deep chain of 200 folders);
  - `walk-std` — a single-threaded walk with `read_dir` + `symlink_metadata`;
  - `walk-bulk` — the same walk with `getattrlistbulk`, single-threaded, then with N threads over a `Mutex` queue
    (N = cores, 2 ×, 3 ×, 4 ×, 64);
  - `walk-home` — `walk-bulk` over the developer's home folder, printing counts and timings only.

## Questions to answer

1. Which attributes come back for files, folders and links: name, kind, device, file id, link count, modification
   time, flags, logical size, allocated size; with `FSOPT_ATTR_CMN_EXTENDED`, the clone id and the private size. Is
   the private size there for every file, or only for clones? What does a folder report?
2. The layout of a record: lengths, the 4-byte packing inside 8-byte-aligned records, the returned-attributes mask,
   `ATTR_CMN_ERROR` on one entry; the device id's sign.
3. The buffer size: entries per call and time for 16, 64 and 256 KiB.
4. Cloud files (an iCloud Drive folder, if present): which flag marks a file whose data is not local
   (`SF_DATALESS`); what the sizes say; that with `setiopolicy_np(IOPOL_TYPE_VFS_MATERIALIZE_DATALESS_FILES, …OFF)` set
   nothing is downloaded (the network activity of `bird`/`fileproviderd` stays flat).
5. Whether `IOPOL_TYPE_VFS_ATIME_UPDATES` exists on the oldest supported macOS and stops access-time updates of the
   folders read.
6. Timings, warm (second run): std walk vs bulk walk, single-threaded; the bulk walk by thread count; the folders per
   second; CPU time against wall time. Cold numbers only if the developer reboots; no `purge` (it needs root).
7. Lock contention of the `Mutex` queue at the best thread count (time spent waiting for the lock).
8. Errors: a folder with mode `000`, a folder replaced by a link between the listing and the open, a folder removed
   during the read.

## Produces

A table of numbers and answers in the probe's `RESULTS.md` (in `NO_COMMIT/`), read by step 04: the attribute set and
record layout plan 03/02 implements; the buffer size; the thread count; whether a work-stealing queue is worth a
dependency; the file-size threshold for folding (the share of bytes in files under 256 KiB, 1 MiB and 4 MiB in the
home folder).

## Checks

- Every question above has an answer or a stated "could not tell, because …".
- The bulk walk's totals (entries, logical bytes) equal the std walk's on the generated tree.
- No probe output contains a name or a path from the home folder (a `grep` for the home path in the results is empty).
