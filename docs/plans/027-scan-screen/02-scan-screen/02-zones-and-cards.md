# 027/02/02 — Danger zones and cards

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** app/ui, e2e

The danger zone component Scan and Review share, and the card as the feedback asks: still on hover, a Details button,
a settings menu with Ignore, the whole category's size (feedback items 7 to 10 and 16). The overview still shows the
by-meaning list until 02/03; its cards take the new props here. The design: [`../00-design.md`](../00-design.md),
sections 3 and 4 (agent's calls 7 and 9; decision 1).

## Files

- `app/ui/src/product/danger-zone/zones.ts` (new) — `ZONE_ORDER` and `LEVEL_ICONS` (`safe` `ShieldCheck`, `low`
  `Shield`, `medium` `TriangleAlert`, `high` `OctagonAlert`), moved from the card's `ICONS`.
- `app/ui/src/product/danger-zone/danger-zone.tsx`, `danger-zone.module.css`, `danger-zone.stories.tsx` (new) —
  `DangerZone` as in the design, section 3: `section` labelled by its `h2` (`core.danger.<level>.zone`), the
  warning icon for Medium and High only (`aria-hidden`, the level's `--level-<level>-fg`), the meaning
  (`core.danger.<level>.meaning`), the optional `Checkbox` "Select all" (`ui.danger-zone.select-all`) with its
  description (`ui.danger-zone.select-all.description`) and `aria-describedby` the heading (add an optional
  `aria-describedby` to `Checkbox` if it has none); a 1 px rule in the level's colour above the head; children
  below. Stories: one per level with a card inside, `select-all-mixed`, `without-select-all`.
- `app/ui/src/product/category-card/category-card.tsx`, `category-card.module.css`, `category-card.stories.tsx` —
  as in the design, section 4: the body static (the `open` button and `ui.category-card.open` go; the name an
  `h3`), props `access`, `menu`, `onDetails`, `onToggle`, `selected`, `share`, `total` (`onOpen`, `actions`,
  `underApps` go), the actions row (Details `Button` small ghost described by the name's id; `Menu` with
  `Settings`, label `ui.category-card.options`), the checkbox as before; CSS: the hover `translateY` and the press
  scale removed, hover changes the border colour only. Stories: `rest`, `hover`, `selected`, `mixed`, `limited`,
  `unavailable`, `menu-open` (the `Menu`'s gallery-only `defaultOpen`).
- `app/ui/src/product/category-grid/category-grid.tsx` (+ stories) — props `access`, `found`, `label`,
  `moduleOf`, `onDetails`, `onIgnore`, `onToggle`, `scanning`, `selected`, `totals` (`onOpen` and `underApps` go);
  builds each card's `menu`: `{ key: 'ignore-category', label: ui.category-card.ignore-category, description:
  ui.category-card.ignore-category.description, onSelect: onIgnore({ kind: 'category', category }) }` and, when
  `moduleOf(category)` is known, `{ key: 'ignore-module', label: ui.category-card.ignore-module {module},
  description: core.ignore.modules.description, onSelect: onIgnore({ kind: 'module', module: id }) }`. The list's
  `aria-label` is `label`.
- `app/ui/src/product/meaning-list/meaning-list.tsx` — the grid's new props passed through (`onDetails` = its
  `onOpen`, `label` = `ui.category-grid.label`, `moduleOf` and `onIgnore` new props of the list); the cards' sizes
  are the totals' (the `cardsOf` share-outside-apps logic goes, `underApps` goes). The file is deleted in 02/03.
- `app/ui/src/screens/overview/overview-screen.tsx` — passes `moduleOf` (from `useModules()`, with the module's
  `<module>.enabled.label`) and `onIgnore` (`useIgnoreCommands().ignore`); `onDetails` keeps opening the module page
  at the category until 03/01.
- `app/ui/src/screens/use-modules.ts` — `moduleOfCategory(modules, category): ModuleInfo | undefined` (the lookup
  the overview's `open` does today), with its test.
- `app/ui/src/locales/en.json` — new `ui.danger-zone.select-all`, `ui.danger-zone.select-all.description`,
  `ui.category-card.details`, `ui.category-card.options`, `ui.category-card.ignore-category`,
  `ui.category-card.ignore-category.description`, `ui.category-card.ignore-module`; removed
  `ui.category-card.open`, `ui.category-card.under-apps`.
- Baselines: `CategoryCard/*`, `CategoryGrid/*`, `MeaningList/*`, `DangerZone/*` and the overview screens in
  `visual.spec.ts` and `contrast.spec.ts` regenerated.

## Produces

`DangerZone`, `ZONE_ORDER`, `LEVEL_ICONS`; the card's and the grid's new props; `moduleOfCategory` — for 02/03, 03/01
and 03/03.

## Checks

- `danger-zone.test.tsx` (new), negatives first: `it('shows no warning icon for Safe and Low')`; `it('shows no
  Select all without its prop')`; then `it('names the zone by the core and says what the level means')`, `it('shows
  the warning icon for Medium and High')`, `it('Select all reflects mixed and calls its handler')`.
- `category-card.test.tsx` updated, negatives first: `it('has no button for its body')`; `it('does not move on
  hover')` (no `transform` in the computed hover style, or the rule gone from the CSS module: assert the class has
  no `translateY`); `it('an unavailable card has no checkbox but keeps Details and the menu')`; then `it('Details
  calls onDetails')`, `it('the menu offers Ignore this category and Ignore <module>, each with its sentence')`,
  `it('shows the whole category's size')`.
- `category-grid` test (new or updated): no module item when `moduleOf` answers `null`; each item calls `onIgnore`
  with its target.
- `npm run test --workspace app/ui -- src/product/danger-zone src/product/category-card src/product/category-grid src/product/meaning-list src/screens`
- `npm run lint:ts`; `npm run knip`; `npx eslint` and `npx prettier --check` on the changed files.
- E2E: `flows.spec.ts`, negatives first `the card's menu closes on Escape and ignores nothing`; positives `Ignore this
  category from a card's menu takes it out of every total` (the ignore mock records the category), `the card's menu
  works by keyboard alone`. Specs that clicked a card to open a module page click its Details instead.
  `npm run test:e2e:update --workspace app/ui -- gallery.spec.ts visual.spec.ts contrast.spec.ts` (look at each
  changed PNG), then `npm run test:e2e --workspace app/ui`.

## Conclusions

Not written yet.
