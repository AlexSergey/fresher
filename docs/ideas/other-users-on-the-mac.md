# Other users on the Mac

**Since:** 2026-11-26 · From the first tester's feedback on 0.1.0

## Problem

Fresher scans the home folder of the user who runs it. A Mac shared by two or more accounts keeps the same kinds of
waste in every home (caches, old `node_modules`, logs, downloads), and the space they take counts against the one disk.

## Direction

- Today the simplest answer is to run Fresher in each account: say so where the storage map shows other homes' space.
- A later step: an administrator's scan of the other homes, through the privileged helper of plan
  [006](../plans/006-permissions/readme.md), each home a separate root with its own owner shown on every row.
- Removal in another user's home is a privilege boundary: the confirmation names the owner, items go to that user's
  Trash (never the admin's), and nothing of another home is ever pre-selected.

Open questions:

- Is reading other homes worth an admin password prompt, or is "run it in each account" enough?
- Files another user has open cannot be checked without root (see the known issue on other users' processes).
