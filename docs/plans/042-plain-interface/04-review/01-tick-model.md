# 042/04/01 — The tick model

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

A "Look before removing" category ticked on Scan comes to Review with nothing ticked; each of its items goes only when
the user ticks it. The selection the core receives names only what goes, so the core does not change. The design:
[`../00-design.md`](../00-design.md), section 8; agent's calls 4 and 20. This is the step that keeps rule 2 on the new
screens: read it twice.

## Read first

`state/{selection.ts,selection-summary.ts,reduce.ts,defaults.ts}` and their tests, `screens/review/{review-groups.ts,
review-group.tsx,review-screen.tsx}`, `screens/scan/scan-groups.tsx` (03/01), `screens/category/category-screen.tsx`
(its "Select" of the category), `product/selection-bar/`, `screens/frame-sidebar.tsx` (the badge),
`product/danger-zone/zones.ts`, `e2e/flows.spec.ts` (every flow that selects a Medium or High category and removes),
`docs/restrictions/deletion-safety.md` ("Explicit confirmation only").

## Files

- `product/danger-zone/zones.ts` — `export const BY_HAND: ReadonlySet<DangerLevel> = new Set(['medium', 'high'])`
  (a comment: the levels whose items the user ticks one by one; their names and lines are the core's). `PART_OF`,
  `PART_ORDER` and `DangerPart` stay until 04/02.
- `state/selection.ts`:
  - `Selection` gains `toCheck: ReadonlySet<CategoryId>`: the categories of a `BY_HAND` level the user brought to
    Review; none of their findings goes unless it is in `findings` (ticked one by one) or the category is in
    `categories` (the user ticked all of it in Review, below). `NOTHING_SELECTED` has an empty set.
  - `reduceSelection(selection, action, levels)` with `levels: (category) => DangerLevel | undefined` (the category's
    highest level from the overview's totals; `reduce.ts` passes it).
  - `toggle-category` of a `BY_HAND` category: when it is in `toCheck` or `categories`, everything of it leaves
    (`toCheck`, `categories`, and its findings ticked one by one); otherwise it joins `toCheck` (its findings ticked
    one by one stay ticked). Other levels as today.
  - a new action `{ type: 'tick-all'; category }` (Review's all-checkbox): the category joins `categories` whole and
    leaves `toCheck`, its one-by-one findings folded in, as `toggleCategory` does; and `{ type: 'untick-all';
    category }`: the category leaves `categories` and its findings, and stays in `toCheck` when its level is
    `BY_HAND`.
  - `choose` (a module's choice, a group's Select all): a `BY_HAND` category joins `toCheck`, never `categories`.
  - `toggle-finding` as today (a `BY_HAND` category not in `categories` keeps its ticks in `findings`).
  - `clear` empties `toCheck` too.
  - `checkedOf(selection, category)`: a category in `toCheck` reads `'mixed'` when some of its findings are ticked,
    else `true` (the tile shows it is brought to Review); others as today.
  - `cleanupSelectionOf` unchanged in shape: `toCheck` adds nothing.
- `state/defaults.ts` — `selectByLevel` refuses every level but Safe (unchanged); the comment mentions `toCheck`.
- `state/selection-summary.ts` — `summarize` counts only what goes (as now); its answer gains `toCheck: { categories:
  number; unticked: number }` (`unticked`: for each category in `toCheck`, its total's count minus its findings ticked
  one by one).
- `state/reduce.ts` — passes `levels` from `state.overview?.totals`.
- `screens/review/review-groups.ts` — `ReviewRows` gains `{ kind: 'to-check'; ticked: ReadonlyMap<FindingId, Picked> }`
  for a category in `toCheck` (its findings ticked one by one move from the `picked` group into it); `reviewGroups`
  lists it (count and size of what is ticked; `listed` its total count). `screens/review/review-group.tsx` renders it
  with the paged `CategoryRows`, `isSelected = ticked.has(id)`, toggling `toggle-finding`.
- `screens/category/category-screen.tsx` — the category's "Select" checkbox is not shown for a `BY_HAND` category (its
  rows still tick one by one).
- `product/selection-bar/` — when `toCheck.categories > 0`, after the count and size: `ui.selection-bar.to-check`
  "· {count} to check in Review" (plural); the bar shows when anything goes or anything is to check.
- `screens/frame-sidebar.tsx` — the Review badge counts what goes; with nothing going and categories to check, it
  counts those with `ui.screen.frame.review-to-check` "{count} to check in Review".
- `screens/review/review-screen.tsx` — "Move to Trash…" is inactive while nothing goes (a description
  `ui.screen.review.remove.none` "Tick at least one item first."); the empty state only when nothing goes and nothing
  is to check.
- `e2e/flows.spec.ts` (and any spec that selects a Medium or High category and removes) — after selecting such a
  category on Scan, tick its items in Review as a user would; no flow is weakened. New, negative first: "a Look before
  removing category ticked on Scan sends nothing until its items are ticked" (the mock's `prepareCleanup` gets no
  finding of it); "Move to Trash stays inactive while only categories to check are selected"; positive: "an item ticked
  in Review goes"; "ticking all of a Look category in Review sends the category".

## Tests (negatives first)

`selection.test.ts`, `describe('negative cases')`: "a Look before removing category ticked on Scan selects none of its
items"; "unticking it drops its items ticked one by one"; "a module's choice never selects a Look category whole";
"Clear empties the categories to check"; "a category to check adds nothing to the core's selection". `positive
cases`: "an item ticked one by one goes"; "tick-all selects the category whole and it leaves the list to check";
"untick-all keeps a Look category to check"; "a Safe category ticked on Scan goes whole, as before"; "checkedOf reads a
category to check as checked, mixed once some items are ticked". `selection-summary.test.ts`: negative "a category to
check counts nothing that goes"; positive "it counts its unticked items". `review-groups.test.ts`: negative "a category
to check with no tick sums to nothing"; positive "it is listed with its ticked items". Selection bar, sidebar and
category screen: one negative and one positive each.

## Checks

`npx vitest run src/state src/screens/review src/screens/category src/product/selection-bar src/screens`;
test:cov:ui; lint:ts, lint:code, knip; `npx playwright test e2e/flows.spec.ts e2e/apps.spec.ts e2e/visual.spec.ts
--project webkit --project chromium` (baselines only if a visible line changed; look at each).

Commit: `feat(ui): a Look before removing item goes only when ticked by hand`.

## Conclusions

- `Selection.toCheck`: a Look category ticked on Scan joins it and sends nothing; `tick-all` (only from Review's
  all-checkbox) and `untick-all`; `choose` puts a Look category to check and drops one the new choice no longer holds
  (ruling D-3); a category whose level the overview does not tell counts as Look. The core's selection gets only what
  goes. Counts follow `selection-summary.ts` (ruling D-4).
- No Select all for a Look category on the details; Look tiles on Scan read checked or mixed and untick.
