# The cleaner checks protection on names as the plan writes them

**Status:** open · **Updated:** 2026-11-25

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

- Plan 036/01/02 folds case (only `to_lowercase`) for a bundle below an app folder against the places inside that
  folder; that narrows the case variant there, not the general gap.
- Put back (security audit 2026-11-25, plan 041): its original path comes from the journal file, which any program of
  the user can write, not from a finding. Put back now also checks the protection on the disk's own spelling of the
  destination's folder (`F_GETPATH` of the folder it holds open, joined with the item's name), so a case or
  normal-form variant of a protected folder is refused (`put_back_into_a_protected_folder_spelt_in_another_case_is_refused`,
  `crates/cleaner/src/put_back.rs`). The item's own last name is still compared as written: a variant of a protected
  place that does not exist yet could be created by put back. An existing one is never replaced (`RENAME_EXCL`).

## How it is kept safe today

- Every plan item comes from a finding the scanner made, or from the system's listing of installed apps for an
  uninstall: their names are the names the disk returned, never what the interface types.
- Put back checks the disk's spelling of the destination's folder (above), and the engine puts back only an entry
  that landed in a Trash from the place its history row shows (`Located::of`).
- The interface picks findings by id (`CleanupSelection`: finding ids and categories), never by path.
- The uninstall plan refuses a bundle whose canonical path is not byte for byte the listed one (plan 036/02/01).

## Options left

- In the guard, before checks 3 and 4, read the name each component has on the disk (the canonical path of the
  opened item, from its held folder) and check the protection on that form too, as put back now does for its folder.
- For put back's last name: compare it with the protected places of the same folder as the volume compares names
  (case and normal form folded).
