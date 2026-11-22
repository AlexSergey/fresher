# The cleaner checks protection on names as the plan writes them

**Status:** open · **Updated:** 2026-11-22

## Symptom

For every item, the cleaner checks the protection on two forms of its path: as the plan names it, and the root's
canonical path joined with the item's names below the root (`crates/cleaner/src/paths.rs`, `Placed::physical`,
`check_protection`). Only the root is made canonical; the item's own names stay as written. On a volume that ignores
case, a variant of a `never`, `whole` or ignored place's path that differs by case (`~/library/keychains`), by Unicode
folding (`ſ` for `s`) or by normal form (decomposed against composed) names the same folder on the disk, yet is
compared as a different path, so the protection does not refuse it. This predates plan 036 and holds for every
protected place, not only the apps folder.

## Cause

The protection compares paths by their components, lexically, and expects canonical paths (its own documentation).
The cleaner makes the root canonical once, then opens the item beneath it through no link, name by name; it does not
read back the names the disk holds for each component before the check.

## What was tried

Nothing in the cleaner yet: it is frozen in plan 036. Plan 036/01/02 folds case (only `to_lowercase`) for a bundle
below an app folder against the places inside that folder; that narrows the case variant there, not the general gap.

## How it is kept safe today

- Every plan item comes from a finding the scanner made, or from the system's listing of installed apps for an
  uninstall: their names are the names the disk returned, never what the interface types.
- The interface picks findings by id (`CleanupSelection`: finding ids and categories), never by path.
- The uninstall plan refuses a bundle whose canonical path is not byte for byte the listed one (plan 036/02/01).

## Options left

- In the cleaner, before checks 3 and 4, read the name each component has on the disk (the canonical path of the
  opened item, from its held folder) and check the protection on that form too. To be weighed in the security audit
  (plan 041), where the cleaner may change.
