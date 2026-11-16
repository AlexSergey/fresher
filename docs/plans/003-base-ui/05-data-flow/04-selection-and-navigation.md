# 003/05/04 — Selection and navigation

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `app/ui/src/state/selection.ts` — `Selection`: `{ findings: ReadonlyMap<FindingId, Picked>, categories:
  ReadonlyMap<CategoryId, ReadonlyMap<FindingId, Picked>> }`. `Picked` keeps the finding's category, sizes and danger
  as the core sent them, so the summary needs no request. A selected category stands for all its findings, minus the
  ones the user ticked off inside it (its map of exclusions), so selecting a category of 100,000 findings loads none of
  them. Actions `toggle-finding`, `toggle-category`, `clear`; cleared on `scan-started`.
- `app/ui/src/state/selection-summary.ts` — `summarize(selection, totals)` → `{ count, sizes, highest }`: a selected
  category counts its total from the overview minus its exclusions, a finding its own sizes; sizes add with the core's
  rule (the weakest completeness wins, the sum saturates); `highest` is the category's highest level (exclusions do not
  lower it: the core's total does not say which finding holds it) or the finding's. A preview: the confirmation shows
  only the core's cleanup plan (plan 008).
- `app/ui/src/state/navigation.ts` — `type Screen = { kind: 'overview' } | { kind: 'module'; module: ModuleId } |
  { kind: 'review' } | { kind: 'settings' }`; `navigate(screen)`, `useScreen()`; a module screen for a module the core
  no longer lists falls back to the overview.
- `app/ui/src/state/defaults.ts` — what is pre-selected: nothing. (Recorded so a later "select all Safe" never
  reaches above Safe: the function that selects by level refuses any level but `safe`.)

## Produces

The selection the cards, rows, selection bar and review screen share; the screen union the app frame renders.

## Checks

- Negative: `nothing is selected after a scan starts`, `selecting by level refuses anything above safe`, `a partial
  size makes the summary partial`, `a sum past the largest size saturates`, `a module screen of an unknown module falls
  back to the overview`, `ticking a finding off inside a selected category loads no other finding`, `an exclusion does
  not lower the category's highest level`.
- Positive: `selecting a category counts its total`, `a category minus an exclusion counts the rest`, `the summary's
  highest level is the highest selected`, `toggling a finding twice leaves the selection as it was`, `navigate changes
  the screen`.
- `npx vitest run src/state` passes.

## Notes

- The selection lives in the core store: `CoreState.selection`, changed by the actions `toggle-finding { finding }`,
  `toggle-category { category }` and `clear` (`reduceSelection` in `selection.ts`), and emptied by `scan-started` in
  the same reducer. Components read it with `useCore((s) => s.selection)` and change it with `store.dispatch`; no
  second store. `Picked` is `{ category, danger, sizes }` (no finding is kept).
- Selecting a category takes in the findings of it that were picked one by one; ticking a finding inside a selected
  category toggles it in that category's exclusions.
- The summary's sums stay at `LARGEST_SIZE` (2^53 − 1 bytes), the largest byte count a TypeScript number holds
  exactly; the core's own sums stay at the largest `u64`. A category the overview no longer lists counts nothing.
- `selectByLevel(selection, totals, level)` answers `{ kind: 'selected', selection }` for `safe` (every category
  whose highest level is Safe) and `{ kind: 'refused', level }` for any other level.
- Navigation: `createNavigation()` → `{ getScreen, navigate, subscribe }`, provided by `NavigationContext`;
  `useNavigation()` gives `navigate`, `useScreen(modules)` the screen to show among the modules the core lists
  (`null` while unknown: the screen is kept), through `screenToShow`.
