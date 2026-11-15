# Process-to-Storage Timeline

**Since:** 2026-11-15

## Problem

"What did I do right before the disk ran out?"

## Direction

- A timeline that joins file events, processes and storage history:

  ```
  10:00  Docker started
  10:02  +300 MB of new data
  10:05  Chrome created 120 MB of cache
  10:10  Xcode started generating DerivedData
  10:15  disk usage up 1.4 GB
  ```

- Same feasibility and rules as [`live-storage-attribution.md`](./live-storage-attribution.md): per-process detail needs
  Endpoint Security or root; recording only while the app is open and only after the user starts it — no background
  recording ([`../goal.md`](../goal.md), section 8).
- Between recordings, the timeline falls back to scan snapshots: coarser, but honest about it.

Open questions:

- Does this survive the no-background rule in a useful form (the user must start recording before the problem)?
