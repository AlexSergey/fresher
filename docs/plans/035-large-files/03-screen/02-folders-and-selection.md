# 035/03/02 — Filters, Folders and the selection

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** app/ui

The rest of the screen: the three filters of Files, the Folders view with its breadcrumb and drill-down, and the rows'
checkboxes wired to the app's selection through the core's pick, with the selection bar on this screen. Nothing is
pre-selected and there is no "Select all" here. The design: [`../00-design.md`](../00-design.md), section 8.

Read first: 03/01 (the screen, the hook, the row), `app/ui/src/state/selection.ts` (`toggle-finding` of a `Finding`,
`Picked`), `app/ui/src/state/store.ts` (how a screen dispatches), `app/ui/src/screens/review/review-groups.ts` (a
picked finding is grouped by its category; Review reads `<category>.name` and `.if-removed`, here `core.own-file.*`),
`app/ui/src/product/selection-bar/` and where Scan renders it (`app/ui/src/screens/scan/scan-screen.tsx`),
`app/ui/src/design/layout/tabs/` (the design system's tabs), the design system's select or segmented control if there
is one (`app/ui/src/design/primitives/`).

## Files

- `app/ui/src/screens/largest/largest-screen.tsx`:
  - two views in the design system's `Tabs`: "Files" and "Folders" (the screen's own state, Files first);
  - Files' filters above the list: kind (from `page.kinds`, each "{kind} ({count})", "Any kind" first), place (from
    `page.places`, "~/{name} ({count})", "Anywhere in your home" first), and "Not changed for" (`Unchanged`, the
    core's `core.largest.unchanged.*` texts); a change asks the first page again with the query; the filters are the
    design system's existing control (no new design component: if none fits a choice of a few, use the native
    `select` styled by the screen's CSS module);
  - the selection bar as Scan shows it.
- `app/ui/src/screens/largest/folders-view.tsx` (new), `.test.tsx` — `getLargestPlace(folder, offset, 100)`: the
  breadcrumb from `place.path` (each step a button but the last, the home first, written "Home"), the rows
  (`own-item-row` with `onOpen` when `opens`), "Show more" as in Files; opening a folder asks its first page and moves
  the focus to the view's heading (`screens/keep-focus.ts`); the folder is kept while the user switches tabs.
- `app/ui/src/screens/largest/use-own-pick.ts` (new), `.test.ts` — the row's toggle: when the row is not ticked, call
  `pickOwnItem(node)` once, then dispatch `{ type: 'toggle-finding', finding }`; when it is ticked (its `picked` id, or
  the finding the hook got back, is in `selection.findings`), dispatch the same toggle with the kept finding; a refused
  pick leaves the row unticked (the app's error toast shows the core's text). The hook keeps the picked findings by
  node for the session's state id and forgets them when the selection is cleared at a scan's start.
- `app/ui/src/locales/en.json` — the tabs, the filters' labels and their "any" choices, "Home", the breadcrumb's
  accessible name.

## Checks

Vitest, negatives first:

- `largest-screen.test.tsx`: negative — `no row is ticked when the screen opens`, `a withheld row cannot be ticked`,
  `a refused pick leaves the row unticked`, `there is no select all`; positive — `ticking a row picks it and puts its
  finding in the selection`, `unticking a row takes it out without asking the core`, `a filter asks the first page
  again`, `the filters list the core's kinds and places with their counts`, `the selection bar shows the picked total`.
- `folders-view.test.tsx`: negative — `a folder that does not open has no Open button`, `the last step of the way is
  not a button`; positive — `the home opens first`, `opening a folder shows its rows and the way back`, `a step of the
  way opens that folder`, `the focus moves to the view's heading`.
- `review-screen.test.tsx` (existing file, one case added): positive — `a picked own file is listed under Your own
  files in the High zone`.
- `npx vitest run app/ui/src/screens/largest app/ui/src/screens/review`; `npm run lint:ts`; the gallery test.
- Commit: `feat(ui): the Largest screen's filters, folders and selection`.

## Conclusions

- The Files/Folders tabs, the three filters (kind, place, changed) as the design system's Select, the Folders view with
  its breadcrumb and way back, the selection bar and the ticking of a row (the core picks first, then the box is
  ticked).
- Review found stale folder and place ids after a rescan (a `StaleNode` for ever) and a pick answered after a new scan
  landing an old finding in the selection: the screen resets its folder and place when the scan id changes, a folder
  that fails to read falls back to the home, and a pick answered after a scan started is dropped.
- Vitest 1 249 tests at this step. Open: a pick in flight is not guarded against a manual Clear.
