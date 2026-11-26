# The bug-report file

**Agreed:** 2026-11-17, by the user (plan 011, the light form) · **Due at:** 0.2.0 (moved from before the first release by the user, 2026-11-26)

## What is postponed

"Save a report…": a text file the user makes, reads before saving and sends themselves — the app's version, the
permissions, the last scan's statistics, the recent problems, and the session's log when one was recorded; paths left
out with one switch. After an unexpected exit, the next launch says so plainly and offers the report. Nothing is ever
sent ([`../restrictions/privacy.md`](../restrictions/privacy.md)).

## Why

It needs the [log of a session](./session-log-file.md) and a place in the UI
([diagnostics](./diagnostics-screen.md)); the first modules need neither.

## What doing it needs

1. The [session log](./session-log-file.md) and the [diagnostics corner](./diagnostics-screen.md).
2. A preview of the whole content before saving; the save dialog of the system; no network.
3. A marker of a clean exit in the data folder, to tell an unexpected one at the next launch.
