# Storage History

**Since:** 2026-11-15 · The user's title: "Storage History & Time Machine" — renamed, since Time Machine is the name of
Apple's backup.

## Problem

"The disk was fine a month ago" — but what changed since?

## Direction

- The disk's state over time: yesterday, 7 days, 30 days, 90 days, a year — broken down by app and category.
- "In the last month Docker grew by 24 GB, Chrome by 3.2 GB."
- Compare any two saved states and see exactly what changed (added, removed, grown), down to folders.
- Built on watch mode: every scan saves a snapshot
  ([`architecture/snapshots-and-watch-mode.md`](./architecture/snapshots-and-watch-mode.md)); totals of every scan are
  kept, full trees for recent scans only.
- History has points only where the user scanned: no background scans ([`../goal.md`](../goal.md), section 8). The
  timeline says so ("no scan between 3 and 17 March").

Open questions:

- Retention: how many full trees, and the size budget of the database.
