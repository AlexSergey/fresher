# Cleanup Preview & Safe Removal

**Since:** 2026-11-15

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
- A local history of cleanups (in the SQLite store) so the user can see what was removed and when.

Open questions:

- Undo beyond the Trash (a "put back" list from the history)?
