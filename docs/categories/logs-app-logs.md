# Old app logs

**Danger level:** Safe · **Rule:** none (the module's code, `modules/logs`) · **Since:** 0.1.0

**Example:** logs in `~/Library/Logs` untouched for a week

## What it is

Records apps write about what they did, kept for finding problems. Nothing has written to these files for at least a
week (the setting runs from 7 to 90 days and can only be raised).

## Where it lives

`~/Library/Logs/**`, up to six names deep, except `DiagnosticReports` (the crash reports' category), `JetBrains` (the
`jetbrains` module's) and top-level folders shaped as a bundle id (`app-leftovers`'); a top-level file of that
shape is an app log. Each file is one finding.

## What happens if you remove it

The file goes to the Trash and its folders stay, so each app finds its log folder where it left it. A log changed
recently is kept, and one in use when you confirm is skipped. The old records do not come back; apps write new logs as
they run.

## How the app decides

The module `logs` (code, not rules: the answer depends on kinds, owners and times). One finding per file. For each
entry, the first answer that applies:

| #  | Situation                                                                                                   | Result                                         |
| -- | ----------------------------------------------------------------------------------------------------------- | ---------------------------------------------- |
| 1  | a folder or a link                                                                                          | not claimed                                    |
| 2  | the first name below `Logs` is `JetBrains` or a folder shaped as a bundle id                                | not claimed                                    |
| 3  | the name ends with `.plist`, `.db`, `.sqlite`, `.sqlite-wal`, `.sqlite-shm`, `.lock` or `.pid`             | not claimed                                    |
| 4  | the scan is shown from the last scan                                                                        | left out: scan again to check its age          |
| 5  | a socket, a pipe or a device                                                                                | kept for good: a program uses it to talk to another |
| 6  | not yours, locked by the system, or restricted                                                              | kept for good: it belongs to the system             |
| 7  | locked by you (Finder's Get Info)                                                                           | left out: locked                               |
| 8  | the newest modification or status change is unknown, in the future, or not more than the setting's days ago | left out: changed recently                     |
| 9  | otherwise                                                                                                   | **offered**                                    |

The setting is `logs.older-than-days`: 7 to 90 days, 7 by default. "More than N days" is exact. The access time is
never read: reading a log does not keep it.

Never claimed: a folder, a link, `JetBrains/` (the `jetbrains` module's), a bundle-id-shaped top folder
(`app-leftovers`'), `.plist`, `.db`, `.sqlite`, `.sqlite-wal`, `.sqlite-shm`, `.lock`, `.pid`, anything outside
`~/Library/Logs`, and the system-wide logs.
