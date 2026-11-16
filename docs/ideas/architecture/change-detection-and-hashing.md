# Change detection and hashing

**Since:** 2026-11-15 · **Proposed by:** the agent

The question from the team: "to be fast we need a clever graph with hashes (md5) — but md5 is expensive". Hashes serve
two problems here: **fast rescans** (what changed since the last scan) and **duplicates** (which files have the same
content). The short answer: the graph is the scanner's tree with **fingerprints of metadata, not of content** — no
rescan reads file contents; contents are read only for duplicate candidates, and md5 is used nowhere.

The facts below were measured with a throwaway spike on an Apple M1 Pro with APFS on 2026-11-15. Plan
[004](../../plans/004-fast-scan/readme.md) measured the scan on the reference Mac in its first step; where the numbers
differ, the plan's are the ones that count.

## Measured

| Fact                                                                                      | Result                                                                                                                                          |
| ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| Hash speed on a cached 1 GiB file, one core                                               | md5 0.53 GiB/s · SHA-256 (hardware) 1.48 GiB/s · BLAKE2b 0.60 GiB/s · plain read 6.5 GiB/s — md5 is 12 × slower than the read                   |
| `getattrlistbulk` with `FSOPT_ATTR_CMN_EXTENDED`                                          | returns `ATTR_CMNEXT_CLONEID`, `ATTR_CMNEXT_PRIVATESIZE` and `ATTR_CMNEXT_CLONE_REFCNT` in the same batch read                                  |
| A fresh clone (`cp -c`)                                                                   | the same clone id as the original, reference count 2, private size 0 for both                                                                   |
| The clone after 64 KiB of it is rewritten                                                 | **a new clone id**, reference count 1 for both, private size 64 KiB each — the other ~4 MiB of blocks are still shared                          |
| `ATTR_CMNEXT_RECURSIVE_GENCOUNT` on ordinary folders                                      | 0: maintained only for folders with directory statistics turned on, which is a privileged write — not usable by a read-only scan                |
| Parallel `getattrlistbulk` walk of a home folder (1.74 million entries), cheap attributes | **12.5 s** warm · **24.9 s** cold, right after a restart (~70 000 entries/s, 11 MB of memory) · thread count (10 or 30) does not matter         |
| The same walk with private size asked for                                                 | **32–41 s** warm · **44.4 s** right after the cold cheap walk (partly warm, so a true cold run is slower)                                       |
| The same walk: logical, allocated and private size                                        | 1014 GiB logical, 103.9 GiB allocated, 98.6 GiB private — ~5.3 GiB sits in blocks shared between files; logical size is useless as "space used" |
| Docker's disk image                                                                       | 460 GiB logical, 5.6 GiB allocated ([why Docker does not give the space back](../container-disk-image.md))                                      |

Not measured: BLAKE3 and xxh3.

## A rewritten clone still shares blocks

The most useful finding, for plan 004 to check its clone accounting against:

- An **untouched** clone shares its clone id with the original. A **partly rewritten** clone gets a **new clone id**
  while still sharing most of its blocks. So clone ids cannot tell how much is shared; only the private size can, and
  "count each clone id once" (decision 6 of the [architecture ideas](./README.md)) miscounts partly rewritten clones.
- **"Freed if deleted" is the private size**: a lower bound, shown as "at least". A file whose private size is below
  its allocated size is marked "shares space with another file".
- **The whole-group case**: when every holder of an untouched clone id is in the selection — the selection holds as
  many of them as the reference count says — the group's allocated size is freed, not the sum of its private sizes
  (0). Partly rewritten clones cannot be resolved this way and stay at the lower bound.
- Freed space shows up only after the Trash is emptied
  ([the Trash holds the freed space](../trash-holds-freed-space.md)) and only for blocks no APFS local snapshot still
  holds.

## Fast rescans

| Option                                                  | Verdict                            | Why                                                                                                           |
| ------------------------------------------------------- | ---------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| R1. Full metadata walk every scan                       | **the baseline**                   | nothing about the previous scan has to be trusted; the measured cold walk is 24.9 s                           |
| R2. Merkle tree of content hashes                       | reject                             | reads every byte of the disk on every scan                                                                    |
| R3. Merkle tree of metadata fingerprints                | **later, with history** (plan 010) | speeds up comparing two scans, not the walk itself (details below)                                            |
| R4. Skip a folder whose modification time is unchanged  | reject                             | a file growing in place does not change its folder's time — the case the product exists to find               |
| R5. Replay the file-system event history (FSEvents)     | only with R6's check               | saves the walk only when the history covers the whole gap; after weeks without the app that is **unverified** |
| R6. R5 + re-read a random sample of "unchanged" folders | the shape R5 must take             | a sample that disagrees with the history means the history missed something: walk in full                     |
| R7. The Spotlight index                                 | reject                             | incomplete: excluded folders, package contents, can be turned off                                             |
| R8. A live watcher or a security-framework client       | reject                             | a background process ([`../../goal.md`](../../goal.md), section 8)                                            |
| R9. APFS recursive generation counts of folders         | reject                             | measured as 0 unless a privileged write turns them on                                                         |

