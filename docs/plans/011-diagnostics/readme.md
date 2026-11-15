# 011 — Errors, logs and diagnostics

**Status:** 📝 draft · **Version:** 0.5.0 · **Date:** 2026-11-15 · **Modules:** core, engine, app, app/ui

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

One way for a problem to travel from the core to the user, one way to log, and a way to see why the app did what it did
— without writing the user's paths to disk and without sending anything anywhere.

## Goals

1. The user never sees a raw technical error: every problem has plain-English text and, where there is one, a next step.
2. The app explains itself: why an item matched, why one was skipped, what a scan could not read.
3. Logs do not leak and do not litter: nothing on disk unless the user asks.
4. A bug report is a file the user makes, reads and sends themselves.

## Non-goals

- Crash upload, telemetry, remote logging — forbidden by [`restrictions/privacy.md`](../../restrictions/privacy.md).
- Developer tooling (plan [007](../007-safe-development/readme.md)).

## Design

### Three kinds of problem

| Kind                  | Example                                         | How it is handled                                              |
| --------------------- | ----------------------------------------------- | -------------------------------------------------------------- |
| An expected condition | a folder cannot be read; Docker is not running  | not an error: part of the result, counted by kind and shown    |
| A failure             | the settings file is corrupt; a removal failed  | a typed error, mapped to a user message with a next step       |
| A bug                 | an invariant is broken                          | the operation stops safely; cleaning never continues past one  |

- Each crate has its own typed errors. At the engine's edge every error becomes one shape for the UI: a code, a
  dictionary key with arguments, a severity and an optional action ("Open System Settings", "Rescan"). The UI shows text
  from the dictionary, never a debug string.
- One failing module does not fail the scan: its section shows the problem, the others finish.

### Logging

- `tracing` everywhere; by default into a **ring buffer in memory**, gone when the app quits. No log file.
- "Record a log file for this session" is an explicit switch in settings, off again at the next launch.
- Logs never contain file contents. Paths are kept in memory only; in anything written to disk the home folder is
  shortened to `~`.
- The phases of a scan are always timed (entries, seconds, errors by kind): rule 4 of [`rules.md`](../../rules.md) needs
  the numbers, and the user may see them.

### Explain

Every finding can answer "why is this here": the rule, the conditions that held, the claim it won. Every skipped or
refused item carries its reason. This is one mechanism, used by the UI ("Why this?"), by the developer tools and by the
tests.

### The diagnostics report

- An "Advanced" corner of settings: versions, permissions, scan statistics, recent problems.
- "Save a report…" writes a text file where the user chooses. The app shows the content first; paths can be left out
  with one switch. Nothing is sent.
- After an unexpected exit the next launch says so plainly and offers the report; it sends nothing either.

## Plans (after approval)

| #   | Plan              | What                                                                  |
| --- | ----------------- | --------------------------------------------------------------------- |
| 01  | Error model       | typed errors per crate, the user-facing shape, module isolation       |
| 02  | Logging           | the ring buffer, the session log switch, scan timings                 |
| 03  | Explain           | reasons on findings and on skips                                      |
| 04  | Diagnostics       | the Advanced section, the report, the unexpected-exit notice          |

The error model (01) is small and belongs right after plan 002, before the first modules; the rest can follow.

## Ideas this touches

"Why it matched, and how sure", "reasons for every skip" and the read-only report of
[best practices](../../ideas/best-practices.md); the [storage explainer](../../ideas/storage-explainer.md).

## Decisions

None yet.

## Agent's calls (to check)

1. **No log file by default**; a per-session switch for one.
2. **Unreadable folders and missing tools are results, not errors.**
3. **A bug stops the operation**; a release build still aborts on a panic, and the next launch reports it.
4. **The report is saved by the user and never sent.**
5. **The error model is built right after plan 002.**
6. **Target version 0.5.0.**

## Conclusions

Not written yet.
