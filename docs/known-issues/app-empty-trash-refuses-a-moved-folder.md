# Empty Trash refused a node_modules folder Fresher moved, "in use"

**Status:** open · **Updated:** 2026-11-26

## Symptom

Found on the 0.1.0 release build on the developer's Mac: a cleanup of old projects' packages moved several
`node_modules` folders to the Trash. Finder's Empty Trash then refused one of them, "The operation can't be completed
because the item "node_modules 10-29-27-966" is in use", again and again. Finder's **Delete Immediately** on that one
item removed it at once, without relaunching Finder.

## Cause

Not found. What was ruled out: no process of the user or of root had any file of it open (`lsof`, `sudo lsof`); Fresher
held only its own data folder; the folder had no lock flags and was writable. The folder's time had changed two
minutes after the move, so Empty Trash had removed part of it before it stopped. Nothing points at Fresher: the item was
an ordinary folder in the Trash, and the Trash is emptied only by the user.

## What was tried

The checks above; Delete Immediately worked.

## Options left

- Find out what Empty Trash calls "in use" for a large folder (Finder's own state while it deletes, or a system scan of
  newly trashed items).
- If it comes back, tell users in the release notes' Trash section to use Delete Immediately on the item.
