# Intelligent Storage Explainer

**Since:** 2026-11-15 · The most important feature for non-technical users.

## Problem

macOS shows opaque categories such as "System Data — 42.8 GB".

## Direction

Explain every gigabyte:

```
System Data — 42.8 GB
  12.4 GB  APFS local snapshots
   8.2 GB  app data
   7.1 GB  system and user logs
   6.3 GB  caches
   4.8 GB  development data
   4.0 GB  unreadable without Full Disk Access   ← never "not classified"
```

For each part: what it is, who created it, what it is for, whether it can be removed, what happens if it is, and the
**confidence** of the classification.

## Rules

- **Consistent and never double-counted** (the user's requirement): the parts add up to the total; a byte belongs to one
  part. Clones, hard links and snapshot-held blocks are counted once.
- **Not everything in "System Data" maps to files** (purgeable space, snapshot-held blocks, swap): those parts come from
  volume figures, are labelled as such, and are never offered as files to delete.
- **No "not classified" bucket** ([`../rules.md`](../rules.md), rule 3): a remainder is named for what it is — unreadable
  without Full Disk Access, owned by root, or the real folders that hold it.
- Extends [`system-data-analyzer.md`](./system-data-analyzer.md) from System Data to the whole disk.
