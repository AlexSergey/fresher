# 032 — Docker

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, engine, testing, sandbox, modules, e2e, app, app/ui, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind." And on this plan (2026-11-22): "Perhaps just describe: your
Docker takes this and this — here is the command to clean it if you need."

[`goal.md`](../../goal.md), section 4, and the README promise Docker; nothing is built. It is the largest single
consumer on many developers' Macs and the most dangerous one: a volume may hold the only copy of a database.

## Goals

1. The user sees what Docker takes, split into images, containers, volumes and build cache, in plain English.
2. For each part the app gives the exact command that frees it and says what running it costs.
3. The user understands why the Mac may get nothing back at once.

## Non-goals

- **Removing anything of Docker's** (decision 1): the app runs no command that changes Docker and offers no Docker
  item in a confirmation.
- Starting Docker, changing its settings, shrinking or deleting its disk image.
- Other runtimes' own commands; a runtime that answers the `docker` command is covered, the rest is not.

## Design

A new module `docker` (`fresher-module-docker`, group **Development**). It only reads: `docker system df -v` as JSON
through `CommandRunner`, with fixed arguments and a timeout. Its page is an explanation, not a list of findings.

| Part               | What the app says                                         | The command it shows        | The warning beside it                         |
| ------------------ | --------------------------------------------------------- | --------------------------- | --------------------------------------------- |
| Build cache        | size; rebuilt by the next build                           | `docker builder prune`      | the next builds are slower                    |
| Unused images      | size and count, untagged apart from tagged                | `docker image prune` (`-a`) | pulled or built again when needed             |
| Stopped containers | size and count                                            | `docker container prune`    | anything written inside them is lost          |
| Unused volumes     | size and count                                            | `docker volume prune`       | may hold the only copy of a database          |

- Each command has a **Copy** button and one sentence on what it removes; the app never runs it. The riskier a part,
  the more plainly its warning reads; the volumes' line carries the High level's wording.
- **The disk image card.** Docker runs in a virtual machine whose whole disk is one file. Finder shows that file at
  its maximum, not at what it takes, and space freed inside returns to macOS only when the runtime shrinks the file.
  The card gives three numbers: **maximum size** ("Docker may grow up to 64 GB"), **on your disk** (the allocated
  size, what the Mac actually loses), **used inside Docker** (from the command line). With Docker stopped it shows
  the two file numbers and "start Docker to see what is inside"; the app never starts it.
- **Counted once**: the parts are the breakdown of the disk image's line on the storage map, never extra space, and
  never part of the "can be cleaned" total — the app cleans none of it.
- Docker not installed: the module is silent.
- **Facts to check in the design**: how and when each runtime shrinks its disk (later, on restart, by a setting), and
  where the other runtimes that answer `docker` keep theirs. The card points at the runtime's own way to shrink.
- How an information-only module sits on the scan screen (a card with no selection, beside the storage map's lines) is
  the design's first question: today every module's card selects items.

**Sandbox.** `fixtures/sandboxes/docker.toml`, `docker-stopped.toml` and `docker-absent.toml`, negatives first: no
`docker` program, the daemon not answering, an answer the module cannot parse (says nothing rather than guess), a
world where a removal command would be a test failure (the fake runner refuses every argument list but the read);
then the four parts and the card. The answers come from the sandbox world's `[[world.tool]]` (see plan
[031](../031-xcode/readme.md)). Golden report, UI e2e of the page and the copy buttons, and the finish walks
`EXPECTED.md` in sandbox mode, then a look on the developer's Mac.

## Plans (after approval)

| #   | Plan       | What                                                              |
| --- | ---------- | ----------------------------------------------------------------- |
| 01  | Tool world | only if plan 031 has not built it                                 |
| 02  | Module     | reading the answers, the parts, the texts and commands            |
| 03  | Disk image | the card and the single count on the storage map                  |
| 04  | Screen     | the information-only card and page, Copy                          |
| 05  | End to end | case files, golden reports, e2e                                   |
| 06  | Finish     | docs, the README's Docker line, the look on the developer's Mac   |

Depends on plans 009, 011, 012, 015 and 027.

## Ideas this touches

[Developer Environment Cleaner](../../ideas/developer-environment-cleaner.md) (the Docker line). This plan replaces
the idea "Why Docker does not give the space back" (2026-11-15, the agent's proposal); everything it held is above.

## Open questions

None.

## Decisions

1. **Docker is described, not cleaned** (the user, 2026-11-22): the app shows what Docker takes and the command that
   frees each part; the user runs it if they want. This also settles how a removal that cannot go to the Trash would
   have met rule 2: there is none.

## Agent's calls (to check)

1. **The whole-kind commands** (`prune`) are shown, each with its warning, rather than one command per object.
2. **Only what answers the `docker` command.**
3. **Docker's space is never in the "can be cleaned" total.**
4. **No category files**: the module has no cleanable category; its texts live in the core's dictionary.

## Conclusions

Not written yet.
