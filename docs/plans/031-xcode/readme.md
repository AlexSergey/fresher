# 031 — Xcode and simulators

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, testing, sandbox, modules, e2e, app, app/ui, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

Every cleaner aimed at developers handles the whole of Xcode's data; Fresher offers only DerivedData (plan
[018](../018-developer-caches/readme.md)). Archives, device support files and simulators are usually the largest part,
and [`goal.md`](../../goal.md), section 4, names them.

## Goals

1. Each kind of Xcode data is its own category with its own danger level; nothing is one "Xcode junk" bucket.
2. What only a tool can remove safely is removed through that tool, never by deleting its files.
3. Nothing is offered while Xcode or the Simulator is open.

## Non-goals

- Xcode itself, its SDKs, command line tools, and installed simulator runtimes (system-managed disk images).
- Moving DerivedData out of `dev-caches`: its category id stays.

## Design

A new module `xcode` (`fresher-module-xcode`, group **Development**), one finding per folder or device:

| Category                    | Level  | What                                                                           | Removal                       |
| --------------------------- | ------ | ------------------------------------------------------------------------------ | ----------------------------- |
| `xcode.device-support`      | Low    | debug symbols per OS version of a connected device; copied again on connect    | Trash, one finding a version  |
| `xcode.previews`            | Safe   | the previews' build cache                                                      | Trash                         |
| `xcode.simulator-caches`    | Safe   | the simulators' shared caches                                                  | folder emptied                |
| `xcode.unavailable-devices` | Medium | simulator devices whose runtime is no longer installed                         | `xcrun simctl delete <id>`    |
| `xcode.archives`            | High   | archived builds with the symbols of shipped versions; never pre-selected       | Trash, one finding an archive |

- A device support version in use by the newest two OS versions seen is still offered, but named so; the level never
  drops below the table's.
- Unavailable devices are read from `xcrun simctl list --json` through `CommandRunner` and removed by id with fixed
  arguments, never forced (the cleaner's tool removal, [`architecture/removal.md`](../../architecture/removal.md)). A
  tool removal cannot go to the Trash: the confirmation says so in its own sentence.
- Left out, with a reason, while Xcode or the Simulator runs (`app-open`, as `dev-caches-xcode-open.toml` does today).
- The paths are facts to check on the developer's Mac before the design is final; the layout is undocumented and the
  module refuses a folder it does not recognise.

**Sandbox.** `fixtures/sandboxes/xcode.toml` and `xcode-open.toml`, negatives first: a look-alike folder, a link at a
place, an available device, a device support folder with an unreadable name, Xcode open; then one positive per
category. The device cases need a tool's answers in the sandbox world (`[[world.tool]]`: a program, its exact
arguments, its output — item of [`deferred/sandbox-world.md`](../../deferred/sandbox-world.md)); this plan or plan
[032](../032-docker/readme.md), whichever is built first, builds it. Golden report, core e2e flow, and the finish
walks `EXPECTED.md` in `npm run dev:sandbox -- xcode`.

## Plans (after approval)

| #   | Plan           | What                                                                       |
| --- | -------------- | -------------------------------------------------------------------------- |
| 01  | Tool world     | `[[world.tool]]` in case files, `world.toml`, sandbox mode and the autotest |
| 02  | Module         | the tables, the decisions, the texts, the category files                   |
| 03  | End to end     | case files, golden reports, the core e2e flow, the UI's fixtures           |
| 04  | Finish         | docs, the walk on the developer's Mac                                      |

Depends on plans 008, 009, 011, 015 and 018.

## Ideas this touches

[Developer Environment Cleaner](../../ideas/developer-environment-cleaner.md) (the Xcode line).

## Open questions

None.

## Decisions

None yet.

## Agent's calls (to check)

1. **A module of its own**, `xcode`; DerivedData stays where it is.
2. **Only unavailable simulator devices**, through `simctl`; available devices and runtimes are not offered.
3. **Archives are High and never pre-selected.**
4. **The tool world is built here** if this plan goes before Docker.

## Conclusions

Not written yet.
