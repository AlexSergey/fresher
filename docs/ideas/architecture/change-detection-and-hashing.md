# Change detection and hashing

**Since:** 2026-11-15 · **Proposed by:** the agent

The question from the team: "to be fast we need a clever graph with hashes (md5) — but md5 is expensive". Hashes serve
two problems here: **fast rescans** (what changed since the last scan) and **duplicates** (which files have the same
content). The short answer: the graph is the scanner's tree with **fingerprints of metadata, not of content** — no
rescan reads file contents; contents are read only for duplicate candidates, and md5 is used nowhere.

Facts marked "measured" were tested on an Apple M1 Pro with APFS on 2026-11-15. The options and five whole architectures
were then put to a **calibrated decision model**, which answers typed questions with probabilities and no text, and
earlier to **two independent reviewer models** (see "How the options were scored"). The decision model's choices are the
verdict; the reviewers' scores stand next to them.

This is **measured input for plan [004](../../plans/004-fast-scan/readme.md)** (fast scan), whose first step, "Measure
first", asks for exactly these numbers. What they change there is in "What this means for plan 004".

## Measured

| Fact                                                                                      | Result                                                                                                                                                                                                                                                                           |
| ----------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Hash speed on a cached 1 GiB file, one core                                               | md5 0.53 GiB/s · SHA-256 (hardware) 1.48 GiB/s · BLAKE2b 0.60 GiB/s · plain read 6.5 GiB/s — md5 is 12 × slower than the read                                                                                                                                                    |
| `getattrlistbulk` with `FSOPT_ATTR_CMN_EXTENDED`                                          | returns `ATTR_CMNEXT_CLONEID`, `ATTR_CMNEXT_PRIVATESIZE` and `ATTR_CMNEXT_CLONE_REFCNT` in the same batch read                                                                                                                                                                   |
| A fresh clone (`cp -c`)                                                                   | the same clone id as the original, reference count 2, private size 0 for both                                                                                                                                                                                                    |
| The clone after 64 KiB of it is rewritten                                                 | **a new clone id**, reference count 1 for both, private size 64 KiB each — the other ~4 MiB of blocks are still shared                                                                                                                                                           |
| `ATTR_CMNEXT_RECURSIVE_GENCOUNT` on ordinary folders                                      | 0: maintained only for folders with directory statistics turned on, which is a privileged write — not usable by a read-only scan                                                                                                                                                 |
| Parallel `getattrlistbulk` walk of a home folder (1.74 million entries), cheap attributes | **12.5 s** warm · **24.9 s** cold, right after a restart (~70 000 entries/s, 11 MB of memory) · thread count (10 or 30) does not matter                                                                                                                                          |
| The same walk with private size asked for                                                 | **32–41 s** warm · **44.4 s** right after the cold cheap walk (partly warm, so a true cold run is slower)                                                                                                                                                                        |
| The same walk: logical, allocated and private size                                        | 1014 GiB logical, 103.9 GiB allocated, 98.6 GiB private — ~5.3 GiB sits in blocks shared between files; Docker's disk image alone is 460 GiB logical and 5.6 GiB allocated ([`../container-disk-image.md`](../container-disk-image.md)); logical size is useless as "space used" |

Not measured yet: BLAKE3 and xxh3 (see "To measure").

## What this means for plan 004

