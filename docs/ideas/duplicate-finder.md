# Duplicate Finder

**Since:** 2026-11-15

## Problem

The same photo, video, download or project copy lives in several places. Similar images (resized, re-encoded, burst
shots) waste even more space and are invisible to an exact-match search.

## Direction

- **Exact duplicates by content**, in stages so that most files are never read in full: group by size → hash of the
  first block → hash of the last block → full hash. Hashes are cached between runs (by path, size, modification time).
- **Similar images** by perceptual hash, grouped by distance, shown side by side with resolution and size.
- **APFS clones are not duplicates**: a cloned file shares its blocks; removing it frees nothing. Clones are detected
  and shown as "copy that takes no extra space".
- **Hard links are counted once.**
- The user picks which copy to keep (newest, oldest, in a preferred folder — rules in
  [`user-settings.md`](./user-settings.md)); the app never picks silently. Danger level **High** (user files).
- Bundles (`.app`, `.photoslibrary`, `.fcpbundle`) are treated as one unit, never deduplicated inside.

Open questions:

- Offer "replace duplicates with APFS clones" (frees space, keeps every path) as a safer alternative to deletion?
- Which folders to skip by default (Photos library internals, iCloud placeholders not downloaded).
