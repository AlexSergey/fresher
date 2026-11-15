# Ideas

The user's ideas that are not planned yet: no design, no date, often only a sentence or two. Unlike
[`../deferred/`](../deferred/README.md), which holds agreed work postponed to a known stage, an idea may still be
dropped or reshaped. When one is taken up it gets a design and a plan in [`../plans/`](../plans/README.md), and its file
here links there.

## Format

One file per idea, `<topic>.md`:

```markdown
# <Idea>

**Since:** YYYY-MM-DD

## Problem

<What is missing or painful.>

## Direction

<A rough approach, open questions.>
```

| Idea                                                                    | Since      | What                                                                                                                                  |
| ----------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| [User settings](./user-settings.md)                                     | 2026-11-15 | a settings screen stored in the local SQLite database; never turns off confirmation                                                   |
| [Stale project cleanup](./stale-project-cleanup.md)                     | 2026-11-15 | find projects untouched for a customizable period (default 1 year); offer their `node_modules`, `target/` and other regenerable parts |
| [Ignore list](./ignore-list.md)                                         | 2026-11-15 | an Ignore button on a section or item (e.g. Docker images), remembered in settings, restorable from there; ignored means not watched  |
| [Uninstalled app leftovers](./uninstalled-app-leftovers.md)             | 2026-11-15 | Orphaned Files Finder: data left in `~/Library` by apps the user removed, matched by bundle identifier, offered per app               |
| [Large Files Finder](./large-files-finder.md)                           | 2026-11-15 | the largest files, folders and archives, sorted by size; a treemap and a list over one result                                         |
| [Duplicate Finder](./duplicate-finder.md)                               | 2026-11-15 | exact duplicates by staged hashing, similar images by perceptual hash; APFS clones and hard links are not duplicates                  |
| [System Junk Cleaner](./system-junk-cleaner.md)                         | 2026-11-15 | caches, temporary files, logs, crash reports, old device updates — each a named category                                              |
| [System Data Analyzer](./system-data-analyzer.md)                       | 2026-11-15 | "System Data" broken down: APFS local snapshots, purgeable space, swap, caches; a reconciliation line, no "Other"                     |
| [External Drive Analyzer](./external-drive-analyzer.md)                 | 2026-11-15 | external SSD/HDD/USB and network volumes; their Trash, AppleDouble files, Time Machine backups; history per volume UUID               |
| [Smart Uninstaller](./smart-uninstaller.md)                             | 2026-11-15 | remove an app with everything it owns, or keep its settings; Homebrew and App Store apps the way their manager expects                |
| [Application Footprint](./application-footprint.md)                     | 2026-11-15 | the real size of each app including its data outside `/Applications`, with the last time it was used                                  |
| [Developer Environment Cleaner](./developer-environment-cleaner.md)     | 2026-11-15 | Xcode, simulators, Docker, package managers, IDEs — through each tool's own cleanup command where it has one                          |
| [Cleanup Preview & Safe Removal](./cleanup-preview-and-safe-removal.md) | 2026-11-15 | the confirmation screen and flow: preview, re-check before delete, Trash first, result screen, cleanup history                        |
| [Startup & Login Items Manager](./startup-and-login-items-manager.md)   | 2026-11-15 | login items and launch agents; **conflicts with a non-goal of `goal.md`** — needs the user's decision                                 |
| [Best practices of the field](./best-practices.md)                      | 2026-11-15 | what the best open-source cleaners, analyzers and duplicate finders do well, and what users complain about — ideas only               |
| [Architecture ideas](./architecture/README.md)                          | 2026-11-15 | scanning, hashing, macOS specifics, crates: what the best open-source projects do and what we take                                    |
