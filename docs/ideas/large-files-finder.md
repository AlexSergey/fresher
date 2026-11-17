# Large Files Finder

**Since:** 2026-11-15

## Problem

Often a handful of files and folders hold most of the space: a forgotten 40 GB video export, a disk image, an old
archive. They are hard to find by browsing.

## Direction

- A ranked list of the largest files, folders and archives (`.zip`, `.dmg`, `.iso`, `.tar.*`, `.xip`, `.pkg`) across
  the scanned roots, sorted by size, filterable by kind, age (last opened / modified) and location.
- Folder view drills down from the top (a treemap or sunburst on the same data as the list — the chart and the list
  are two views of one result).
- Each item names its kind in plain English ("Disk image", "Archive", "Video") and the app that made it when it can.
- User files are never "junk": items here carry no cleanup suggestion and default to danger level **High**; the user
  decides. Trash first.
- Archives next to their extracted folder ("`Foo.zip` and `Foo/`") and installers already used (`.dmg`, `.pkg` of an
  app that is installed) are pointed out as likely leftovers, still with the user deciding.

Open questions:

- Minimum size to list (default 100 MB? configurable in [`user-settings.md`](./user-settings.md)).
- Use "last opened" (Spotlight's `kMDItemLastUsedDate`) where available?

**Touched by [plan 012](../plans/012-storage-map/readme.md):** the by-place view lists a folder's children largest first, with how much of each the modules explain. **Still an idea:** the ranked list of files and archives, the filters, and the chart (a treemap or sunburst).
