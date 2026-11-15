# Cleanup Simulator

**Since:** 2026-11-15

## Problem

Before deleting anything, the user wants to see the result and play with the choice of categories.

## Direction

- A virtual result that updates as categories are ticked and unticked — nothing is deleted:

  ```
  Cleanup simulation
  8.42 GB  estimated space freed
    Cache files        3.2 GB
    Old logs           1.4 GB
    Temporary files    2.1 GB
  Space before   12.4 GB free
  Space after    20.82 GB free
  ```

- **An estimate, not a guarantee**, and labelled as one: the freed space is computed from each item's private size
  (APFS clones, hard links and snapshots share blocks, so deleting them frees less than their size —
  [`architecture/scanning.md`](./architecture/scanning.md)).
- Every line is a named category — no "Other candidates" line ([`../rules.md`](../rules.md), rule 3).
- Merges naturally with the preview screen of [`cleanup-preview-and-safe-removal.md`](./cleanup-preview-and-safe-removal.md):
  the simulation is the preview before the confirmation.

Open questions:

- When a local snapshot still holds the blocks, the space comes back only after the snapshot goes: show that case
  explicitly?
