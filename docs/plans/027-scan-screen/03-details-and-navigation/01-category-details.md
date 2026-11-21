# 027/03/01 — The category's details

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** app/ui, e2e

Details on a card open the category's own screen: its name, what it is, its level and why, what was left out, the
module's choices that touch it, every finding with its checkbox, and the selection bar; Back returns to the overview.
The module pages, their custom-view registry and the sidebar's module sections go (feedback items 2, 9, 17). The
design: [`../00-design.md`](../00-design.md), sections 1, 5 and 6 (agent's calls 11 and 14; decision 5).

## Files

- `app/ui/src/state/navigation.ts` — `Screen`: `{ kind: 'category'; category: CategoryId }` in place of `module`;
  `screenToShow(screen, categories: null | readonly CategoryId[])` (a category no listed module has falls back to the
  overview); `useScreen(categories)`; comments updated. `navigation.test.tsx`: negatives first (an unknown category
  falls back; while the modules are unknown, the screen stays), then a known category stays.
- `app/ui/src/screens/screen-stack.tsx` — `screenKey` for `category` (`category:<id>`); `module` gone.
- `app/ui/src/screens/app-frame.tsx` — `contentOf`'s `category` case: the module from `moduleOfCategory`, then
  `<CategoryScreen category={screen.category} module={module} />` labelled by the category's name; `null` while the
  module is unknown. `customViews` and `ModuleScreen` imports go; `useScreen` gets every descriptor id.
- `app/ui/src/screens/frame-sidebar.tsx` — the module sections, `groupTitle`, `trailingOf`, `NOT_SCANNED` and the
  `modules` prop go; Overview (active on `overview` and on `category`), Review, Cleanups, Settings. Icons no longer
  used leave `design/icons/icons.ts` (`Package` if knip reports it).
- `app/ui/src/screens/category/category-screen.tsx` and `category-screen.module.css` (new) — `CategoryScreen` as in
  the design, section 5, items 1 to 9. The toolbar's Back (`ui.screen.category.back` "Back to Overview" — renamed
  "Back to Scan" in 03/02) navigates to the overview; Scan or Stop as the module page had them. The total from
  `overview.totals`; the level `total.highestDanger ?? descriptor.danger`; access from `accessOfCategories([module],
  status)`; left-outs from the store's module outcome filtered to the category; the selection bar from `summarize`.
- `app/ui/src/screens/category/category-about.tsx` (new) — the four sentences as a `dl`, headed
  `ui.screen.category.about` "What it is" (the `About` of `category-section.tsx` without `details`).
- `app/ui/src/screens/category/category-findings.tsx` (new) — `CategoryFindings` moved from `category-section.tsx`
  unchanged but for its file; the frame's height up to 12 rows.
- Moved with `git mv` from `screens/module/` to `screens/category/`: `module-choices.tsx` (+ `.module.css`,
  `.test.tsx`; new `category` prop: only choices whose `categories` include it; its test gains negatives first `it('shows
  no choice that does not touch the category')` and positives), `unavailable-reason.tsx`.
- Deleted: `app/ui/src/screens/module/module-screen.tsx`, `module-screen.module.css`, `module-screen.test.tsx`,
  `category-section.tsx`; `app/ui/src/product/module-section/` (component, CSS, stories, baselines);
  `app/ui/src/modules/registry.ts` (the folder goes); `screens/use-ignored.ts` and its test and `moduleFound` of
  `screens/use-modules.ts` when knip reports them unused.
- `app/ui/src/screens/overview/overview-zones.tsx` — `onDetails` navigates to `{ kind: 'category', category }`.
- `app/ui/src/screens/review/review-screen.tsx` — the empty state's text no longer names a module's page
  (`ui.screen.review.empty.text` "Tick what you want to remove on the overview, then check it here.").
- `app/ui/src/locales/en.json` — new `ui.screen.category.back`, `ui.screen.category.about`,
  `ui.screen.category.level` ("How safe it is to remove"), `ui.screen.category.items` ("The items of {name}"),
  `ui.screen.category.select` ("Select all {name}"), `ui.screen.category.empty.title` ("Nothing found here"),
  `ui.screen.category.empty.text` ("The last scan found nothing of this kind."); the `ui.screen.module.*` keys reused
  by the moved files are renamed `ui.screen.category.*` (`what`, `made-by`, `if-removed`, `comes-back`, `choices`,
  `count`); every other `ui.screen.module.*`, `ui.screen.frame.group-ignored` and `ui.screen.frame.module-ignored`
  removed.
- E2E: every spec that opened a module page from the sidebar opens the category's details from its card's Details
  instead (`flows`, `axe`, `visual`, `keyboard`, `contrast`, `motion`); the module page cases about ignoring a module
  from its page (`Ignore this module on the module page …`, `Restore on its page …`, `a module of an ignored group is
  locked …`) are rewritten around the card's menu and Settings; `the sidebar lists JetBrains IDEs under Development …`
  becomes `the details of a JetBrains category name its level`; `nothing ignored shows … no note on the module page`
  drops the module page.

## Produces

`Screen`'s `category`; `CategoryScreen`; a sidebar without modules — for 03/02.

## Checks

- `category-screen.test.tsx` (new), negatives first:
  - `describe('negative cases')`: `it('shows the empty state when the last scan found nothing of the category')`;
    `it('shows no choice of the module that does not touch the category')`; `it('an unavailable category says why and
    offers no checkbox')`.
  - `describe('positive cases')`: `it('names the category, what it is and its level with the core's meaning')`;
    `it('lists the left-out lines of this category only')`; `it('the head's checkbox selects the whole category')`;
    `it('shows the selection bar once something is selected')`; `it('Back returns to the overview')`.
- `app-frame.test.tsx`: negatives first (a category screen renders nothing while the modules are unknown), then the
  category's details render with the category's name as the screen's label; the sidebar has no module item.
- `npm run test --workspace app/ui` and `npm run test:cov --workspace app/ui` (at or above 90 %).
- `npm run lint:ts`; `npm run knip`; `npx eslint .` in `app/ui`; `npx prettier --check` on the changed files.
- E2E: new in `flows.spec.ts`, negatives first `Back from a category's details returns to the overview with the
  selection kept`; positives `a card's Details opens the category's details with every finding`, `ticking a finding on
  the details shows the selection bar, and its Review lists it`, `a choice on the details of a removed app's category
  selects only its own categories`. `npm run test:e2e:update --workspace app/ui -- gallery.spec.ts visual.spec.ts
  contrast.spec.ts zoom.spec.ts` (look at each PNG; module page baselines removed with `git rm`, details baselines
  added), then `npm run test:e2e --workspace app/ui`.

## Conclusions

Not written yet.
