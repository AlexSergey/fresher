# The diagnostics corner

**Agreed:** 2026-11-17, by the user (plan 011, the light form) · **Due at:** before the first release

## What is postponed

An "Advanced" corner of settings: the app's version, the permissions, the last scan's statistics, the recent problems
(a module's failure, a cleanup's failed items with their details), and the entry to the
[bug report](./bug-report-file.md). Also the left-outs of a restored scan: plan 011 sends them with the module's end
only, so a scan restored from the cache shows none.

## Why

Plan 011 shows every problem where it happens (the toast, the module's section, the cleanup's result) and that is
what the first modules need; a screen that gathers them is for support, which starts with the first release.

## What doing it needs

1. A bounded list of recent problems in the engine (texts and codes, no paths), read by one command.
2. The left-outs kept with the scan cache, if the user should see them after a restart.
3. The screen, answering one question: "is the app working, and what did it not do?"
