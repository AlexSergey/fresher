# 006/04/03 — Limited and unavailable sections

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** app/ui

## What exists already

- `ModuleInfo.categories: CategoryInfo[]`, each `{ descriptor, availability }` with `Availability` `ready` |
  `limited { missing }` | `unavailable { missing }`; `ModuleOutcome` has `unavailable { missing }`; the UI reads
  modules through `app/ui/src/screens/use-modules.ts`. No component reads `availability` yet.
- The overview's cards: `OverviewScreen` → `product/meaning-list/meaning-list.tsx` → `product/category-grid` →
  `product/category-card/category-card.tsx` (`CategoryCardProps { total: CategoryTotal, selected, onToggle, onOpen,
  share, underApps? }`). `CategoryTotal` carries `sizes.completeness`, so a `Limited` category's partial size already
  reads `42 GB+` through `SizeText`; it has no availability. A category with no finding has no total and no card.
- The module page: `screens/module/module-screen.tsx` (`shownOf`: a section per category with findings or left-outs,
  `nothingFound(category)` for the latter) → `screens/module/category-section.tsx` (`CategorySectionProps { category:
  CategoryDescriptor, total, leftOut?, … }`, with the category's `Checkbox`), inside
  `product/module-section/module-section.tsx`, which already shows a status for `ModuleOutcome` `unavailable`
  (`ui.module-section.unavailable`). A module that found nothing shows the `EmptyState` "empty".
- Which modules declare needs today: only the dev-only `trial` module; `app-leftovers` declares
  `SeesMore(OtherAppsData)` from step 03/05. No shipped module has a `Required` need yet, so `Unavailable` is seen in
  the gallery, the tests and the `trial` module only.

## Files

- `app/ui/src/product/category-card/category-card.tsx` — an optional `availability` prop:
  - `ready` (or absent) — as today;
  - `limited` — the size as today (already partial) and a tooltip "Some of it could not be seen: <capability
    name>." (`core.access.<id>.name` for each missing capability; the sentence frame is chrome);
  - `unavailable` — the card stays, disabled with `aria-disabled` (not `disabled`, so focus reaches it and VoiceOver
    reads the reason: agent's call 21), its checkbox absent, with the one sentence `core.access.<id>.missing` (a tool:
    its state's note from `AccessStatus`, "Docker is installed but not running. Start it, then scan again.").
- `app/ui/src/product/meaning-list/meaning-list.tsx` (and `category-grid.tsx` if the prop passes through it) — each
  card gets its category's availability from the modules; a category that is `unavailable` has no total, so it is
  shown as a disabled card after the sized ones, with a zero size, so it is never silently absent. (An agent's call of
  this step, to check: the by-meaning view groups cards by meaning; an unavailable category has no meaning yet.)
- `app/ui/src/screens/module/module-screen.tsx` — `shownOf` also keeps a category that is `unavailable`
  (`nothingFound(category)`); a module that ended `unavailable` shows `core.access.<id>.missing` (or the tool's note)
  in place of the "empty" `EmptyState`.
- `app/ui/src/screens/module/category-section.tsx` — the same three renderings as the card: a `limited` section's
  tooltip, an `unavailable` section disabled (`aria-disabled`, no checkbox, its sentence, and for an OS permission the
  same action as its coverage notice from 04/02, `openAccessSettings` or the opt-in switch).
- `app/ui/src/locales/en.json` — the chrome sentence frame of the tooltip.
- Tests and stories next to each changed component: `category-card.test.tsx`, `category-card.stories.tsx`,
  `module-screen.test.tsx`, `module-section.stories.tsx` (or the section's own stories), `meaning-list` tests.

## Produces

Rule "nothing is silently absent" of the design, section 8.

## Checks

- Negative (first): `an_unavailable_category_is_shown_disabled_not_hidden`, `an_unavailable_category_cannot_be_selected`
  (no checkbox, `toggle-category` never dispatched), `a_ready_category_shows_no_access_text`,
  `an_unavailable_module_page_is_not_an_empty_list`.
- Positive: `a_limited_category_shows_a_partial_size_and_why`, `an_unavailable_category_names_what_is_missing`,
  `a_tool_category_shows_the_tools_note`, the disabled card is reachable by keyboard and reads its reason
  (`aria-disabled` with `aria-describedby`).
- `npx vitest run` on the changed files; the gallery shows each availability on a card and on a module page; the
  Playwright screenshots are added and looked at.

## Conclusions

- Built as specified. New `screens/permission/category-access.ts` turns the core's `Availability` and `AccessStatus`
  into one `CategoryAccess` (`limited` | `unavailable`, its text and the missing capabilities), builds the access map of
  the modules and the zero totals of unavailable categories; a tool's sentence is its state's `note`, a capability's
  is `core.access.<id>.missing` (`name` for the limited tooltip). Chrome keys: `ui.access.limited`,
  `ui.access.tool-missing` (a tool with no note), `ui.access.unavailable`.
- `CategoryCard` takes `access`: `limited` wraps the card in the tooltip (focus shows it, and it describes the button);
  `unavailable` is `aria-disabled` (not `disabled`), has no checkbox, never calls `onOpen`, and shows its reason with
  `aria-describedby`. `CategoryGrid` and `MeaningList` pass `access` through; `MeaningList` takes `unavailable` totals.
- Call checked, as written: an unavailable category has no total, so the overview adds a zero-size disabled card for
  it; the grid sorts by size, so it comes after every sized card. It sits among the cards, not under a meaning group.
- `ModuleScreen`: `shownOf` keeps an unavailable category; a module that ended `unavailable` with nothing to show
  says why in place of the "empty" state. `CategorySection`: a limited section's size has a focusable tooltip anchor;
  an unavailable one is a row with no checkbox or disclosure and `UnavailableReason`, which for an OS permission or an
  opt-in renders the 04/02 `CoverageNotice` (same action) and for a tool one sentence.
- Decision: `aria-disabled` is on the card's button only; a section (a region) does not support it, so it is described
  by its reason (`aria-describedby`) and greyed through the row's `disabled`.
- Decision: no gallery story of the module page (it reads the store); the card and the list have stories, the page is
  covered by `module-screen.test.tsx`.
- Tests: `category-card.test.tsx`, `meaning-list.test.tsx`, `module-screen.test.tsx`, `category-access.test.ts`
  (negative first); `src/screens src/product src/state src/i18n` green (635), eslint and `tsc` clean. Three new gallery
  baselines (`CategoryCard/limited`, `CategoryCard/unavailable`, `MeaningList/with-unavailable`) added and looked at.
  The gallery run reported differences in `PermissionCard/denied`, `PermissionCard/unknown` and
  `PlaceList/unreadable`, which this step does not touch.

### Fix round 1

- The core recomputes each category's availability from its current access on every `get_modules` call
  (`Engine::modules`), so the UI only had to ask again: `AppProviders` now lists the modules at start and again each
  time the access changes (one request at a time, one more after it when a change came meanwhile; a failed refresh keeps
  the last good list). Test: `app-providers.test.tsx`, after an `access-changed` event an unavailable category reads
  ready; a failed refresh keeps the list.
