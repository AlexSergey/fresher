# Finder's own Put Back is unreliable for what Fresher moved to the Trash

**Status:** open · **Updated:** 2026-11-26

## Symptom

Found in the sandbox run by hand before 0.1.0 (`npm run dev:sandbox -- trial --real-trash`, the system's Trash):

- **The wrong place.** Two folders with the same name, `work/Sample app cache` and `finder-put-back/Sample app cache`,
  went to the Trash in one cleanup (macOS renamed the second one in the Trash). Fresher put both back to their own
  places. Later `finder-put-back/Sample app cache` went to the Trash again, alone. Finder's Put Back on it first refused
  ("an item with that name already exists"); with `work/` cleared, it restored the folder into `work/`, not into
  `finder-put-back/`. Nothing was overwritten.
- **No Put Back.** A file moved out of an emptied folder (`keep/Old sample downloads/entry.bin`, `Removal::EmptyFolder`)
  had no Put Back in Finder at all, while a whole folder moved in the same cleanup had one.

Fresher's own put back (the Cleanups screen) restored every item to its own place.

## Cause

Not investigated. Finder keeps its own record of where a Trash item came from; the cleaner moves items with the
system's Trash call (`trashItemAtURL:resultingItemURL:error:`, `fresher-macos`'s `MacTrash`) and journals the original
path itself. Finder's record seems to follow the item's name in the Trash and to be missing or stale for some items
the call moved; the exact rule is the system's.

## What was tried

Nothing beyond the run above.

## Options left

- Tell users to put back from the Cleanups screen (the release notes and the changelog promise only that).
- Find out what Finder's record needs and whether the Trash call can leave it right.
