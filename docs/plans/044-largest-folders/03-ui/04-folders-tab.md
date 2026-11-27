# 044/03/04 — The Folders and Browse tabs on Largest

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** app/ui, e2e

The Folders tab shows the folders left alone with the bar, the separator and the tiles; the old folder view moves,
as it is, to a third tab, Browse (03/05 makes it read-only). The design: [`../00-design.md`](../00-design.md),
sections 8 and 9; agent's calls 17 and 21; the approved mockup.

## Read first

03/02 and 03/03's commits; `app/ui/src/screens/largest/{largest-screen.tsx,use-largest.ts,folders-view.tsx}` and
their tests; `app/ui/src/screens/use-row-menu.ts` (a refused reveal); `app/ui/src/design/{data/usage-bar,
layout/section-separator,layout/tile,layout/surface}`; `app/ui/e2e/{flows,visual,axe,keyboard}.spec.ts` (their
Largest parts: flows :394, :411, :1023, :3262; visual :593; axe :505; keyboard :200).

## Files

- `app/ui/src/screens/largest/use-largest-folders.ts` (new) — `useLargestFolders(): LargestFolders` (`{ failed, more,
  page: FoldersPage | null }`), as `useLargest`: asked when a scan ends and again after a cleanup removed something
  or an ignore list changed; nothing asked before a scan or while one runs; older answers dropped by a generation
  counter; a refusal to the store.
- `app/ui/src/screens/largest/folders-list.tsx` (new) — `FoldersList`: the heading (`h2`), the rule line, the
  `UsageBar` of `page.home` with the five largest rows as segments and the rest as one, its caption, the
  `SectionSeparator`, the three `Tile`s, the rows (`FolderRow`, `onReveal` → `revealNode(row.node.id)`, a refusal
  dispatched as `command-failed`) in a list named `ui.screen.largest.folders`, "Show more" while rows remain; the
  empty, loading and unavailable states.
- `app/ui/src/screens/largest/largest-screen.tsx` — three tabs (`View = 'files' | 'folders' | 'browse'`): Folders
  renders `FoldersList`, Browse renders `BrowseView` with the screen's `folder` state (kept, and reset after a new
  scan, as today); the subtitle follows the tab (`ui.screen.largest.folders.summary` on Folders, none on Browse).
- `app/ui/src/screens/largest/folders-view.tsx`, `folders-view.test.tsx` — renamed `browse-view.tsx`,
  `browse-view.test.tsx` (`git mv`), the component `BrowseView`, its list named `ui.screen.largest.browse`, its
  heading `ui.screen.largest.browse.heading`; otherwise unchanged (ticks still there until 03/05).
- `app/ui/src/screens/largest/largest-screen.module.css` — the Folders tab's parts (tokens only).
- `app/ui/src/locales/en.json` — `ui.screen.largest.folders.heading` "Open these in the Finder and decide what to
  keep", `.folders.rule` "Folders of {size} or more where nothing changed for {days} days. Fresher never removes
  them.", `.folders.bar` "{size} of your home's {home}", `.folders.tile.count` "Folders", `.folders.tile.space`
  "Space", `.folders.tile.projects` "Projects", `.folders.empty` "No folder of {size} or more has gone unchanged for
  {days} days.", `.folders.summary.one` "{count} folder, {size}", `.folders.summary.other` "{count} folders, {size}";
  `ui.screen.largest.folders` "Folders left alone, largest first"; `ui.screen.largest.view.browse` "Browse",
  `ui.screen.largest.browse` "Folders and files, largest first", `ui.screen.largest.browse.heading` "Go into a
  folder, or show it in the Finder"; `ui.screen.largest.no-scan` "Scan first to see your largest files and folders.".
- `app/ui/e2e/flows.spec.ts` — `largestFolders` names the new list and `largestBrowse` the tree's; the flows that
  opened a folder (:1023, :3262) open it on Browse; new: "Folders lists the folders left alone, largest first, with
  no checkbox" and "Show in Finder reveals the folder by its id" (`revealed(page)`); a refused reveal shows the core's
  text.
- `app/ui/e2e/visual.spec.ts` — "the largest folders after a scan" photographs the Browse tab and keeps the baseline
  `largest-folders-{chromium,webkit}.png` (re-made: the tab bar has three tabs); a new test "the folders left alone
  after a scan" photographs Folders as `largest-left-alone-{chromium,webkit}.png`; each compared with the mockup
  before it is kept (`--update-snapshots`).
- `app/ui/e2e/axe.spec.ts` (`the largest folders` becomes Browse's; a new `the folders left alone`),
  `app/ui/e2e/keyboard.spec.ts` — both lists' names; Tab reaches each Show in Finder of Folders in order.

## Interfaces produced

- `BrowseView` and the screen's three tabs; 03/05 makes Browse read-only.

## Tests (negatives first)

- `use-largest-folders.test.ts`, negative: `asks nothing before a scan`, `asks nothing while a scan runs`, `reports a
  refusal and keeps no page`; positive: `asks the first page when a scan ends`, `adds the next page on more`, `asks
  again after a cleanup removed something`.
- `folders-list.test.tsx`, negative: `shows the empty text with the size and the days`, `says the list could not be
  read`, `reports a refused reveal`; positive: `shows the heading, the rule and the tiles`, `draws the bar of the
  home`, `shows more rows on Show more`, `reveals a row by its id`.
- `largest-screen.test.tsx`, negative: `the Folders tab has no checkbox`, `the Folders tab asks nothing before a
  scan`; positive: `the Folders tab lists the folders left alone`, `the Browse tab shows the home's folders`, `the
  opened folder stays while another tab is looked at`, `the subtitle follows the tab`.
- `browse-view.test.tsx`: the renamed tests pass with the new names.

## Checks

- `npx vitest run src/screens/largest` (from `app/ui`)
- `npx playwright test --project gallery --project webkit --project chromium e2e/flows.spec.ts e2e/visual.spec.ts
  e2e/axe.spec.ts e2e/keyboard.spec.ts e2e/layout.spec.ts`
- `npm run lint:ts`, `npm run lint:code`, `npm run check:texts`, `npm run knip`

Commit: `feat(ui): Largest's Folders lists the folders left alone, the tree moves to Browse`.
