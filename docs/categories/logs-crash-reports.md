# Old crash reports

**Danger level:** Safe · **Rule:** none (the module's code, `modules/logs`) · **Since:** 0.1.0

**Example:** `~/Library/Logs/DiagnosticReports` older than a week

## What it is

Reports macOS writes when an app crashes or stops responding, to help its developer find the cause. Nothing has changed
them for at least a week (the setting runs from 7 to 90 days and can only be raised).

## Where it lives

`~/Library/Logs/DiagnosticReports/**`, the `Retired/` folder included. Each file is one finding.

## What happens if you remove it

The report goes to the Trash and leaves Console's list of reports; its folders stay. Nothing else changes. If you mean
to send one to its developer, do it first. The old ones do not come back; a new report is written the next time
something crashes.

## How the app decides

The same module and the same table as [`logs-app-logs.md`](./logs-app-logs.md): a file whose first name below `Logs`
is `DiagnosticReports` is a crash report, any other file is an app log. One finding per file. Never claimed: a folder,
a link, `.plist`, `.db`, `.sqlite`, `.sqlite-wal`, `.sqlite-shm`, `.lock`, `.pid`, anything outside `~/Library/Logs`,
and the system-wide logs. The setting is `logs.older-than-days`: 7 to 90 days, 7 by default, one for both categories.
