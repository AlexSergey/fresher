# Spotlight does not see an IDE on a volume it does not index

**Status:** open · **Updated:** 2026-11-17

## Symptom

The `jetbrains` module offers the data of an IDE that is no longer in the apps folders only when Spotlight knows no
app of it anywhere on the Mac. An IDE kept on a volume that Spotlight does not index (an external drive added to
Spotlight's privacy list, a network volume, a disk image mounted without indexing) is not found, so its data is
offered as the data of an IDE that is no longer installed (`jetbrains.uninstalled`, Medium, confirmed by the user).

## Cause

`MacInstalledApps::find` asks Spotlight (`mdfind`) by bundle id. Before it does, it asks for the Finder, which every
Mac has: no answer means the index is off as a whole, and every folder is kept (`jetbrains.left-out.spotlight-unread`).
A single volume left out of the index answers like a volume with no app on it; nothing tells the two apart.

## What was tried

- The Finder probe (plan 016, final review): it catches an index that is off or not built, not one volume left out.

## Options left

- Ask `mdutil -s` for every mounted volume and keep everything when one is not indexed: one more command per volume,
  and a network volume may hang it.
- Walk the mounted volumes' top apps folders as well: slow on a large external drive, and the module must not walk
  outside its places.
- Accept it: the finding names the IDE and its version, the user confirms every removal, and the folders go to the
  Trash.
