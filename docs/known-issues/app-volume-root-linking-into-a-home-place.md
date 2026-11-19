# A scan root on a volume that links into a guarded home folder

**Status:** open · **Updated:** 2026-11-19

## Symptom

A scan root that lies on a removable or network volume and is, or passes through, a link into a guarded folder of the
home (Desktop, Documents, Downloads) is treated only as a volume root. The walk opens the link's target, so macOS may
ask about that home folder during the scan, although the folder was not opted into.

## Cause

`MacAccess::guarded_places` (`crates/system/src/access.rs`) never reads past a volume's mount point while resolving a
root, so that no file system call reaches inside a removable or network volume outside a scan the user started. A root
on a volume therefore gets only its volume's guard; the link inside it is not followed to find the home folder it
leads to.

## What was tried

Resolving every root fully, links included (final review of plan 006): it stats inside volumes, which may itself make
macOS ask. The volume stop rule was chosen instead.

## Options left

- Read the link's target with `readlink` only (no stat inside the volume) and match it against the home's guarded
  places.
- Leave it: a link on a USB stick or a share that points into the user's Desktop, chosen as a scan root, is rare.
