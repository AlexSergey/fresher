# Duplicates

**Since:** 2026-11-15

## The staged pipeline

1. **Group by size**; a size seen once cannot have a duplicate.
2. **Drop hard links** (same device and inode) and **identical APFS clone ids** (they share their blocks — deleting one
   frees nothing).
3. **Prefix hash** of the first 4 KiB.
4. **Suffix hash** of the last 4 KiB, only for files above 64 KiB.
5. **Full hash**.

Most files leave the pipeline at step 1 or 3 and are never read in full. Block sizes and parallelism can be tuned per
device type (an SSD takes more parallel reads than a spinning disk).

## Hashes

- BLAKE3 for the full hash: fast, wide, no collisions to worry about.
- A fast non-cryptographic hash (xxh3) is fine for the prefix and suffix stages; full matches are always confirmed by
  the full hash.

## Hash cache

- Cache hashes between runs, keyed by `(device, inode, chunk position, chunk length)`; an entry is valid only while
  size and modification time match.
- Cache only files above a minimum size; small files are cheaper to rehash than to look up.
- Our cache lives in the same SQLite store as the snapshots.

## Acting on duplicates

- A two-step flow: a report the user reviews, then an action on it.
- Besides "move to Trash", offer **replace with an APFS clone** (`clonefile`): every path keeps working, the space is
  freed.
- Report "space freed by deleting one" from the private size, not from the file size.

## Similar images

- Perceptual hashes (several algorithms and hash sizes to choose from), candidates found with a BK-tree over Hamming
  distance; groups shown side by side.
- An optional feature: it reads and decodes every image, so it is slow and opt-in.
