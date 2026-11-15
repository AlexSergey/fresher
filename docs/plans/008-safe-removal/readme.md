# 008 — Safe removal and the cleanup journal

**Status:** 📝 draft · **Version:** 0.5.0 · **Date:** 2026-11-15 · **Modules:** core, cleaner, macos, store, engine

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The one path by which anything is ever removed: from the user's selection to the Trash, with every rule of
[`restrictions/deletion-safety.md`](../../restrictions/deletion-safety.md) enforced in code, and a journal of what was
done. Every feature that cleans goes through it.

## Goals

1. Nothing is removed that the user did not confirm, in this session, as that exact list.
2. Every safety rule is a check in the cleaner, below every module: a wrong rule or a wrong selection cannot get past
   it.
3. A mistake is recoverable: Trash first, and a journal that can put things back.
4. The result is honest: what was removed, what was skipped and why, how much space came back.

## Non-goals

- Running as administrator (plan [006](../006-permissions/readme.md)); emptying the Trash — the app never does.
- The confirmation screen's look (plan [003](../003-base-ui/readme.md)).

## Design

### The flow

1. The user selects findings. The engine builds a **plan**: for each item its identity, kind, size, danger level and
   removal, frozen as they were shown.
2. The UI shows the summary — the exact list, the total, the highest danger level, Trash or permanent — and the user
   confirms.
3. Confirming turns the plan into a **confirmed plan**. The cleaner's only entry point takes a confirmed plan, and
   nothing but a confirmation can create one: the type system holds rule 2 of [`rules.md`](../../rules.md).
4. A plan dies when the app quits, when a rescan changes its items, or after a short time unconfirmed.
5. The cleaner works item by item, in order, cancellable between items, and reports each one as an event.

### Checks right before each item

- the physical path (links resolved) is inside the scanned roots;
- it is not a protected path nor an ancestor of one; not the home folder or one of its standard folders as a whole;
- it is still what was confirmed: same kind, not replaced by a link, not grown far past the confirmed size;
- it is not in use by a running app or service;
- the capability for it is granted;
- the protection list and the ignore list could be read — otherwise nothing is cleaned (fail closed).

A failed check skips the item with its reason. The cleaner never forces, never retries with more rights, never quits an
app.

### Kinds of removal

| Removal               | How                                                                                         |
| --------------------- | ------------------------------------------------------------------------------------------- |
| Move to Trash         | the system call that returns where the item landed, so it can be put back                   |
| Empty, keep the folder | for caches whose app expects the folder to exist                                           |
| Tool command          | Docker and similar: a fixed argument list through the command port, never a shell string    |
| Permanent             | a separate explicit choice per confirmation                                                 |

The same pipeline runs as a **dry run**: every check, no action. The simulator, the tests and development builds use it.

### The journal

- Every cleanup is one record: when, each item, the rule that offered it, its size, the removal, where it went, the
  outcome. Append-only, in the app's data folder, inside the footprint budget.
- A history screen lists cleanups; an item still in the Trash can be put back to its place.
- The freed space is reported as measured on the volume, next to the estimate that was promised.

### Also

- One deletion point in the whole codebase (the existing lint), and a written threat model: links, parents of protected
  items, live owners, the time between check and removal.
- Tests: the fake Trash and sandboxes of plan [007](../007-safe-development/readme.md); property tests that no plan can
  remove anything outside its roots or inside a protected path.

## Plans (after approval)

| #   | Plan              | What                                                                    |
| --- | ----------------- | ----------------------------------------------------------------------- |
| 01  | Plan and confirm  | the plan types, the confirmation, expiry                                |
| 02  | Guards            | protected paths, the per-item checks, fail closed, the threat model     |
| 03  | Removals          | Trash, empty-and-keep, tool commands, permanent; the dry run            |
| 04  | Journal           | records, the history of cleanups, put back, measured freed space        |

Depends on plans 002, 006 and the guards of 007.

## Ideas this touches

[Cleanup preview and safe removal](../../ideas/cleanup-preview-and-safe-removal.md) and the
[cleanup journal](../../ideas/cleanup-journal.md) (both become this plan), the
[cleanup simulator](../../ideas/cleanup-simulator.md) (the dry run), the
[cleanup guardian](../../ideas/cleanup-guardian.md) (it reads the journal later).

## Decisions

None yet.

## Agent's calls (to check)

1. **Confirmation enforced by types**: only a confirmed plan can reach the cleaner.
2. **Sequential removal**, cancellable between items.
3. **The journal is an append-only file** (see plan 002, call 1), with put back only for items still in the Trash.
4. **Tool commands are fixed argument lists.**
5. **A plan expires** on quit, on a changed rescan, or unconfirmed after a short time.
6. **Target version 0.5.0.**

## Conclusions

Not written yet.
