# 044/04/01 — The docs

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** docs

Every doc the plan made untrue is made true; the idea and the deferred items it took say so. The design:
[`../00-design.md`](../00-design.md), section 11.

## Read first

Every commit of plans 01 to 03; `docs/architecture/README.md` ("The scan tree", "The cache body", the storage map's
paragraph on `get_storage_place`, "The largest files"); `docs/categories/{README,core-own-file}.md`;
`docs/deferred/{largest-files-more,folder-age-from-children,tester-feedback-0.1.0}.md` and `docs/deferred/README.md`;
`docs/ideas/{README,big-stale-projects}.md`.

## Files

- `docs/architecture/README.md`:
  - "The scan tree" and "The cache body": a folder's `newest` (the later of `modified` and `changed` of the files,
    links and special entries below it, `.DS_Store` left out, folders' own times left out), own until `freeze`, the subtree's after; the format 7; a rescan never
    older than the truth.
  - The storage map's paragraph: the by-place view lives on as Largest's Browse (`get_largest_place`), read-only;
    `Engine::storage_place` has no caller in the UI.
  - "The largest files": **Files** (as today; a plain folder is never picked), **Folders** (`DiskState::left_alone`,
    the verdict table with the ignore list first, the settings `core.largest.folder-min-size` and the stale days, no
    cache) and **Browse** (`largest_place`, read-only: no pick, Show in Finder); the commands `get_largest`,
    `get_largest_folders`, `get_largest_place`, `pick_own_item`, `reveal_node`; the UI (the three tabs, `FolderRow`,
    the bar, the tiles).
- `docs/categories/README.md` — beside `core.own-file`: Largest's Folders and Browse are information views, not
  categories: nothing on them is removed, counted in a total or sent to a plan. `docs/categories/core-own-file.md` — what can be
  picked: a file, a package or a folded subtree from Files, never a plain folder, never from Browse.
- `docs/deferred/largest-files-more.md` — "A folder's Changed from its newest file" goes (the tree has `newest`);
  the rest stays.
- `docs/deferred/folder-age-from-children.md` — a line: the tree's `newest` (plan 044) is known only after the walk,
  so it does not serve a module's `Match`; the item stays deferred.
- `docs/deferred/tester-feedback-0.1.0.md` — item 2: Largest's Folders (plan 044) answers part of "where the rest
  went" (big folders left alone); the line on the scan's result is still open. Item 4: taken by plan 044.
- `docs/ideas/big-stale-projects.md` — its header: taken by plan 044 (Folders, Show in Finder only); what was not
  taken (uncommitted or unpushed work; removal) stays under "Direction" marked "not taken". `docs/ideas/README.md` —
  its row: "taken by plan 044".
- `docs/testing.md` — "Sandboxes" already names the new `[case.largest]` values (02/05); the budget test
  `folders_of_the_large_tree` is added where `items_of_the_large_tree` is described.

## Interfaces produced

- None.

## Tests

None (documentation).

## Checks

- `npm run check:readme` (the links of `docs/`), `npx prettier --check docs`
- `git grep -n -i "storage map keeps browsing\|browsing stays on the storage map" docs` finds nothing

Commit: `docs(docs): Largest's folders left alone`.
