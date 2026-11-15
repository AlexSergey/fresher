# Live Storage Attribution

**Since:** 2026-11-15

## Problem

The user starts Docker, Chrome or Xcode and the disk fills up — but which process wrote what, where, and how fast?

## Direction

- A live view while the user works: which processes create new files, in which folders, how many bytes per minute,
  which apps keep generating temporary data, which write unusually many logs.
- Example of the view:

  ```
  Live disk activity · last 10 minutes · High activity
  Docker           +1.8 GB
  Google Chrome    +420 MB
  Xcode            +280 MB
  ```

- Recording runs **only while the app is open and only after the user starts it** — a visible "Recording" state with a
  stop button ([`../goal.md`](../goal.md), section 8: no background work).

## Feasibility (to check in the plan)

- `FSEvents` reports changed paths, not the process that changed them.
- Per-process attribution needs the Endpoint Security framework (an entitlement granted by Apple and root) or
  `fs_usage` (root). Without them, attribution is inferred: path → owning app by the rules (cache folder of
  `com.google.Chrome` → Chrome), which covers most cases but not all.
- Option: attribute by path rules first (no privileges), and offer the precise per-process mode later if the
  entitlement is obtained.

Open questions:

- Is path-based attribution enough for a first version?
