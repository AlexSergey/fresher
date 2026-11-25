# 042/05/01 — The details screen

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

The details of one category: back to Scan, its name and group, its size and count, three answers, what was left out,
the module's choices and the items. The design: [`../00-design.md`](../00-design.md), section 9; agent's call 15.
Mockup: [`category-details.dc.html`](../../../mockups/0.1.0/category-details.dc.html).

## Read first

`screens/category/{category-screen.tsx,category-about.tsx,category-findings.tsx,module-choices.tsx,
unavailable-reason.tsx,category-screen.module.css}` and their tests, `product/left-out-lines/`,
`product/finding-row/` (04/02's `compact`), 04/01 (`BY_HAND`: no "Select all" for a Look category),
`e2e/visual.spec.ts` ("a category's details"), `e2e/flows.spec.ts` (details flows).

## Files

- `screens/category/category-screen.tsx` — main padding 24 px 44 px 0, gap 20 px:
  - `Toolbar` with only the back button `ui.screen.category.back` (text "Scan", a `ChevronLeft`, 13 px,
    `--text-secondary`); the Scan and Stop buttons go (agent's call 15).
  - header (flex, end, space-between): a column (gap 6 px) of the `h1` name (24 px, 600) and the group (an 8 px dot
    `--level-<level>-fg`, then `core.danger.<level>.zone`, 13 px, `--text-secondary`); at the end the size (28 px,
    700) and the count (13 px, `--text-tertiary`, `ui.screen.category.count`). The danger badge section goes.
  - the unavailable reason, as now.
  - `CategoryAbout`: a grid of three columns, gap 12 px; each block background `--surface-raised`, 1 px border
    `--border-subtle` (the mockup's `#1f232a`: use the token equal to `--gray-800`), radius 10 px, padding 14 px,
    column gap 6 px; its title 12 px `--text-tertiary`, its text 13 px, line height 1.45. Blocks:
    `ui.screen.category.what` "What it is" (`<category>.what`, then `<category>.made-by` under it, 12 px,
    `--text-tertiary`), `ui.screen.category.if-removed` "If you remove it" (`<category>.if-removed`),
    `ui.screen.category.comes-back` "Comes back?" (`<category>.comes-back`). The old heading "About" and the list go.
  - each left-out line as a bar (flex, centred, gap 10 px, 13 px `--text-secondary`, background `--surface-raised`,
    the same border, radius 10 px, padding 10 px 14 px) led by a 14 px `Info` icon in `--text-tertiary`
    (`product/left-out-lines/` gains a `bar` look, or the screen wraps it).
  - `ModuleChoices`, as now.
  - the items: for a Safe or Low category a checkbox `ui.screen.category.select-all` "Select all {count}" (13 px,
    `--text-secondary`, padding 0 14 px); none for a Look category. A frame (background `--surface-raised`, the
    border, radius 10 px) of rows in the `compact` row, sized as the mockup: padding 10 px 14 px, gap 14 px, the title
    14 px 500, the path 12 px mono `--text-tertiary`, the size 14 px, the `⋯` 28 px; 1 px dividers; it scrolls past 12
    rows as now.
  - The selection bar, as on Scan.
- `app/ui/src/locales/en.json` — the texts above (`ui.screen.category.select-all`, the new `what`/`if-removed`/
  `comes-back` labels); remove `ui.screen.category.{about,made-by,level,select}` if unused; `scripts/check-texts.ts`'s
  `EXEMPT` keeps the three labels (they are not category texts).
- `e2e/` — `visual.spec.ts` details baselines (every test of "a category's details"), `flows.spec.ts` (select from
  details; a Look category has no Select all), `axe`.

## Tests (negatives first)

`category-screen.test.tsx`: negative "a Look before removing category has no Select all"; "the details have no Scan
button"; "an unavailable category shows its reason and no items"; positive "shows the name, group, size and count";
"the three answers, with who makes it under What it is"; "each left-out line is an i bar"; "Select all selects a Safe
category whole".

## Checks

`npx vitest run src/screens/category src/product/left-out-lines`; test:cov:ui; lint:ts, lint:code, knip;
`npm run check:texts`; `npx playwright test e2e/visual.spec.ts e2e/flows.spec.ts e2e/axe.spec.ts --project webkit
--project chromium` (look at each details baseline against the mockup).

Commit: `feat(ui): the category details answer three questions`.

## Conclusions

- The details: Back "Scan" (named "Back to Scan"), the name, the group, size and count, the three answers with who
  makes it, the left-out "i" bars, the module's choices, the items (`FindingRow` variant `details`), "Select all N"
  for Safe and Usually safe. The title is 22 px (no 24 px token) and the menu button 24 px (the primitive's size).
