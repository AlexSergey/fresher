# A project's last activity, precisely

**Agreed:** 2026-11-16, by delegation (plan 009, decision 6) · **Due at:** the plan of the stale-project module, its
first consumer

## What is postponed

Plan 009 gives each project the modification time of its own folder as its last activity: it changes when a file is
added, removed or renamed at the top of the project (which atomic saves do), not when a file deeper inside is edited in
place. The [stale project idea](../ideas/stale-project-cleanup.md) wants the newest change of any of the project's own
files, outside its artifact folders, and the last commit date of a git repository.

## What doing it needs

- A query of the walked tree before it is committed (the walk keeps every folder's modification time; files under
  1 MiB are folded into their folder), answered for a folder while skipping named children; or the walk's own
  aggregation of the newest time per subtree.
- The engine filling `Project::last_activity` from it before the modules' `finish`.
