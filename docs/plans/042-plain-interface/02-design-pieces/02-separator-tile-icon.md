# 042/02/02 — Separator, tile and icon badge

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

Three small pieces: the section separator ("What you can remove"), the tile every category is drawn on, and a
category's icon in a tinted square. Mockups: [`scan.dc.html`](../../../mockups/0.1.0/scan.dc.html) (separator,
`.tile`, `.ico`), [`review.dc.html`](../../../mockups/0.1.0/review.dc.html) (the smaller badge, the Medium tile).

## Read first

`app/ui/src/design/layout/surface/` (the model of a layout piece), `design/icons/{icons.ts,icon.tsx}`,
`gallery/registry.ts`, `product/danger-zone/zones.ts` (`LEVEL_ICONS`), the tokens of 02/01, 01/04's `CategoryIcon`.

## Files

- `design/layout/section-separator/section-separator.tsx` (+ `.module.css`, `.stories.tsx`, `.test.tsx`) —
  `SectionSeparator({ label }: { readonly label: string })`: a `div` (flex, centred, gap 14 px, padding-top 4 px)
  holding the label (12 px, weight 600, `--text-secondary`, uppercase, letter-spacing 0.8 px, no wrap) and an
  `aria-hidden` line (flex 1, 1 px, `--separator-line`). The label is plain text in the reading order; no ARIA role.
- `design/layout/tile/tile.tsx` (+ css, stories, test) — `Tile({ children, tone, surface = 'plain', as = 'div' })`,
  `tone: Level` (the danger badge's `Level`), `surface: 'plain' | 'medium'`: column flex, gap 12 px, padding 16 px,
  radius 14 px (`--radius-lg`), background `--surface-tile` (`--surface-tile-medium` for `medium`), 1 px border
  `--tile-border-<tone>`, `box-shadow: 0 1px 0 var(--white-a4) inset`; hover border `--tile-border-hover`. Takes
  `data-level` for tests. A `padding="none"` option for Review's tile (its header and list have their own).
- `design/icons/icons.ts` — export the lucide shapes used: `Layers, Hammer, Download, Folder, Code, FileText, Clock,
  AppWindow, Cpu, SquareCode, Globe, Smartphone, Archive, File, ShieldCheck, Shield, Eye, ShieldAlert, Info, Search`
  (those not already exported).
- `product/category-icon/category-icon.tsx` (+ css, stories, test) — `CATEGORY_ICONS: Readonly<Record<CategoryIcon,
  IconComponent>>`: cache Layers, build Hammer, download Download, project Folder, toolchain Code, log FileText,
  temporary Clock, app AppWindow, model Cpu, editor SquareCode, browser Globe, device Smartphone, archive Archive,
  generic File. `CategoryIconBadge({ icon, level, size = 'large' })`: a square, `large` 38 px radius 10 px with a
  20 px icon, `small` 34 px radius 9 px with an 18 px icon; background `--level-<level>-icon-bg`; stroke
  `--level-<level>-fg`, stroke width 2; `aria-hidden`.
- `product/category-icon/icon-of.ts` — `iconOf(modules: readonly ModuleInfo[] | null, category: CategoryId):
  CategoryIcon`: the descriptor's icon of the module that lists it, `'generic'` for a category no listed module has
  (the core's own categories). A lookup, no judgement.
- `product/danger-zone/zones.ts` — `GROUP_ICONS: Readonly<Record<DangerLevel, IconComponent>>`: safe ShieldCheck, low
  Shield, medium Eye, high ShieldAlert (the group headings of plan 03; `LEVEL_ICONS` stays until its users go).
- `gallery/registry.ts` — the three stories.

## Tests (negatives first)

`section-separator.test.tsx`: negative "the line is hidden from assistive technology"; positive "shows its label".
`tile.test.tsx`: negative "a plain tile has no medium surface"; positive "takes its tone's border", "a medium tile
takes the medium surface". `category-icon.test.tsx`: negative "a badge is hidden from assistive technology";
positive "every icon key has a shape" (iterate the generated `CategoryIcon` union through a fixed list kept equal to
the core's, which the bindings test guards), "a small badge is smaller". `icon-of.test.ts`: negative "a category no
module lists is generic"; positive "a module's category gives its icon".

## Checks

`npx vitest run src/design/layout src/product/category-icon src/product/danger-zone`; `npm run lint:ts`, `npm run
knip` (`LEVEL_ICONS` still used), lint:code; `npx playwright test e2e/gallery.spec.ts --project gallery
--update-snapshots` for the new stories (look at each).

Commit: `feat(ui): the section separator, the tile and the category icon badge`.

## Conclusions

- `SectionSeparator` (no ARIA role), `Tile` (`as` div, li, section, later article), `CategoryIconBadge`, `iconOf`
  reading the core's module descriptors and `GROUP_ICONS`; the existing `IconShape` type reused (ruling D-7).
