# Change detection and hashing

**Since:** 2026-11-15 · **Proposed by:** the agent

The question from the team: "to be fast we need a clever graph with hashes (md5) — but md5 is expensive". Hashes serve
two problems here: **fast rescans** (what changed since the last scan) and **duplicates** (which files have the same
content). The short answer: the graph is the scanner's tree with **fingerprints of metadata, not of content**; contents
are read only for duplicate candidates, and md5 is used nowhere.

The options were scored by the agent and checked against an independent review by a second model; the facts below
marked "measured" were tested on an Apple M1 Pro with APFS on 2026-11-15.

## Measured

| Fact                                                                                         | Result                                                                                                                                                                                                           |
| -------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Hash speed on a cached 1 GiB file, one core                                                  | md5 0.53 GiB/s · SHA-256 (hardware) 1.48 GiB/s · BLAKE2b 0.60 GiB/s · plain read 6.5 GiB/s — md5 is 12 × slower than the read                                                                                    |
| `getattrlistbulk` with `FSOPT_ATTR_CMN_EXTENDED`                                             | returns `ATTR_CMNEXT_CLONEID`, `ATTR_CMNEXT_PRIVATESIZE` and `ATTR_CMNEXT_CLONE_REFCNT` in the same batch read                                                                                                   |
| A fresh clone (`cp -c`)                                                                      | the same clone id as the original, reference count 2, private size 0 for both                                                                                                                                    |
| The clone after 64 KiB of it is rewritten                                                    | **a new clone id**, reference count 1 for both, private size 64 KiB each — the other ~4 MiB of blocks are still shared                                                                                           |
| `ATTR_CMNEXT_RECURSIVE_GENCOUNT` on ordinary folders                                         | 0: maintained only for folders with directory statistics turned on, which is a privileged write — not usable by a read-only scan                                                                                 |
| Parallel `getattrlistbulk` walk of a home folder (1.73 million entries), metadata cache warm | **12.5 s** with names, types, sizes and link counts · **32–41 s** when private size is also asked for · thread count (10 or 30) does not matter                                                                  |
| The same walk: logical vs allocated size                                                     | 1014 GiB logical, 103.6 GiB allocated — Docker's disk image alone is 460 GiB logical and 5.6 GiB allocated ([`../container-disk-image.md`](../container-disk-image.md)); logical size is useless as "space used" |

Not measured yet: BLAKE3 (no tool installed; its multi-threaded SIMD speed is expected well above the read speed —
check in the plan), and the walk with a cold metadata cache, right after a restart (see "To measure").

## Fast rescans

| Option                                                                  | Verdict                                                                                    |
| ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| R1. Full walk every time                                                | **first choice**, until a cold-cache measurement says it is too slow                       |
| R2. Merkle tree of content hashes                                       | rejected: reads every byte of the disk on every scan                                       |
| R3. Merkle tree of metadata fingerprints                                | **taken**: nearly free during the walk; makes diffs proportional to what changed           |
| R4. Skip a folder whose modification time is unchanged                  | rejected: a file growing in place does not change its folder's time — sizes would be stale |
| R5. Replay the file-system event history (FSEvents) since the last scan | **only if R1 is too slow cold**, behind a strict fallback (below)                          |
| R6. R5 + R3 + a verification sample of "unchanged" folders              | the shape R5 must take if it is ever built                                                 |
| R7. The Spotlight index                                                 | rejected: incomplete (excluded folders, package contents, can be off)                      |
| R8. A live watcher or a security-framework client                       | rejected: a background process ([`../../goal.md`](../../goal.md), section 8)               |
| R9. APFS recursive generation counts of folders                         | rejected: measured as 0 unless a privileged write turns them on                            |

- **R3 in detail.** Each node of the arena tree ([scanning](./scanning.md)) gets a 64-bit fingerprint: xxh3 of
  `(device, inode, logical size, allocated size, mtime, ctime)` for a file; for a folder, xxh3 of its children's
  `(name, fingerprint)` in name order. Equal fingerprints at a folder mean an equal subtree, so the diff of two scans
  ([snapshots and watch mode](./snapshots-and-watch-mode.md)) skips it. Only the fingerprints of the summary tree are
  stored between scans ([snapshot summaries](./snapshot-summaries.md)), so no full path list is kept for this.