- **R3 in detail.** Each folder of the tree gets a 64-bit fingerprint: xxh3 of
  `(device, inode, logical size, allocated size, mtime, ctime)` for a file; for a folder, xxh3 of its children's
  `(name, fingerprint)` in name order. Equal fingerprints at a folder mean an equal subtree, so a comparison of two
  scans ([history and watch mode](../../plans/010-history/readme.md)) skips it.
- **R5's fallbacks**: walk in full when the history was reset or dropped events, the volume changed, the cache is
  missing, or the patched total disagrees with the volume's used space; R6 adds the random sample on top.

## When to read the APFS size attributes

| Option                                                             | Verdict                        | Why                                                                        |
| ------------------------------------------------------------------ | ------------------------------ | -------------------------------------------------------------------------- |
| T1. Clone id and private size for every entry in the walk          | reject                         | asking for them makes the walk 2–3 × slower (24.9 s → 44.4 s or more cold) |
| T2. Two tiers: a cheap walk, extended attributes only where needed | **take** — plan 004's decision | plan 004 reads the private size only for files that may share blocks       |

## Duplicates

| Option                                                                   | Verdict          | Why                                                                             |
| ------------------------------------------------------------------------ | ---------------- | ------------------------------------------------------------------------------- |
| D1. md5 of every file                                                    | reject           | reads everything, with the slowest hash measured                                |
| D2. Staged: size → links and clone ids out → prefix → suffix → full hash | **take**         | most candidates drop out after a few KiB ([duplicates](./duplicates.md))        |
| D3. Sampled chunks as an extra candidate stage                           | only if measured | helps only if many same-size files share both their first and last 4 KiB        |
| D4. Duplicate folders through a lazily built content-addressed graph     | **after D2**     | identical developer folders (`node_modules`) become one finding (details below) |
| D5. Content-defined chunking for near-duplicates                         | reject           | reads everything; near-duplicates are not safe to remove                        |
| D6. Group candidates by file extension                                   | reject           | a copy under another extension still wastes the space                           |

| Hash                                                        | Verdict                | Why                                                                                      |
| ----------------------------------------------------------- | ---------------------- | ---------------------------------------------------------------------------------------- |
| H1. md5                                                     | reject                 | 12 × slower than a cached read, and broken                                               |
| H2. SHA-256 with the hardware instructions of Apple Silicon | **measure against H3** | no extra dependency; 1.48 GiB/s per core keeps up with the disk on a few cores           |
| H3. BLAKE3, multi-threaded                                  | **measure against H2** | not measured yet; take it only if it is clearly faster                                   |
| H4. xxh3 for the prefix and suffix stages                   | **take**               | the early stages need speed, not resistance to crafted collisions; the full hash decides |

| Acting on duplicates                                   | Verdict  | Why                                                                                |
| ------------------------------------------------------ | -------- | ---------------------------------------------------------------------------------- |
| A1. Move chosen copies to the Trash                    | **take** | the full hash is re-checked right before                                           |
| A2. Replace a copy with an APFS clone of the kept file | later    | keeping permissions, extended attributes, ACLs and hard links exact is error-prone |

- **D4 in detail.** First a cheap structural signature per folder: total size, file count and the sorted list of
  `(relative name, size)`. Only folders whose signatures collide get content hashes (from the D2 pipeline and its
  cache), and a folder's content hash is the hash of its children's `(name, content hash)`. Most folders never get
  past the signature.
- **The hash cache key gains `ctime`**: `(device, inode, size, mtime, ctime, range)` — a rewrite that restores the
  modification time still changes the inode change time.
- **Hashing reads bypass the page cache** (`F_NOCACHE`), so gigabytes of file contents do not push the file-system
  metadata out of memory in the middle of a scan.
- **The re-check before removal goes through an open file descriptor** (`O_NOFOLLOW`, then `fstat`), so the item
  checked is the item removed ([safety in the types](./safety-in-types.md)).

## Not in this spike

- The extended-attribute walk right after a restart (the 44.4 s above was partly warm).
- Walker threads at user-initiated QoS on performance cores: the flat curve from 10 to 30 threads may come from
  efficiency cores or lock contention.
- BLAKE3 and xxh3 against the hardware SHA-256.

## Open questions

For the owner to decide:

- **A cold-scan target?** The app is opened rarely, often after a restart, so the cold scan is the one the user waits
  for. For example: a cold full scan of a home folder within 30 s, first findings within 2–3 s. The spike's cold walk
  took 24.9 s for 1.74 million entries; a home folder of 3–5 million entries would take ~43–72 s at that rate.
- The duplicates' hash cache: kept on disk between scans, inside the data budget, or in memory only?
- Per-folder fingerprints (R3, 8 bytes per folder) in the tree cache, for history comparisons?
