# A project cleaned once looks active again for the days of the setting

**Status:** open · **Updated:** 2026-11-19

## Symptom

After the user removes a stale project's `node_modules`, the same project's other artifact folders (its `target`, a
`.venv`) are no longer offered on the next scan, and the project is kept as recent for the days of the setting
(365 by default).

## Cause

Removing a folder changes its parent: the project folder's own modification time moves to the day of the removal, and
that time counts as the project's last activity. The core's end-to-end flow 10 of plan
[021](../plans/021-stale-projects/00-design.md) (`crates/e2e/tests/stale_projects.rs`) pins this behaviour.

## What was tried

No fix was tried: a project's folder time cannot tell its owner's change from the cleaner's. The category documents
say so. As a mitigation, the scan offers all of a stale project's folders together, so the user can select them in one
cleanup.

## Options left

- Skip the project folder's own time when it is the only newer one and its artifact folders' names are the only thing
  missing (needs a fact of what changed, which the walk does not keep).
- Remember the cleaner's own removals in the history and ignore a time that equals one (a coupling of the cleaner's
  journal to the project finder).
