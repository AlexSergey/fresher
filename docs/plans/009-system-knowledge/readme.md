# 009 — System knowledge: apps, projects, processes

**Status:** 📝 draft · **Version:** 0.5.0 · **Date:** 2026-11-15 · **Modules:** core, macos, system, engine, testing

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

Several modules need the same facts about the Mac: which apps are installed and in which version, where the projects
are, what is running. Modules may not depend on each other, so these facts are shared services behind ports of the core
— gathered once, used by all.

## Goals

1. One source of truth for each fact; no module gathers it on its own.
2. Gathered once per scan, in memory, and only when a module asks.
3. Every service has a fake, so a module is tested with made-up apps and processes.

## Non-goals

- Per-process disk activity ([live storage attribution](../../ideas/live-storage-attribution.md)).
- Storing any of it: these facts are cheap to gather again and are not kept on disk.

## Design

### The services

| Port               | Answers                                                                    | Who needs it                                   |
| ------------------ | -------------------------------------------------------------------------- | ---------------------------------------------- |
| `InstalledApps`    | apps with bundle id, team id, version, path, helpers                       | stale app versions, leftovers, uninstaller, app footprint |
| `Projects`         | projects by marker file, their kind, artifact folders, last activity       | `node_modules`, build folders, stale projects  |
| `RunningProcesses` | which apps and services are running, and whose a folder is                 | "in use" checks of modules and of the cleaner  |
| `Volumes`          | volumes, their kind, free and purgeable space                              | overview, external drives, the scan's total check |
| `KnownFolders`     | where the system keeps app caches, settings, logs, support data, the Trash | every module that names a place                |
| `Clock`            | the current time                                                           | everything with an age                         |
| `CommandRunner`    | runs an external tool with a fixed argument list                           | Docker, package managers                       |

### Rules of the services

- Implemented in a new crate `fresher-system` (depends on core and macos); modules see only the traits.
- **Apps are identified by bundle id and team id**, read from each app's own `Info.plist` and signature — never by a
  loose name (the top cause of wrong deletions in the field).
- **Projects are found during the one shared walk**: the service registers its marker files as interests like a module
  does, so no second pass over the disk. A project's last activity ignores its artifact folders.
- Reads only what identifies things (`Info.plist`, container metadata); never file contents beyond that.
- Lazy and cached for the session; a rescan refreshes them.
- An app that is running while its data is offered raises the finding's caution ("quit the app first"), and the cleaner
  skips items in use.

### Testing

Each port has a fake in `fresher-testing`, filled from the scenario files of plan
[007](../007-safe-development/readme.md): "WebStorm 2026.3 installed, 2026.1 not" is three lines of data.

## Plans (after approval)

| #   | Plan              | What                                                             |
| --- | ----------------- | ---------------------------------------------------------------- |
| 01  | Ports and fakes   | the traits in the core, the fakes, scenario support              |
| 02  | Installed apps    | discovery, bundle and team ids, versions, helpers                |
| 03  | Projects          | marker files in the shared walk, kinds, last activity            |
| 04  | Processes and volumes | running apps and services, ownership of folders, volume facts |

Depends on plans 002 and 004. `Clock` and `CommandRunner` are needed earlier and are created in plan 002.

## Ideas this touches

[macOS specifics](../../ideas/architecture/macos.md) ("finding what belongs to an app"),
[rules as data](../../ideas/architecture/rules-as-data.md) (project artifacts),
[stale project cleanup](../../ideas/stale-project-cleanup.md), [application footprint](../../ideas/application-footprint.md),
[uninstalled app leftovers](../../ideas/uninstalled-app-leftovers.md), the
[smart uninstaller](../../ideas/smart-uninstaller.md).

## Decisions

1. **macOS first, Windows not ruled out** (the user, 2026-11-15): modules name known folders through `KnownFolders`,
   never macOS paths; the macOS adapter answers `~/Library/Caches` and the like, a later Windows adapter would answer
   its own folders.

## Agent's calls (to check)

1. **A new crate `fresher-system`** for the implementations.
2. **Nothing is persisted**; gathered per scan.
3. **Identity by bundle id and team id only.**
4. **Project detection rides on the shared walk.**
5. **Target version 0.5.0.**
6. **`KnownFolders` replaces `Pattern::UnderHome`**: an interest names a known folder and a relative path under it
   (`AppCaches` + `JetBrains/*`); the walk resolves it through the port. Built before the first real module, so no
   module is written with a macOS path.

## Conclusions

Not written yet.
