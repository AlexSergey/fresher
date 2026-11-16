# Safe Cleanup Journal

**Since:** 2026-11-15

**Taken by plan [008](../plans/008-safe-removal/readme.md)** (2026-11-16): an append-only journal written ahead of
every cleanup, recording where each item went, and put back from the history for items still in the Trash.

## Problem

After a cleanup the user wants to know exactly what happened, and to undo it when possible.

## Direction

- Every cleanup is a transaction in the local store: what was removed, from where, when, how much space was freed,
  **which rule decided it**, and whether it can be restored.

  ```
  Cleanup history
  Chrome cache cleanup     Today · 09:42       +1.42 GB   Recoverable
  Xcode temporary files    Yesterday · 18:20   +3.80 GB   Completed
  ```

- Items go to the Trash (or a quarantine folder of the app) rather than being deleted; "put back" restores them to their
  original place from the journal ([`architecture/macos.md`](./architecture/macos.md), "Moving to the Trash").
- **Honest about irreversibility**: the journal separates recoverable actions (in the Trash, restorable) from
  irreversible ones (a tool's own cleanup command, a permanent delete, an emptied Trash) and says which is which before
  the action, not after.
- Complements [`cleanup-preview-and-safe-removal.md`](./cleanup-preview-and-safe-removal.md).

Open questions:

- A quarantine folder in addition to the Trash: who empties it, and when — only the user.
