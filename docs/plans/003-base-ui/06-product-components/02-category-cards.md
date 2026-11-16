# 003/06/02 — Category cards

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `product/category-card/` — `CategoryCard`: props `total: CategoryTotal`, `share: number` (of `found`), `selected:
  boolean | 'mixed'`, `onToggle`, `onOpen`, `actions?` (a slot: the ignore list's "Ignore" will go here later).
  - A `Surface` (elevation 1) with the spotlight: the level's icon in a 32 px tinted circle with its soft glow, the
    category's name (`<category>.name`, `--type-callout`) and one line of what it is (`<category>.what`, secondary,
    truncated with its full text in a tooltip), the `DangerBadge` (small), the size (`SizeText`, `--type-title-3`) and
    the count; along the bottom edge a 2 px share bar in the level's colour, `transform: scaleX(share)` from the left.
  - The whole card opens the category (`onOpen`, a `<button>` covering the card); the `Checkbox` in its corner selects
    it (`onToggle`), and is a separate focus stop.
  - Hover: border to `--border-default`, spotlight, 1 px lift; press: `scale(0.99)`; selected: an Iris border and a
    faint `--glow-accent`.
- `product/category-grid/` — `CategoryGrid`: a responsive grid (container queries: 1, 2 or 3 columns) of cards
  sorted by allocated size, largest first; new cards enter with the stagger; a change of order plays `useFlipList`
  (measured: the grid is small); during a scan reorders are coalesced to at most one per `--duration-move`, so cards
  never chase each other.
- `product/module-section/` — `ModuleSection`: a module's heading (its name, its found size, its outcome: done,
  scanning with a small spinner, failed with the core's reason in a `Notice`) above its content.
- `locales/en.json` — `ui.category-card.*` (the open and select labels with the category's name, the count's plural).

## Produces

The overview's category grid (07/02) and the module page's sections (07/03).

## Checks

- Negative: `a card never shows its level by colour alone` (icon and word present), `the checkbox does not open the
  category`, `a zero share draws no bar`, `cards do not reorder more than once per move duration during a scan`,
  `motion off reorders without animation`.
- Positive: `opens on click and Enter`, `selects with the checkbox and Space`, `cards are sorted largest first`, `a
  card that grows past another glides to its place` (the WAAPI stub records the inverted translation), `the module
  section shows a failed module's reason`.
- Stories: a card per level, partial size, selected, mixed, hover (spotlight at centre); the grid with the demo totals
  at 1, 2 and 3 columns; module section finished, scanning, failed; screenshots looked at.

## Conclusions

- Built: `CategoryCard`, `CategoryGrid` and `ModuleSection`, with unit tests (negative first), stories (a card per
  level, partial size, selected, mixed, hover; the grid at 1, 2 and 3 columns; the section finished, scanning,
  failed) and WebKit screenshots, which were looked at. No new dependency, token or icon.
- The card is a `Surface` with the spotlight inside a plain wrapper that carries the level, the selection and the
  hover lift; the open `<button>` holds the card's content (so a click anywhere but the checkbox opens it) and is named
  "Open {name}", described by its content (what it is, the level's word, the size, the count). The checkbox ("Select
  {name}") is a sibling in the corner, so it never opens the card. The level shows as an icon escalating in shape and
  as the `DangerBadge`'s word, never by colour alone. The focus ring is drawn inside the card, since the spotlight's
  surface clips what overflows it.
- The share bar is drawn only above 0, clamped to 1, by `transform: scaleX(var(--share))`. The size is the core's
  allocated size through `product/sizes.ts` (a lower bound with its plus, a dash when unknown).
- The grid sorts by allocated size, largest first (ties keep the core's order), in a container-query grid (two columns
  from 560 px, three from 840 px). `use-coalesced-order.ts` holds the shown order during a scan: a new order is shown
  at most once per `--duration-move` (320 ms); in between, cards keep their places, a new card joins at the end and a
  gone one leaves. At rest a new order is shown at once. Moves are played by `useFlipList`; new cards enter staggered
  by their place (`--i`, CSS only).
- Agent's calls: the grid takes `found` (each share is a card's allocated size out of `found.allocated`), `scanning`,
  `selected(category)`, `onOpen(category)` and `onToggle(category)`; `onToggle` of a card takes no argument (the store
  toggles the category). `ModuleSection` takes `module`, `found` and `state` (`scanning` or the core's
  `ModuleOutcome`): its heading is the module's name (`<module>.enabled.label`), the size and the status (done,
  scanning with a small spinner, stopped, or failed with the core's reason in a danger `Notice`, not live: the hero
  speaks the scan's end). Texts are `ui.category-card.*`, `ui.category-grid.*` and `ui.module-section.*`.
- The card's `actions` slot is drawn in its bottom-right corner; nothing fills it yet (the ignore list will).
