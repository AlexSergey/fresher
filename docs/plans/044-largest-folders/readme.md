# 044 — Big folders left alone, on the Largest screen

**Status:** 📝 draft · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** core, scanner, engine, testing,
sandbox, e2e, app/tauri, app/ui, docs

A one-file draft with the recommended decisions taken. From the first tester's feedback
([`../../deferred/tester-feedback-0.1.0.md`](../../deferred/tester-feedback-0.1.0.md)) and the user's answers of
2026-11-26: "a section like Large Files that shows files and folders, does not offer to remove, only opens the Finder";
Largest gets two sections, Files and Folders; files keep their removal (High), folders are shown only. Takes the idea
[Big projects left alone](../../ideas/big-stale-projects.md). Needs [043](../043-build-flags/readme.md) (Largest shown).

## Goals

1. Largest has two sections: **Files** (as today: the largest files, ticked one by one, removed at High after the
   confirmation) and **Folders**: folders over a size whose last change is old.
2. A folder row shows its name, its path from `~`, its size, when it last changed, and whether it is a project (a `.git`
   folder or a manifest the stale projects know). Its only action is **Show in Finder**: no tick, no removal, nothing
   sent to the cleaner.
3. The rule: a folder in the scanned home, over the size (default 1 GB), whose newest own file and, for a git project,
   last commit are older than the stale projects' days. The highest such folder is listed, never one inside a listed
   one; a folder that holds a recently changed file is not listed, its stale children may be.
4. Never listed: `~/Library`, the home's standard folders themselves (Desktop, Documents, Downloads, …), app bundles,
   the Trash, protected places, places the user ignores, and a folder already explained whole by a Scan category.
5. The size and the age come from the last scan's tree (no new walk); the newest-change date is folded into the tree
   during the walk, so the list costs no second pass.
6. The texts say plainly that the row is information: "Not changed for 14 months. Open it in the Finder to decide."

## Decisions taken (recommended)

- **Show only**: the user's decision (2026-11-26). A whole project is the user's work; the app never offers it.
- **The threshold and the age** reuse settings: the size as a new setting (default 1 GB, a slider like the others), the
  age as the stale projects' days.
- **The folder tree of today's Largest "Folders" view** gives way to the new list; browsing the disk stays on the
  storage map. (Agent's call: check whether the tree should stay as a third view.)
- **Uncommitted or unpushed work** is not read in the first version (it needs `git` run per project); a later plan may
  add it as a line.

## Plans

1. The core: the newest-change date per folder in the scan tree; the query `big_folders_left_alone` with its rule,
   unit tests and sandbox cases (a stale project, a fresh one inside a stale parent, a standard folder, a nested pair).
2. The engine and the IPC: the page, the setting, `reveal` reused for Show in Finder.
3. The UI: two sections on Largest, the folder row, empty and loading states; mockup first (Design canvas), the
   gallery, e2e, axe and visual tests.
4. Docs: `docs/categories/` (an information view, not a category), the test cases, the idea's file marked taken.

## Agent's calls to check

- The old folder tree leaves Largest.
- "Highest qualifying folder" as the nesting rule.
- The default size of 1 GB.
