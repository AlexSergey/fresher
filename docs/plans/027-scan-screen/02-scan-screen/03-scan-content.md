# 027/02/03 — The overview in zones

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** app/ui, e2e

Below the hero, the cards in their danger zones, each with "Select all"; the view switch, the view by place and the
rows under the cards leave; the selection bar shows on the overview (feedback items 8, 11, 12, 14, 15). The design:
[`../00-design.md`](../00-design.md), sections 3, 6 and 9 (decisions 1 and 6).

## Files

- `app/ui/src/screens/overview/zones.ts` (new) — `Zone`, `zonesOf`, `zoneSelectAll` as in the design, section 3.
- `app/ui/src/screens/overview/overview-zones.tsx` (new) — `OverviewZones { access, found, scanning, totals,
  unavailable }`: for each zone of `zonesOf([...totals, ...unavailable not already counted])` a `DangerZone` with
  `selectAll` from `zoneSelectAll` (selectable: `count > 0` and not unavailable; the handler dispatches `choose` as in
  the design), holding a `CategoryGrid` labelled by the zone's name, with `moduleOf`, `onIgnore`, `onDetails` (the
  module page at the category, until 03/01), `onToggle`, `selected`.
- `app/ui/src/screens/overview/overview-screen.tsx` and `.module.css` — the `SegmentedControl`, `MeaningList`,
  `ByPlace` and the `views` CSS go; `OverviewZones` in their place (after the notices and the other volumes' blocks);
  the `SelectionBar` at the end, as the module page renders it (Clear dispatches `clear`, Review navigates to
  Review); the content's bottom padding leaves room for it. `useStorageMeaning(0)` stays for the ring's levels.
- `app/ui/src/state/navigation.ts` — `OverviewView`, `PlaceAt`, `placeTop`, `getOverviewView`, `showOverviewView`,
  `useOverviewView` removed; `navigation.test.tsx` updated.
- `app/ui/src/state/storage.ts` — `useStoragePlace`, `useStorageRoots`, `createPlaceSource`, `PlaceRows`,
  `PlaceSource` and their page cache removed (only `useStorageMeaning` stays); `storage.test.tsx` keeps the meaning
  tests only. `ipc/index.ts` drops the wrappers nothing calls any more (`getStoragePlace`, and the roots' wrapper
  if it has one) once knip reports them; the commands stay in the core and the shell, and the e2e mock keeps
  answering them.
- Deleted: `app/ui/src/screens/overview/by-place.tsx`; `app/ui/src/product/meaning-list/` (all);
  `app/ui/src/product/place-list/` (all); their stories' baselines
  (`app/ui/e2e/screenshots/gallery.spec.ts/{AppRow,MeaningList,PlaceList,…}/`).
- `app/ui/src/product/storage-map/core-keys.test.ts` — the `notScannedText` part goes with `place-list/folder-row`
  (the `core.storage.not-scanned.*` keys stay in the core, which still emits them).
- `app/ui/src/locales/en.json` — removed `ui.meaning-list.*`, `ui.place-list.*`, `ui.screen.overview.view.*`, and
  any key only those files used.
- `app/ui/src/test/*` fixtures and `app/ui/e2e/core-mock.ts` — nothing to remove from the mock (the core still
  answers the place queries); fixtures used only by the deleted files go.
- E2E specs: `keyboard.spec.ts` (`the view by place …` goes), `axe.spec.ts` (`the overview by place` goes),
  `visual.spec.ts` (`a finished scan, by place` goes, its baselines deleted), any spec using `By meaning` or a
  meaning row.

## Produces

The overview's final content (renamed Scan in 03/02); `zonesOf`, `zoneSelectAll`.

## Checks

- `zones.test.ts` (new), negatives first:
  - `describe('negative cases')`: `it('leaves out an empty zone')`; `it('gives no Select all to a zone with nothing
    selectable')` (only unavailable or zero-count categories); `it('an unavailable category is never among the
    categories Select all ticks')`.
  - `describe('positive cases')`: `it('orders the zones Safe, Low, Medium, High')`; `it('puts each category in the
    zone of its highest level, keeping their order')`; `it('is checked when every selectable category is selected
    whole, mixed when some are or some of their findings')`.
- `overview-screen.test.tsx` updated, negatives first: no view switch, no apps' rows, no places, no "rest of the disk"
  rows; no selection bar while nothing is selected; then: the cards sit in their zones; "Select all" of Safe selects
  every Safe category and nothing else (`choose` within the zone); turning it off clears only that zone; the
  selection bar shows the count and Review opens Review.
- `npm run test --workspace app/ui` (the whole UI suite: files were deleted) and `npm run test:cov --workspace app/ui`
  (coverage at or above 90 %).
- `npm run lint:ts`; `npm run knip` (no unused export, file or dependency left by the deletions); `npx eslint .` in
  `app/ui`; `npx prettier --check` on the changed files.
- E2E: in `flows.spec.ts`, negatives first `Select all of the Safe zone ticks nothing in the other zones`; positives
  `Select all of a zone, then the selection bar's Review lists those categories`, `the selection bar's Clear empties
  the selection on the overview`. `npm run test:e2e:update --workspace app/ui -- gallery.spec.ts visual.spec.ts
  contrast.spec.ts zoom.spec.ts` (look at each PNG; deleted baselines removed with `git rm`), then
  `npm run test:e2e --workspace app/ui`.

## Conclusions

Not written yet.