- **R1 is measured cold, not warm.** The app is opened rarely — after days, often after a restart — so the honest
  case is a cold metadata cache. A warm rescan seconds later says little.
- **R5's fallback, if it is built:** keep the last event id and the volume's event-store UUID; walk in full when the
  UUID changed, the ids wrapped, the history was dropped or the root changed; re-check a random sample of folders the
  history calls unchanged, and walk in full on any mismatch. Whether the history survives a week on a busy Mac is
  **unverified**.

## Duplicates

| Option                                                                        | Verdict                                                          |
| ----------------------------------------------------------------------------- | ---------------------------------------------------------------- |
| D1. md5 of every file                                                         | rejected: reads everything, slowest hash                         |
| D2. Staged: size → hard links and clone ids out → prefix → suffix → full hash | **taken** ([duplicates](./duplicates.md))                        |
| D3. Sampled chunks as an extra candidate stage                                | optional, only if measured to help; never enough to act on       |
| D4. Duplicate folders through a lazily built content-addressed graph          | **taken** for developer folders (identical `node_modules`)       |
| D5. Content-defined chunking for near-duplicates                              | rejected: we want exact matches                                  |
| Grouping candidates by file extension                                         | rejected: a duplicate under another extension still wastes space |

- **D4 in detail.** First a cheap structural signature per folder: total size, file count and the sorted list of
  `(relative name, size)`. Only folders whose signatures collide get content hashes (from the D2 pipeline and its
  cache), and a folder's content hash is the hash of its children's `(name, content hash)`. Most folders never get
  past the signature.
- **Hashes:** xxh3 for the prefix and suffix stages, BLAKE3 for the full hash; md5 nowhere.
- **The hash cache key gains `ctime`**: `(device, inode, size, mtime, ctime, range)` — a rewrite that restores the
  modification time still changes the inode change time.

## Clones and "freed if deleted"

The measurements change decision 6 of the [architecture ideas](./README.md) ("APFS clone groups counted once per
`(device, clone id)`"):

- An **untouched** clone shares its clone id; counting the group once is right.
- A **partly rewritten** clone gets a new clone id while still sharing most of its blocks. Clone ids then cannot tell
  how much is shared; only the private size can.
- So: **"freed if deleted" is the private size**, always. Totals count each clone id once, and a file whose private
  size is below its allocated size is marked "shares space with another file" instead of pretending its whole size
  would come back.
- Two files with one clone id are not duplicates to remove: deleting one frees nothing (D2 already drops them).

## Sizes in two tiers

The measured walk changes decision 1 of the [architecture ideas](./README.md) (clone id and private size "in the same
syscall"): asking for the APFS extended attributes makes the whole walk about **3 × slower**. So:

- **The walk reads the cheap attributes only**: name, type, device, inode, link count, logical and allocated size,
  times. Totals and the tree use the allocated size.
- **Private size and clone id are read later, only where they matter**: for the subtrees a rule claims (the finding's
  "freed if deleted" — one bulk read of those folders with extended attributes), for duplicate candidates of the same
  size, and in the re-check right before deletion, which reads the item again anyway
  ([safety in the types](./safety-in-types.md)).
- Until the private size of a finding is read, the finding shows its allocated size marked "may share space with
  other files" — honest, and replaced within seconds.

## To measure

A throwaway spike (kept out of the repository) walks a home folder in parallel with `getattrlistbulk` and only
counts — entries, folders, sizes, errors by kind, elapsed time. Warm results are above; still needed: **one run right
after a restart** (cold), which decides between R1 alone and R1 + R5. If a cold walk without extended attributes stays
under the target, event-history replay is not needed at all.

Open questions:

- The target for a cold full scan of a home folder: 30 seconds?
- The summary tree's fingerprints: enough for watch mode, or does R3 need the full tree of the latest scan?
