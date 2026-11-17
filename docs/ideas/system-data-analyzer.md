# System Data Analyzer

**Since:** 2026-11-15

## Problem

macOS Storage settings show a large, unexplained "System Data". It hides APFS local snapshots, purgeable space, caches,
virtual memory, sleep image, Time Machine data and more. This is exactly the "Other" bucket the project exists to
explain ([`../goal.md`](../goal.md), section 1).

## Direction

- Break "System Data" into named parts, each with a plain-English explanation and whether the user can act on it:
  - APFS local snapshots (Time Machine and system update snapshots) — list with dates and sizes; thinning them is a
    **Medium** action through `tmutil`, never by deleting files;
  - purgeable space — what macOS will free by itself when needed (information, no action);
  - virtual memory swap files and the sleep image — information only, never removed;
  - system caches and logs readable without root — shown, cleanable per [`system-junk-cleaner.md`](./system-junk-cleaner.md);
  - iOS device backups, simulators, Docker's disk image — linked to their own categories.
- A reconciliation line: "macOS says X GB used; we explain Y GB; the rest is what we cannot read without Full Disk
  Access / root" — the remainder is named for what it is, never called "Other".

Open questions:

- Which numbers can be read without root, and how exact is the purgeable figure (volume capacity APIs)?

**Partly taken by [plan 012](../plans/012-storage-map/readme.md):** the system's volumes, the virtual memory volume, purgeable space beside Free and the reasons of the remainder (unreadable folders, snapshots and bookkeeping) are on the map as information, never as files to delete. **Still an idea:** listing local snapshots with their dates and sizes, and thinning them through `tmutil`.
