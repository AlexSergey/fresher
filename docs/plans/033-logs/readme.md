# 033 — Logs and crash reports

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** modules, testing, sandbox, e2e, app, app/ui, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

Old logs and crash reports are the first thing any cleaner offers. Fresher offers the logs of removed apps and of
JetBrains IDEs only; the rest of `~/Library/Logs` belongs to nobody.

## Goals

1. Logs and crash reports of the user's own Library are offered, named by the app that wrote them where it can be told.
2. A log an app is writing now is never touched.
3. One claim per folder with the modules that already own logs.

## Non-goals

- System-wide logs (`/Library/Logs`, `/private/var/log`): they need administrator rights; not offered.
- The unified log, and anything a tool must rotate.

## Design

A new module `logs` (`fresher-module-logs`, group **System**), in the shape of `temp`:

| Category             | Level | What                                                                  |
| -------------------- | ----- | --------------------------------------------------------------------- |
| `logs.app-logs`      | Safe  | files under `~/Library/Logs` older than the threshold, a finding an app |
| `logs.crash-reports` | Safe  | reports under `~/Library/Logs/DiagnosticReports` older than the threshold |

- The threshold is a module setting, 7 days by default: a recent log is what a bug report needs.
- Files go to the Trash; folders stay, so an app never finds its log folder missing.
- Left out with a reason: a file open by a process, a file changed inside the threshold, a special entry.
- Folders already claimed by `jetbrains` (its logs) and `app-leftovers` (a removed app's logs) are not claimed here;
  the same installed apps of the same scan decide, as in plan [030](../030-installed-app-caches/readme.md).

**Sandbox.** `fixtures/sandboxes/logs.toml`, negatives first: a fresh log, a removed app's log folder (the other
module's), a JetBrains log folder, a link at a place, a socket in a log folder, a file under a look-alike folder
outside the Library; then an old log of an installed app, an old log with no app behind it, an old crash report. Ages
are stated in the case file. Golden report, core e2e flow, and the finish walks `EXPECTED.md` in
`npm run dev:sandbox -- logs`.

## Plans (after approval)

| #   | Plan       | What                                                        |
| --- | ---------- | ----------------------------------------------------------- |
| 01  | Module     | the decisions, the setting, the texts, the category files   |
| 02  | End to end | the case file, golden reports, the core e2e flow, the UI    |
| 03  | Finish     | docs, the walk on the developer's Mac                       |

Depends on plans 008, 011, 015, 017 and 019. The walk does not change shape (`Library/Logs/*` is already of interest
to `app-leftovers`): no scan bench is due unless the design finds otherwise.

## Ideas this touches

[System Junk Cleaner](../../ideas/system-junk-cleaner.md) (logs, crash and diagnostic reports).

## Open questions

None.

## Decisions

None yet.

## Agent's calls (to check)

1. **Only the user's Library**; nothing that needs administrator rights.
2. **Older than 7 days**, a setting.
3. **Files go, folders stay.**
4. **A module of its own** rather than a category of `temp`.

## Conclusions

Not written yet.
