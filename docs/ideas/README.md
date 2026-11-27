# Ideas

The user's ideas and the agent's proposals that are not planned yet: no design, no date, often only a sentence or
two. Unlike [`../deferred/`](../deferred/README.md), which holds agreed work postponed to a known stage, an idea may
still be dropped or reshaped. When one is taken up it gets a design and a plan in [`../plans/`](../plans/README.md), and
its file here links there.

The **From** column tells the user's ideas from the agent's proposals. An agent's proposal becomes the user's idea when
the user accepts it (its **From** turns to `user`), or is removed.

## Format

One file per idea, `<topic>.md`:

```markdown
# <Idea>

**Since:** YYYY-MM-DD · **Proposed by:** the agent (only on the agent's proposals)

## Problem

<What is missing or painful.>

## Direction

<A rough approach, open questions.>
```

| Idea                                                                    | Since      | From  | What                                                                                                                                  |
| ----------------------------------------------------------------------- | ---------- | ----- | ------------------------------------------------------------------------------------------------------------------------------------- |
| [User settings](./user-settings.md)                                     | 2026-11-15 | user  | a settings screen stored in the local SQLite database; never turns off confirmation                                                   |
| [Stale project cleanup](./stale-project-cleanup.md)                     | 2026-11-15 | user  | find projects untouched for a customizable period (default 1 year); offer their `node_modules`, `target/` and other regenerable parts; built by plan 021 (kinds beyond the table stay ideas; `dist` and framework output never) |
| [Ignore list](./ignore-list.md)                                         | 2026-11-15 | user  | an Ignore button on a section or item (e.g. Docker images), remembered in settings, restorable from there; ignored means not watched  |
| [Uninstalled app leftovers](./uninstalled-app-leftovers.md)             | 2026-11-15 | user  | Orphaned Files Finder: data left in `~/Library` by apps the user removed, matched by bundle identifier, offered per app; partly built by plans 016 and 019 (group containers, launch agents, a choice per app stay ideas)               |
| [Duplicate Finder](./duplicate-finder.md)                               | 2026-11-15 | user  | exact duplicates by staged hashing, similar images by perceptual hash; APFS clones and hard links are not duplicates                  |
| [System Junk Cleaner](./system-junk-cleaner.md)                         | 2026-11-15 | user  | caches, temporary files, logs, crash reports, old device updates — each a named category; logs and crash reports built by plan 033 |
| [System Data Analyzer](./system-data-analyzer.md)                       | 2026-11-15 | user  | "System Data" broken down: APFS local snapshots, purgeable space, swap, caches; a reconciliation line, no "Other"                     |
| [External Drive Analyzer](./external-drive-analyzer.md)                 | 2026-11-15 | user  | external SSD/HDD/USB and network volumes; their Trash, AppleDouble files, Time Machine backups; history per volume UUID               |
| [Developer Environment Cleaner](./developer-environment-cleaner.md)     | 2026-11-15 | user  | Xcode, simulators, Docker, package managers, IDEs — through each tool's own cleanup command where it has one                          |
| [Cleanup Preview & Safe Removal](./cleanup-preview-and-safe-removal.md) | 2026-11-15 | user  | the confirmation screen and flow: preview, re-check before delete, Trash first, result screen, cleanup history                        |
| [Startup & Login Items Manager](./startup-and-login-items-manager.md)   | 2026-11-15 | user  | login items and launch agents; **conflicts with a non-goal of `goal.md`** — needs the user's decision                                 |
| [Opportunities](./opportunities.md)                                     | 2026-11-15 | user  | the user's product strategy: where Fresher can win — time, history, attribution                                                       |
| [Live Storage Attribution](./live-storage-attribution.md)               | 2026-11-15 | user  | which process writes what, where and how fast — recorded only while open and started by the user                                      |
| [Storage Leak Detector](./storage-leak-detector.md)                     | 2026-11-15 | user  | folders that keep growing without a clear reason: abnormal storage behaviour, not big files                                           |
| [App Resource Profile](./app-resource-profile.md)                       | 2026-11-15 | user  | **not now** — "App Resource DNA": one card per app — size, data, cache, logs, RAM, CPU, writes, network, background                   |
| [Storage History](./storage-history.md)                                 | 2026-11-15 | user  | the disk over time by app and category; compare any two states                                                                        |
| [Cleanup Simulator](./cleanup-simulator.md)                             | 2026-11-15 | user  | a virtual result while choosing categories; an estimate from private sizes, not a guarantee                                           |
| [Cleanup Guardian](./cleanup-guardian.md)                               | 2026-11-15 | user  | remembers what came back and suggests personal maintenance rules on the next visit                                                    |
| [Process-to-Storage Timeline](./process-storage-timeline.md)            | 2026-11-15 | user  | **later, needs a tiny daemon** — what happened before the disk ran out: processes, file events and history on one timeline            |
| [Intelligent Storage Explainer](./storage-explainer.md)                 | 2026-11-15 | user  | every gigabyte explained, with confidence; consistent, never double-counted, no "not classified"                                      |
| [Safe Cleanup Journal](./cleanup-journal.md)                            | 2026-11-15 | user  | every cleanup a transaction: what, where, which rule, recoverable or not; put back                                                    |
| [Best practices of the field](./best-practices.md)                      | 2026-11-15 | user  | what the best open-source cleaners, analyzers and duplicate finders do well, and what users complain about — ideas only               |; its item 27, the demo recordings, is built by plan 038 (automatic screenshots stay an idea) and its item 24 by plan 039 (the agreement, the danger zone, the High tick, the help, the writing check) 
| [Architecture ideas](./architecture/README.md)                          | 2026-11-15 | user  | scanning, hashing, macOS specifics, crates: what the best open-source projects do and what we take                                    |
| [Local AI models](./local-ai-models.md)                                 | 2026-11-15 | agent | model stores of local AI tools, one finding per model; taken by plans 023 (Hugging Face and LM Studio) and 029 (Ollama); other apps' models stay ideas |
| [Unused toolchain versions](./unused-toolchain-versions.md)             | 2026-11-15 | agent | toolchain versions nothing on the Mac points at; taken by plan 024 (rustup, and nvm behind a switch off by default; the other managers deferred) |
| [Downloads and installers](./downloads-and-installers.md)               | 2026-11-17 | user  | built in plan 020: unfinished and resumable downloads, installers of installed apps, old macOS installers, big files not opened; the rest stays |
| [Caches of installed apps](./installed-app-caches.md)                   | 2026-11-19 | agent | caches of installed apps; taken by plan 030 (`~/Library/Caches/<bundle id>` of installed apps, emptied while the app is closed); browsers and known web-engine apps taken by plan 034 (a table, four rows); more rows, game launchers, the Metal shader cache deferred         |
| [Games not played for a long time](./unplayed-games.md)                 | 2026-11-19 | agent | installed games with size and last played; removed only through the launcher, never pre-selected                                     |
| [A share card after a cleanup](./share-card.md)                         | 2026-11-19 | agent | an image of the space freed, made on the Mac and copied to the clipboard; no paths, no network                                        |
| [Translations of the interface](./ui-translations.md)                   | 2026-11-19 | agent | the interface in the system language, one file per language from contributors; Russian first                                          |
| [More kinds of project artifacts](./more-project-kinds.md) | 2026-11-23 | user | stale projects: a marker inside the folder (`CACHEDIR.TAG`, `pyvenv.cfg`), Python and Node tool caches, .NET, PHP, Elixir, Haskell, CMake, Zig, Terraform, game engines (Medium); no user-written delete rules |
| [AI coding assistants' data](./ai-coding-assistants-data.md)        | 2026-11-26 | user  | old sessions, transcripts and file snapshots of terminal AI assistants; settings and memory never offered |
| [Big projects left alone](./big-stale-projects.md)                      | 2026-11-26 | user  | a project over 1 GB not changed for months, shown on Largest with Show in Finder only, never offered; taken by plan 044 |
| [Other users on the Mac](./other-users-on-the-mac.md) | 2026-11-26 | user | other accounts' homes: run Fresher in each today; later an admin's scan through the helper, the owner on every row |
| [.NET and NuGet packages](./dotnet-nuget-packages.md) | 2026-11-26 | user | NuGet's caches and package versions no project restores, .NET SDKs nothing points at |
| [Many small places that add up](./many-small-places.md) | 2026-11-26 | user | a checked list of small caches of more tools, added as rows of a module's table |
| [A first-run onboarding](./first-run-onboarding.md) | 2026-11-27 | user | one screen before the first scan: Full Disk Access with Open System Settings or Skip for now; scans and probes nothing on its own; candidate for 0.2.0 |
