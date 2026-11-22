# 032 — Docker

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, engine, cleaner, testing, sandbox, modules, e2e, app, app/ui, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

[`goal.md`](../../goal.md), section 4, and the README promise Docker; nothing is built. It is the largest single
consumer on many developers' Macs and the most dangerous one: a volume may hold the only copy of a database.

## Goals

1. Images, containers, volumes and build cache are separate categories, each with its level and consequence.
2. Everything goes through the Docker command line; Docker's files are never touched
   ([`restrictions/deletion-safety.md`](../../restrictions/deletion-safety.md)).
3. The user understands why the Mac may get nothing back at once.

## Non-goals

- Starting Docker, changing its settings, shrinking or deleting its disk image.
- Other runtimes' own commands; a runtime that answers the `docker` command is covered, the rest is not.

## Design

A new module `docker` (`fresher-module-docker`, group **Development**). It reads `docker system df -v` and
`docker ... ls` as JSON through `CommandRunner` with fixed arguments and a timeout; one finding per object.

| Category                    | Level  | Offered                                                    |
| --------------------------- | ------ | ---------------------------------------------------------- |
| `docker.build-cache`        | Low    | the build cache, as one finding                            |
| `docker.dangling-images`    | Low    | untagged images no container uses                          |
| `docker.unused-images`      | Medium | tagged images no container uses                            |
| `docker.stopped-containers` | High   | containers that are not running; never pre-selected        |
| `docker.volumes`            | High   | volumes no container uses; never pre-selected              |

- Removal is one command per object by id, never `--force`, never a blanket `prune`. An object that came into use
  since the scan is refused by Docker itself and reported as skipped.
- **The disk image card.** Docker runs in a virtual machine whose whole disk is one file. Finder shows that file at
  its maximum, not at what it takes, and space freed inside returns to macOS only when the runtime shrinks the file.
  The card gives three numbers in plain English: **maximum size** ("Docker may grow up to 64 GB"), **on your disk**
  (the allocated size, what the Mac actually loses), **used inside Docker** (from the command line). Information
  only: deleting the file would wipe every image, container and volume. The categories are its breakdown and are
  never added to the map's totals a second time. With Docker stopped the card shows the two file numbers.
- **Facts to check in the design**: how and when each runtime shrinks its disk (later, on restart, by a setting), and
  where the other runtimes that answer `docker` keep theirs. The card points the user at the runtime's own way to
  shrink; the app does not run it.
- The result says "freed inside Docker; macOS gets it back when Docker shrinks its disk".
- Docker not installed: the module is silent. Installed but not running: one line, "start Docker to see what is
  inside"; the app never starts it.

**Sandbox.** `fixtures/sandboxes/docker.toml`, `docker-stopped.toml` and `docker-absent.toml`, negatives first: no
`docker` program, the daemon not answering, an image a container uses, a volume a container mounts, a running
container, an answer the module cannot parse (claims nothing); then one positive per category. The answers come from
the sandbox world's `[[world.tool]]` (see plan [031](../031-xcode/readme.md)); the fake runner also records the
removal commands, and the golden report holds them. The finish walks `EXPECTED.md` in sandbox mode, then a run on the
developer's Mac with a throwaway image and volume.

## Plans (after approval)

| #   | Plan        | What                                                                       |
| --- | ----------- | -------------------------------------------------------------------------- |
| 01  | Tool world  | only if plan 031 has not built it                                          |
| 02  | Module      | reading the answers, the decisions, the texts, the category files          |
| 03  | Removal     | the per-object commands, skips, the result's wording                       |
| 04  | Disk image  | the card and the single count on the storage map                           |
| 05  | End to end  | case files, golden reports, the core e2e flow, the UI                      |
| 06  | Finish      | docs, the run on the developer's Mac                                       |

Depends on plans 008, 009, 011, 012 and 015.

## Ideas this touches

[Developer Environment Cleaner](../../ideas/developer-environment-cleaner.md) (the Docker line). This plan replaces
the idea "Why Docker does not give the space back" (2026-11-15, the agent's proposal); everything it held is above.

## Open questions

1. **A Docker removal cannot go to the Trash.** Rule 2 makes a permanent delete "a separate explicit choice". The
   recommendation: Docker items always show "removed for good, cannot be put back" in the confirmation and need the
   same explicit tick as a permanent delete; a confirmation mixing them with files asks for it once, for the Docker
   items only.

## Decisions

None yet.

## Agent's calls (to check)

1. **One command per object, by id**; no `prune`, no force.
2. **Containers and volumes are never pre-selected.**
3. **Only what answers the `docker` command.**
4. **The disk image is information only**, and shrinking it is pointed at, never run.

## Conclusions

Not written yet.
