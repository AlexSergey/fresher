# Big projects left alone

**Since:** 2026-11-26 · The user's extension of the first tester's feedback on 0.1.0 · **Plan:**
[044](../plans/044-largest-folders/readme.md): taken by plan 044, as Largest's Folders (shown on Largest, Show in
Finder only; the user, 2026-11-26)

## Problem

A project folder over a gigabyte that nobody has changed for months takes space as a whole, not only through its
`node_modules` or `target/`: datasets, media, vendored copies, experiments, scratch projects made by AI assistants.
[Stale project cleanup](./stale-project-cleanup.md) offers only the regenerable parts and never the project itself.

## Direction

Plan 044 took the listing (a folder over the size and left alone for the days, a `.git` folder marks a project,
the newest change from the scan tree) and Show in Finder. The rest is **not taken**.

- Show a project (recognized by its manifest or its `.git`) whose size is over a threshold (default 1 GB) and whose
  last change is older than the stale projects' days, with its size, its last change and what is inside it.
- Not taken: it is the user's own work: "Look before removing" at least, never ticked for the user, moved to the
  Trash only; a project with uncommitted changes or commits not pushed anywhere says so in its line.
- The size and the last change use the same rules as the stale projects (the newest of the project's own files and the
  last commit); the regenerable parts already offered are not counted twice.

Open questions:

- Is a whole project offered for removal at all, or only shown (with "Show in Finder") as information? Today's rule
  for stale projects is "never the source code, never the project folder itself": this needs the user's decision.
- The threshold and the age: settings, or fixed for a first version?
