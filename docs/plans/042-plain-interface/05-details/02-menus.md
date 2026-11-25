# 042/05/02 — The menus

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

One menu on every row and tile, in one order, with separators; the pointed item filled with the accent colour (02/01);
items carry no hint (user's decision 2026-11-25, reversing agent's call 16). The design: [`../00-design.md`](../00-design.md), section 9; agent's call 16.
Mockup: [`row-menu.dc.html`](../../../mockups/0.1.0/row-menu.dc.html).

## Read first

`design/primitives/menu/{menu.tsx,menu.module.css}` (`MenuItem`), `design/primitives/tooltip/`, where the menus are
built: `screens/scan/scan-groups.tsx` (the tile's), `screens/category/category-findings.tsx` and `screens/review/`
(the rows'), `screens/settings/use-ignore-commands.ts`, the `ui.category-card.*` and the row menu's texts,
`e2e/support/ignore-mock.ts`, the menu flows of `e2e/flows.spec.ts` (plans 028, 015).

## Files

- `design/primitives/menu/menu.tsx` — `MenuItem` gains a separator entry (`{ kind: 'separator' }`, a 1 px line
  `--border-default` inset 4 px 6 px, `role="separator"`, skipped by the arrow keys); an item has no description and no
  hint (removed 2026-11-25). Item padding 7 px
  10 px, radius 6 px, 13 px; the menu 250 px wide, padding 5 px, radius 10 px, border `--border-default` token equal to
  `#313741`, shadow `0 12px 32px var(--black-a55)`.
- The tile's menu: `ui.category-card.ignore-category` "Ignore all {name}" only. The "Stop looking for {module}…" item
  was removed (user's decision 2026-11-25): a module is turned off only in Settings, Ignored, Not looked for.
- A row's menu (details and Review): `ui.finding-list.menu.reveal` "Show in Finder" (if the row has one today, else
  the reveal command it already offers), a separator, "Ignore this item", "Ignore all {category}". Keys keep their names where they exist; new texts take keys beside them.
- Items carry no description or hint (user's decision 2026-11-25).
- `e2e/` — the menu flows (ignore an item, a category, a module), `visual.spec.ts` / `gallery.spec.ts` baselines of an
  open menu, `keyboard.spec.ts` (separators skipped), `axe`.

## Tests (negatives first)

`menu.test.tsx`: negative "the arrow keys skip a separator"; "a separator is not an action"; positive "pointing at an item shows
no hint"; "the pointed item is marked active". Tile and row tests: positive "the menu lists its items
in order with its separators".

## Checks

`npx vitest run src/design/primitives/menu src/product src/screens`; lint:ts, lint:code, knip; `npm run check:texts`;
`npx playwright test e2e/flows.spec.ts e2e/keyboard.spec.ts e2e/visual.spec.ts e2e/gallery.spec.ts --project gallery
--project webkit --project chromium` (look at every changed menu baseline against the mockup).

Commit: `feat(ui): one menu on every row and tile`.

## Conclusions

- One menu order with a separator on tiles, the details' rows and Review's rows (`screens/use-row-menu.ts`); the
  pointed item fills with the accent; a description is the item's hint, after the hover delay (fix round: hovering no
  longer focused the item and showed it at once). A tool's object has only the category and module items.