| Plan 004 (draft) says                                                         | Measured                                                                                               | Proposed                                                                                                                                                                                                                                              |
| ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| §1: one syscall returns logical, allocated and private sizes and the clone id | asking for private size and clone id makes the walk 2–3 × slower (24.9 s → 44.4 s+ cold)               | read them in a second tier, only where they matter ([sizes in two tiers](#sizes-in-two-tiers))                                                                                                                                                        |
| Target: first scan, warm, 2 million entries ≤ 5 s                             | 12.5 s warm and 24.9 s cold for 1.74 million entries on an M1 Pro; 10 or 30 threads make no difference | a cold target as well — the app is opened rarely, often after a restart ([target](#target)); the decision model gives a 5-second warm scan of 2 million entries P = 0.06                                                                              |
| Call 1: rescans from the system's change history (FSEvents)                   | a full cold walk already meets a 30-second target                                                      | needed only for the rescan targets (≤ 0.5 s unchanged, ≤ 2 s after a day); if kept, take R6 — its sampled re-check on top of the plan's total check ([the choice](#the-choice)); the decision model gives the history covering several weeks P = 0.07 |
| §2: 3 million files make about 400 thousand nodes                             | 1.42 million files in 270 thousand folders                                                             | consistent with the estimate                                                                                                                                                                                                                          |
| §1: interesting places first                                                  | —                                                                                                      | the reviews proposed the same; nothing to add                                                                                                                                                                                                         |
| Plan 002: `Sizes { logical, allocated, completeness }`                        | ~5.3 GiB of 104 GiB sits in blocks shared between files                                                | "freed if deleted" needs the private size, or a mark that it is a lower bound (C3)                                                                                                                                                                    |

**The two reviewer models scored R5 against a 30-second cold target only**; their "reject" means "not needed for that
target", not "unsafe". The decision model was given both targets ([the choice](#the-choice)): it prefers the cold target
(0.99 against 0.01) and gives plan 004's draft as written (T1 + R5 + C2) P = 0.78 of breaking a hard rule or risking a
false positive.

## The choice

The decision model was asked 158 typed questions over one brief — the constraints, the measured facts, every option,
five whole architectures and two targets: **target A**, a cold full scan within 30 s with first findings in 2–3 s, and
**target B**, plan 004's warm targets (a first scan of 2 million entries in 5 s, rescans in 0.5–2 s). It returns
probabilities, not text. Two runs gave identical answers. The architectures were asked again under neutral, shuffled
names (in brackets), so that a name like "with the measured fixes" cannot pull the answer.

**Target:** A, 0.99 against 0.01 — the app is opened rarely, so the cold scan is the one the user waits for.

| Architecture                                                       | Chosen, target A | Chosen, target B | Chosen overall  | Meets A | Meets B | Breaks a rule or risks a false positive |
| ------------------------------------------------------------------ | ---------------- | ---------------- | --------------- | ------- | ------- | --------------------------------------- |
| AR1. Full walk, two tiers: R1 + T2 + C3, R3 later                  | **0.77 (0.79)**  | 0.07 (0.22)      | **0.78 (0.79)** | 0.77    | 0.04    | 0.12                                    |
| AR2. Event history as drafted in plan 004: T1 + R5 + C2            | 0.02 (0.02)      | 0.04 (0.07)      | 0.03 (0.02)     | 0.09    | 0.05    | 0.78                                    |
| AR3. Event history with the measured fixes: T2 + R6 + C3           | 0.03 (0.05)      | 0.78 (0.39)      | 0.04 (0.03)     | 0.42    | 0.20    | 0.26                                    |
| AR4. Full walk with fingerprints from the start: R1 + R3 + T2 + C3 | 0.10 (0.09)      | 0.09 (0.27)      | 0.10 (0.08)     | 0.51    | 0.08    | 0.07                                    |
| AR5. Full walk, all attributes: R1 + T1 + C3                       | 0.07 (0.06)      | 0.03 (0.05)      | 0.06 (0.08)     | 0.05    | 0.01    | 0.21                                    |

- **AR1 is chosen** for target A and overall (0.77–0.79, unchanged under neutral names) and is the most likely to meet
  target A (0.77; AR4, the same with fingerprints from the start, 0.51).
- **Plan 004 as drafted (AR2)** is chosen almost never, meets neither target, and carries the highest risk (0.78). The
  measurements say why: reading private size and clone id for every entry costs the cold scan, and counting a clone id
  once miscounts partly rewritten clones.
- **No architecture is likely to meet target B** (best 0.20). If target B stays, AR3 is its best shape, but with little
  margin: 0.78 under the given names, 0.39 under neutral ones.

| Group                | Chosen | Probability | Next    |
| -------------------- | ------ | ----------- | ------- |
| Rescans, target A    | **R1** | 0.89        | R3 0.06 |
| Rescans, target B    | **R6** | 0.47        | R5 0.33 |
| APFS size attributes | **T2** | 0.98        | T1 0.02 |
| Clones and "freed"   | **C3** | 0.96        | C2 0.03 |
| Duplicates           | **D2** | 0.89        | D4 0.05 |
| Full-file hash       | **H2** | 0.81        | H4 0.11 |
| Acting on duplicates | **A1** | 0.99        | A2 0.01 |

The per-criterion scores in the tables below are softer than the choices: the decision model's confidence in a single
score level is 0.22–0.40, while its choices above are made with 0.77–0.99 (rescans under target B: 0.47).

## Target

Proposed, not decided yet, next to plan 004's warm targets: a **cold full scan of a home folder finishes within 30
seconds**, and the **first findings appear within 2–3 seconds**. The walk visits the folders rules care about first,
findings stream as it reaches them, and totals are marked partial until it ends. The measured home folder (1.74 million
entries) meets it with the cheap walk (24.9 s); a home folder of 3–5 million entries would take ~43–72 s cold, so there
the streaming carries the experience, not the total time.

## How the options were scored

The decision model and each reviewer scored each option from 1 (bad) to 5 (great) on five criteria, weighted by what
matters to this product: **correctness** × 3 (stale or wrong sizes, false duplicates, unsafe deletion), **cold** speed ×
2 (the app is opened rarely, so a cold cache is the normal case), **warm** speed × 1, **simplicity** × 1 (deterministic
fixture-tree tests included), **fit** × 1 (no daemon, no network, privacy). The maximum is 40. Cells read
`correctness·cold·warm·simplicity·fit = total`; the decision model's scores are probability-weighted, so they have
decimals. The total ranks options within a group; the verdict follows the decision model's choice in "The choice", and
the reviewers' own calls where it was not asked ("later" for D3, D4 and A2). A 1 on correctness or fit means a hard rule
is broken, so the option is rejected whatever its total (R8 and R9 for reviewer A).

### Fast rescans

| Option                                                 | Decision model                 | Reviewer A         | Reviewer B         | Verdict                                                                    |
| ------------------------------------------------------ | ------------------------------ | ------------------ | ------------------ | -------------------------------------------------------------------------- |
| R1. Full metadata walk every scan                      | 4.6·3.3·2.9·4.6·4.8 = **32.7** | 5·3·4·5·5 = **35** | 5·4·4·5·5 = **37** | **take**: the cold walk measured 24.9 s                                    |
| R2. Merkle tree of content hashes                      | 3.8·1.3·2.8·3.9·2.7 = **23.5** | 5·1·1·2·2 = **22** | 5·1·1·4·1 = **23** | reject: reads every byte on every scan                                     |
| R3. Merkle tree of metadata fingerprints               | 3.8·3.3·4.2·3.7·4.6 = **30.3** | 4·3·4·3·4 = **29** | 5·4·4·4·4 = **35** | **later, with watch mode** (plan 010): speeds up diffs, not the walk       |
| R4. Skip a folder whose modification time is unchanged | 2.7·3.3·3.4·3.1·3.1 = **24.4** | 1·4·4·4·5 = **24** | 1·4·5·4·2 = **22** | reject: a file growing in place does not change its folder's time          |
| R5. Replay the file-system event history (FSEvents)    | 3.3·2.3·4.4·2.0·4.2 = **25.1** | 3·4·5·2·3 = **27** | 3·4·5·2·2 = **26** | reject for target A; for target B take R6 instead                          |
| R6. R5 + a random-sample re-check                      | 3.2·2.7·3.6·2.2·4.4 = **25.3** | 4·4·5·1·3 = **29** | 4·4·5·1·2 = **28** | **only for target B** (0.47, R5 0.33): hardest to test                     |
| R7. The Spotlight index                                | 1.9·3.0·3.4·1.7·2.9 = **19.6** | 2·4·4·3·3 = **24** | 2·5·5·3·4 = **28** | reject: incomplete (excluded folders, package contents, can be off)        |
| R8. A live watcher or a security-framework client      | 3.0·3.9·4.5·1.3·1.1 = **23.8** | 5·5·5·2·1 = **33** | 5·1·5·1·1 = **24** | reject: a background process ([`../../goal.md`](../../goal.md), section 8) |
| R9. APFS recursive generation counts of folders        | 2.7·2.5·2.2·2.3·3.6 = **21.1** | 5·5·5·4·1 = **35** | 2·3·5·2·2 = **21** | reject: measured as 0 unless a privileged write turns them on              |

- **R1 is the whole rescan story for now.** The cold walk meets the target, so nothing is skipped and nothing about the
  previous scan has to be trusted.
- **R3 in detail, for watch mode.** Each node of the arena tree ([scanning](./scanning.md)) gets a 64-bit fingerprint:
  xxh3 of `(device, inode, logical size, allocated size, mtime, ctime)` for a file; for a folder, xxh3 of its children's
  `(name, fingerprint)` in name order. Equal fingerprints at a folder mean an equal subtree, so the diff of two scans
  ([snapshots and watch mode](./snapshots-and-watch-mode.md)) skips it. Stored per folder in plan 004's cache file, they
  let the merge pass of plan [010](../../plans/010-history/readme.md) skip equal subtrees.
- **If R5 is kept** (plan 004, call 1): the plan already walks in full when the history was reset or dropped events, the
  volume changed, the cache is missing, or the patched total disagrees with the volume's used space. The reviewers add
  one more check: re-read a random sample of folders the history calls unchanged, and walk in full on any mismatch.
  Whether the history reaches back a week or a month on a busy Mac is **unverified** — the question plan 004 asks in its
  first step.

### When to read the APFS size attributes

| Option                                                           | Decision model                 | Reviewer A         | Reviewer B         | Verdict                                              |
| ---------------------------------------------------------------- | ------------------------------ | ------------------ | ------------------ | ---------------------------------------------------- |
| T1. Clone id and private size for every entry in the walk        | 3.7·2.1·2.4·4.2·3.6 = **25.4** | 5·2·3·5·5 = **32** | 5·2·2·5·5 = **31** | reject: 2–3 × slower walk, misses the target         |
| T2. Two tiers: cheap walk, extended attributes only where needed | 4.0·4.1·3.8·3.5·4.6 = **32.0** | 5·4·4·3·5 = **35** | 5·4·4·4·5 = **36** | **take** ([sizes in two tiers](#sizes-in-two-tiers)) |

### Clones and "freed if deleted"

| Option                                                     | Decision model                 | Reviewer A         | Reviewer B         | Verdict                                                               |
| ---------------------------------------------------------- | ------------------------------ | ------------------ | ------------------ | --------------------------------------------------------------------- |
| C1. "Freed" = allocated size, clones as normal files       | 2.6·4.1·4.0·4.7·3.2 = **28.0** | 2·5·5·5·5 = **31** | 2·5·5·5·5 = **31** | reject: promises space a clone never gives back                       |
| C2. Each clone id counted once, "freed" = the group's size | 2.5·2.5·3.3·3.2·3.4 = **22.4** | 2·4·4·3·5 = **26** | 2·4·4·3·5 = **26** | reject: a partly rewritten clone has a new id and still shares blocks |
| C3. "Freed" = private size; shared files marked            | 4.1·3.8·3.8·2.6·4.7 = **31.1** | 5·4·4·4·5 = **36** | 4·4·4·4·5 = **33** | **take**, with the whole-group case below                             |

The measurements change decision 6 of the [architecture ideas](./README.md) ("APFS clone groups counted once per
`(device, clone id)`"):

- An **untouched** clone shares its clone id; a **partly rewritten** clone gets a new clone id while still sharing most
  of its blocks. Clone ids cannot tell how much is shared; only the private size can.
- So **"freed if deleted" is the private size**: a lower bound, shown as "at least". A file whose private size is below
  its allocated size is marked "shares space with another file".
- **The whole-group case** (raised by a reviewer): when every holder of an untouched clone id is in the selection — the
  selection holds as many of them as the reference count says — the group's allocated size is freed, not the sum of its
  private sizes (0). Partly rewritten clones cannot be resolved this way and stay at the lower bound.
- Freed space shows up only after the Trash is emptied ([the Trash holds the freed
  space](../trash-holds-freed-space.md)) and only for blocks no APFS local snapshot still holds.

### Duplicates

| Option                                                                   | Decision model                 | Reviewer A         | Reviewer B         | Verdict                                                         |
| ------------------------------------------------------------------------ | ------------------------------ | ------------------ | ------------------ | --------------------------------------------------------------- |
| D1. md5 of every file                                                    | 3.4·1.5·2.1·4.8·3.4 = **23.7** | 5·1·1·5·2 = **25** | 3·1·1·5·1 = **18** | reject: reads everything with the slowest hash                  |
| D2. Staged: size → links and clone ids out → prefix → suffix → full hash | 4.0·2.7·3.4·3.5·4.5 = **28.7** | 5·4·4·3·5 = **35** | 5·4·5·4·4 = **36** | **take** ([duplicates](./duplicates.md))                        |
| D3. Sampled chunks as an extra candidate stage                           | 3.6·2.3·3.2·3.1·4.2 = **25.8** | 5·4·4·2·5 = **34** | 5·4·4·3·5 = **35** | later, only if measured to cull more than prefix and suffix     |
| D4. Duplicate folders through a lazily built content-addressed graph     | 3.8·2.7·3.7·2.6·4.3 = **27.5** | 5·4·4·3·5 = **35** | 5·4·4·3·5 = **35** | **take after D2**: identical developer folders (`node_modules`) |
| D5. Content-defined chunking for near-duplicates                         | 2.9·1.7·2.2·1.9·4.0 = **20.4** | 4·1·1·1·2 = **18** | 3·1·2·1·2 = **16** | reject: we want exact matches                                   |
| D6. Group candidates by file extension                                   | 3.3·2.6·3.5·3.7·3.7 = **26.0** | 1·4·4·5·5 = **25** | 2·4·4·5·5 = **28** | reject: a copy under another extension still wastes space       |

- **D4 in detail.** First a cheap structural signature per folder: total size, file count and the sorted list of
  `(relative name, size)`. Only folders whose signatures collide get content hashes (from the D2 pipeline and its
  cache), and a folder's content hash is the hash of its children's `(name, content hash)`. Most folders never get past
  the signature.
- **Hashes:** xxh3 for the prefix and suffix stages; the hardware SHA-256 of Apple Silicon for the full file (the
  decision model's choice, below; 1.48 GiB/s per core keeps up with the disk on a few cores), BLAKE3 only if it measures
  clearly faster; md5 nowhere.
- **The hash cache key gains `ctime`**: `(device, inode, size, mtime, ctime, range)` — a rewrite that restores the
  modification time still changes the inode change time.
- **Hashing reads bypass the page cache** (`F_NOCACHE`), so gigabytes of file contents do not push the file-system
  metadata out of memory in the middle of a scan.

### The hash for full files

| Option                                                      | Decision model                 | Verdict                                                                    |
| ----------------------------------------------------------- | ------------------------------ | -------------------------------------------------------------------------- |
| H1. md5                                                     | 3.5·2.2·2.3·4.4·4.1 = **25.7** | reject: slowest, broken                                                    |
| H2. SHA-256 with the hardware instructions of Apple Silicon | 4.5·4.0·3.8·4.0·4.6 = **33.8** | **take** (0.81): no extra dependency; confirmed by measurement in the plan |
| H3. BLAKE3, multi-threaded                                  | 4.4·3.5·3.8·3.7·4.5 = **32.4** | only if it measures clearly faster                                         |
| H4. xxh3 for the prefix and suffix stages, then H2 or H3    | 4.1·3.8·4.1·3.1·4.7 = **31.9** | **take** for the prefix and suffix stages                                  |

The reviewer models were not asked about the hash.

### Acting on duplicates

| Option                                                 | Decision model                 | Reviewer A         | Reviewer B         | Verdict                                                                             |
| ------------------------------------------------------ | ------------------------------ | ------------------ | ------------------ | ----------------------------------------------------------------------------------- |
| A1. Move chosen copies to the Trash                    | 4.4·3.6·3.6·4.5·4.8 = **33.4** | 5·5·5·5·5 = **40** | 5·5·5·5·5 = **40** | **take**: the full hash is re-checked right before                                  |
| A2. Replace a copy with an APFS clone of the kept file | 4.1·4.5·3.5·2.9·4.3 = **32.0** | 4·5·5·3·5 = **35** | 3·5·5·2·5 = **31** | later: keeping permissions, extended attributes and hard links exact is error-prone |

## Sizes in two tiers

The measured walk changes decision 1 of the [architecture ideas](./README.md) (clone id and private size "in the same
syscall"): asking for the APFS extended attributes makes the walk about **2–3 × slower** (24.9 s → 44.4 s or more cold).
So (T2):

- **The walk reads the cheap attributes only**: name, type, device, inode, link count, logical and allocated size,
  times. Totals and the tree use the allocated size.
- **Private size and clone id are read later, only where they matter**: for the subtrees a rule claims (one bulk read of
  those folders with extended attributes), for duplicate candidates of the same size, and in the re-check right before
  deletion, which reads the item again anyway ([safety in the types](./safety-in-types.md)) — through an open file
  descriptor (`O_NOFOLLOW`, then `fstat`), so the item checked is the item deleted.
- Until the private size of a finding is read, the finding shows its allocated size marked "may share space with other
  files" — honest, and replaced within seconds.

## What the reviewers changed

Taken:

- R3 moved from "taken" to **later, with watch mode** — both reviewers: it speeds up diffs, not the cold walk.
- The **whole-group case** for clones, the **"at least"** wording, and the note on Trash and local snapshots.
- The walk visits **rule anchors first**, so first findings come in 1–2 s — already in plan 004, §1.
- **`F_NOCACHE`** for hashing reads; **BLAKE3 vs hardware SHA-256** decided by measurement.
- The re-check before deletion goes **through a file descriptor** (`O_NOFOLLOW` + `fstat`).
- The 30-second target is honest only up to ~2 million entries; larger home folders rely on streaming.

Rejected:

- "Skip deep system and app folders to save time": totals must be an exact remainder, with no "Other"
  ([`../../rules.md`](../../rules.md)), and caches are where most findings are.
- "A single-threaded cold walk caps at 70 000 entries/s": the measured walk ran 30 threads in parallel; the cap is the
  file system, not the thread count.

## To measure

The spike (kept out of the repository) walks a home folder in parallel with `getattrlistbulk` and only counts. Still
open, all for the plan:

- The extended-attribute walk **right after a restart** (the 44.4 s above was partly warm).
- Walker threads at user-initiated QoS on performance cores — a reviewer suspects the flat thread curve comes from
  efficiency cores or lock contention.
- BLAKE3 and xxh3 speed against the hardware SHA-256.

Open questions:

- The hash cache of duplicates: kept on disk between scans (inside plan 004's budget), or in memory only (a reviewer's
  privacy argument)?
- Are per-folder fingerprints (8 bytes each) worth their place in plan 004's cache file, for plan 010's comparisons?