# The debug panel and the developer command line

**Agreed:** 2026-11-17, by the user (plan 007, section 9 of its design) · **Due at:** when the deferred
[world](./sandbox-world.md) is built

## What is postponed

A debug panel in the development build (knobs for permissions, the clock and running apps) and a developer command
line beyond the sandbox generator ("scan a folder", "explain a path", "print a dry run"). Plan 007 built the
generator, `npm run sandbox`, and `npm run sandbox:test`.

## Why

- The panel's knobs are the fakes of the world outside the disk (apps, processes, the clock, access); sandbox mode
  keeps those real and read-only until the [world](./sandbox-world.md) is built, so a panel has nothing to turn yet.
- The sandbox already gives a real window to look at, on files the developer chose.
- A case file and its autotest answer the questions the command line would: what is offered, in which category, with
  which danger, and what a cleanup does.

## What doing it needs

1. The world of [`sandbox-world.md`](./sandbox-world.md), served to sandbox mode through `World::ports()`.
2. A panel compiled in debug builds only (`cfg(debug_assertions)`, no cargo feature), whose every knob sets one of
   those fakes and which never acts on a file.
3. The command line, if still wanted, as a second binary of `fresher-sandbox` over the same case files, never a
   second way to delete.
