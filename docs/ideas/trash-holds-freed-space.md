# The Trash holds the freed space

**Since:** 2026-11-15 · **Proposed by:** the agent

## Problem

Two of our promises together produce a lie. Items go to the Trash by default
([`../restrictions/deletion-safety.md`](../restrictions/deletion-safety.md)), and the app never empties the Trash
([best practices](./best-practices.md), 9). So after "Freed 20 GB" the disk is exactly as full as before: macOS frees
the space only when the Trash is emptied. The user checks the Storage settings, sees nothing changed, and stops
trusting every other number in the app.

## Direction

- **Say "moved", not "freed".** The result screen: "Moved 20 GB to the Trash. You get the space back when you empty
  the Trash." and an **Open Trash** button (Finder; we never empty it).
- **Plan [008](../plans/008-safe-removal/readme.md) reports the freed space "as measured on the volume"**: while the
  items wait in the Trash that number is close to zero. The result screen shows both — moved to the Trash, and freed on
  the volume — so the zero reads as expected, not as a failure.
- **The Trash as information** on the main screen: "Waiting in the Trash: 23 GB, of which 20 GB from cleanups in this
  app" — the second number from the cleanup journal of plan [008](../plans/008-safe-removal/readme.md).
  Never a cleanable category: what the user put there is theirs.
- **Per volume.** An item trashed on an external drive goes to that drive's `.Trashes` and frees space there, not on
  the Mac ([External Drive Analyzer](./external-drive-analyzer.md)); the numbers are shown per volume.
- **Permanent delete stays the honest alternative**, unchanged: a separate, explicit choice in the confirmation,
  whose description says "frees the space now; you cannot put these items back".
- **History is not fooled.** Items the app moved to the Trash are known from the cleanup journal; the next scan
  reports them as "cleaned", not as "removed" or "shrank by itself", and does not attribute the Trash's growth to any
  app ([history and watch mode](../plans/010-history/readme.md)).

Open questions:

- If the user has turned on Finder's "remove items from the Trash after 30 days", can the app read that setting and
  say "the space comes back by itself on 15 December"? **Unverified**: where the setting is stored and whether it can
  be read without a permission prompt.
- Should the confirmation screen already say "the space comes back when you empty the Trash", or only the result
  screen?

**Touched by [plan 012](../plans/012-storage-map/readme.md):** the Trash is a named place on the by-meaning view, never a cleanable one. **Still an idea:** the rest of this file, in particular the cleanup journal's share of the Trash.
