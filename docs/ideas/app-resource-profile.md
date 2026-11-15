# App Resource Profile ("App Resource DNA")

**Since:** 2026-11-15 · Possibly part of the base architecture: one model of "an app and everything it owns".

## Problem

To understand one app the user needs eight tools: Finder for its size, Activity Monitor for RAM and CPU, a disk tool for
its caches, and so on.

## Direction

One card per app with its whole footprint:

| Metric      | What it shows                                   |
| ----------- | ----------------------------------------------- |
| App size    | the bundle                                      |
| User data   | support files, containers, documents it owns    |
| Cache       | its caches                                      |
| Logs        | its logs                                        |
| RAM         | memory in use now                               |
| CPU         | load now                                        |
| Disk writes | bytes written (since launch or while recording) |
| Network     | traffic (read locally from system counters)     |
| Background  | its background processes, agents, helpers       |

- Open Chrome and see its whole infrastructure in one place.
- Extends [`application-footprint.md`](./application-footprint.md) (the storage part) with live resource use.
- Architecture: an `App` entity (bundle id, team id, bundle path, owned paths, processes) that every category and
  finding points to — the same model serves the uninstaller, the footprint, attribution and watch mode.

## Scope and rules

- Reading RAM, CPU, disk writes and network counters is local and read-only — no data leaves the Mac
  ([`../restrictions/privacy.md`](../restrictions/privacy.md)).
- [`../goal.md`](../goal.md) lists RAM cleaning among the non-goals; **showing** use is not cleaning, but RAM, CPU and
  network extend the product beyond storage — the user decides.

Open questions:

- Storage-only card first, live resources later?
