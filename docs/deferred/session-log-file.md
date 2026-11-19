# A log file of a session

**Agreed:** 2026-11-17, by the user (plan 011, the light form) · **Due at:** before the first release

## What is postponed

Logs on disk, only when the user asks: "Record a log file for this session" in settings, off again at the next
launch. Also the scan's phases timed every time (entries, seconds, problems by kind), which rule 4 of
[`../rules.md`](../rules.md) needs as numbers the user may see. Plan 011 built the problems and the reasons; logs stay
in memory through `tracing`, with no subscriber that writes to disk.

## Why

The first feature modules do not need a log file: a problem reaches the user as a text, and a left-out as a counted
reason. A log on disk is a privacy decision of its own (paths reveal a lot) and is best made with the bug report that
would carry it.

## What doing it needs

1. A `tracing` subscriber that keeps a bounded ring buffer in memory by default; no file.
2. The opt-in switch, read at launch and reset afterwards; the file in the app's data folder, through `fresher-store`.
3. No file contents ever; the home folder shortened to `~` in anything written; a test that a written line holds no
   path outside `~`.
4. The scan's phase timings as one typed record, shown with the scan's result.
5. Plan 026 built a development-only record (`app/tauri/src/dev_runs/`): a new file per run never overwritten, an
   append-only event file, an atomic snapshot, a core text in English. The opt-in log may reuse that writer; its privacy
   decisions (the home shortened to `~`, no path outside it, the location) stay this item's.
