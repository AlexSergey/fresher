# Cleanup Preview & Safe Removal

**Since:** 2026-11-15

**Taken by plan [008](../plans/008-safe-removal/readme.md)** (2026-11-16): the frozen plan checked item by item, the
confirmation of that exact list, the checks again right before each item, the Trash first, the result with the
measured space, and the Cleanups screen with put back. No quarantine folder of our own.

## Problem

The moment of deletion is where trust is won or lost. The user must see exactly what will happen and be able to undo
it.

## Direction

This is the product's core promise ([`../goal.md`](../goal.md), section 3; [`../restrictions/deletion-safety.md`](../restrictions/deletion-safety.md));
the idea is how the screen and the flow look:

- A preview of the selection: items grouped by category, total size, the highest danger level in large type, and what
  the user will lose in one sentence per category.
- Items can be unticked one by one; nothing is pre-selected above **Safe**.
- **Re-check before delete**: each item is verified right before removal (still there, same kind, not replaced by a
  symlink); a changed item is skipped and reported.
- Trash first; a separate, explicit "delete permanently" choice; protected system paths cannot be selected at all.
- A result screen: what was freed, what was skipped and why, and "Open Trash" to undo.
- A local history of cleanups (in the SQLite store) so the user can see what was removed and when. *Decided later:
  plain files, not SQLite — [plan 002, decision 1](../plans/002-base-architecture/00-design.md).*

Open questions:

- Undo beyond the Trash (a "put back" list from the history)?
