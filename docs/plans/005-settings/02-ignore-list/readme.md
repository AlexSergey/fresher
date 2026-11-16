# 005/02 — Ignore list

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, engine, app, e2e

Ignoring and restoring a category or an item, its findings left out of every answer, and an ignored path protected
from removal. The design: [`../00-design.md`](../00-design.md), section 4.

Depends on 01/02.

## Steps

| #   | Step                                                  | Status      | Shared files                                                            |
| --- | ----------------------------------------------------- | ----------- | ----------------------------------------------------------------------- |
| 01  | [Ignore and restore](./01-ignore-and-restore.md)      | ✔️ done    | `crates/core/src/command.rs`, `crates/engine/src/engine.rs`, the bindings |
| 02  | [Left out and protected](./02-left-out-and-protected.md) | ✔️ done    | `crates/engine/src/{results,cleanup/context,cleanup/pending}.rs`, `crates/e2e` |

In order.

## Conclusions

Built as written (with plan 01, one commit).

- The engine keeps `Ignored` (the two lists) inside `Results`, set from the settings at start and after every change;
  `page`, `finding` and `totals` skip what it hides, so Restore works at once. A plan is expanded from
  `Results::visible()`: an ignored finding's id is `UnknownFinding`, an ignored category's pick is `NothingToClean`.
- `Context::now` gives the cleaner `Protection::default().with_ignored(paths)`; a list the protection refuses blocks
  the cleanup with `CleaningBlocked { reason: Settings }` (fail closed). `crates/engine/src/ignore.rs` holds the
  commands and their tests; `crates/e2e/tests/ignore.rs` runs the real scanner and settings file with a golden report
  (`fixtures/golden/ignore.json`). The other goldens gained `ignored` and `settingsChanged`, nothing else.
- `CommandError::CannotIgnore` refuses a tool's object (no path of its own yet).
