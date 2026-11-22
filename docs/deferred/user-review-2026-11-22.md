# The user's review of 2026-11-22

**Agreed:** 2026-11-22, by the user's delegation (build autonomously, write down what the user must look at) ·
**Due at:** the user's next session

## What is postponed

Walks, unverified facts and agent's calls of the plans built on 2026-11-22. Each plan has a section; when the user has
looked at a plan's result, its section is removed, and the file and its row with the last one.

## 033 — Logs and crash reports

Plan [033](../plans/033-logs/readme.md), awaiting the user's walk.

- **Walks by hand**:
  - `rm -rf sandbox/logs && npm run dev:sandbox -- logs`; walk `EXPECTED.md` line by line, negatives first: nothing of a
    negative is offered by `logs`; the fresh, six-day, socket, system and locked cases read as kept with their reasons.
    Each positive is offered at Safe with its title and reason.
  - Select all, Review names Safe, confirm to the Trash. Every log folder is still there. Cleanups, put back.
  - The setting raised to 14 days and a rescan. Ignore one item, the category and the module; restore each.
  - `npm run dev:sandbox -- jetbrains` and `-- app-leftovers`: their logs are still offered by their own modules only.
- **The dry run on the developer's Mac** (`npm run dev`, a scan of `~`):
  - Every offered file is under `~/Library/Logs` and older than 7 days: `find ~/Library/Logs -type f -mtime +7 | wc -l`
    beside the findings' count.
  - Nothing under `Logs/JetBrains` or a `com.*`-shaped folder is offered.
  - The largest ten findings, looked into read-only, for state that is not a log.
  - How many findings a real Mac shows and whether Details stays usable (agent's call 4); the scan time beside plan
    030's (agent's call 6).
- **Facts not verified**:
  - The macOS version the reports' layout was read on (27.2.1).
  - Whether Console still lists a report after it is put back from the Trash.
  - Whether any app on the Mac writes logs deeper than six names.
- **Rulings worth a second look** (the design's agent's calls, [`00-design.md`](../plans/033-logs/00-design.md)):
  - Call 1: `logs` claims every file under `~/Library/Logs` whatever its name, at Safe: the second exception of
    "Unknown is not junk" ([`deletion-safety.md`](../restrictions/deletion-safety.md)). If you reject it, the module
    shrinks to known names.
  - Call 3: top-level dotted files (`system.log`, `install.log`) are offered, while a bundle-id-shaped folder stays
    `app-leftovers`'. Changed in a fix round: refusing every bundle-id-shaped name hid almost every top-level log.
  - Call 5: no app attribution and no running-app check. A log held open is protected only by the cleaner's open-file
    check, which does not see other users' processes.
  - Calls 2 and 7: installed apps' id-named log folders are deferred, against the draft; state and lock suffixes are
    not claimed.
- **Known gaps**: the deferred item [`logs-more-places.md`](./logs-more-places.md); the logs of other users'
  processes held open are not seen by the cleaner ([known issue](../known-issues/cleaner-other-users-processes.md)).
