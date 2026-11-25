# 042/04/02 — The screen

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** core (texts), app/ui, e2e, sandbox (demo)

Review as the mockup draws it: the summary card, the separator "What goes", one tile per category with its items, and
the floating bar. 039's two parts go. The design: [`../00-design.md`](../00-design.md), section 8. Mockup:
[`review.dc.html`](../../../mockups/0.1.0/review.dc.html).

## Read first

04/01's commit; `screens/review/` (all), `product/danger-zone/` (all), `product/{finding-row,finding-list}/`,
`product/confirm-dialog/` (unchanged; it stays the only confirmation), the pieces of plan 02, `crates/core/locales/
en.json` and `crates/core/src/texts.rs` (`core.danger.part.*`), `fixtures/demos/scenarios/cleanup.toml`,
`crates/sandbox/src/scenario/traced.rs`, `e2e/{flows,visual,axe,keyboard}.spec.ts` (Review's tests).

## Files

- `screens/review/review-screen.tsx` — `Toolbar` with the title `ui.screen.frame.nav.review` "Review" (13 px, 600);
  content gap 16 px, padding 20 px 36px 0:
  - **The summary card** (`section` labelled `ui.screen.review.summary`; padding 20 px 24 px, radius 18 px, border
    `--tile-border`, background `--surface-card`, column gap 14 px): a row with the size (30 px, 700, letter-spacing
    -0.5 px) and `ui.screen.review.goes` "goes to the Trash · {items}" (14 px, `--text-secondary`; `{items}` is
    `ui.screen.review.count`), and at its end `ui.screen.review.put-back` "You can put them back from Cleanups" (12 px,
    `--text-tertiary`); a bar (8 px, radius 4 px, gap 3 px, `role="img"`, `aria-label` `ui.screen.review.bar` "By
    group: {parts}") split by the size of each group present, coloured `--level-<level>-fg`; a chip per group present
    (12 px, `--text-legend`, background `--level-<level>-chip`, radius 8 px, padding 6 px 10 px, an 8 px swatch radius
    2 px, the group name `core.danger.<level>.zone` and its size bold); the Look chip adds
    `ui.screen.review.not-checked` "· {count} not checked" in `--level-medium-fg` while items are unticked. The
    summary sums what the store's `summarize` gives (as today).
  - `SectionSeparator` `ui.screen.review.separator` "What goes".
  - one `ReviewTile` per group of `reviewGroups`, in level order (Safe, Low, Medium, High), largest first within a
    level; no level headings.
  - **The bar** (the selection bar's look of 03/01, `aria-label` `ui.screen.review.bar-label`): the count
    (`--text-secondary`), the size (15 px, 600), `ui.screen.review.clear` "Clear" (ghost), and the primary
    `ui.screen.review.remove` "Move to the Trash" (inactive while nothing goes, 04/01; busy "Checking each item…"
    while the core checks). The confirmation, the agreement sheet and the refusal notice as today.
- `screens/review/review-tile.tsx` (new; replaces `review-group.tsx` and `review-part.tsx`) — an `article`
  `aria-labelledby` its name, on a `Tile` (`padding="none"`, tone the group's level; Look: `surface="medium"`):
  - header (padding 12 px 16 px, gap 12 px, centred): the all-checkbox `ui.screen.review.all` "All {name}" (15 px;
    checked when every item goes, mixed when some; ticking dispatches `tick-all`, unticking `untick-all`); the
    `small` `CategoryIconBadge`; a column with the name (`h2`, 15 px, 600) and the line (12 px): for Safe and Low
    `ui.screen.review.line` "{group} · {meaning}" (`--text-tertiary`), for a Look tile `ui.screen.review.check-each`
    "{group} · Tick each one you have checked" (`--tile-line-medium`); the pill (11 px, radius 10 px, padding 3 px 9 px)
    `ui.screen.review.checked` "{checked} of {count} checked" — all checked: a 11 px check icon before it,
    `--level-safe-fg` on `--level-safe-icon-bg`; not all: `--level-medium-fg` on `--level-medium-icon-bg`; the size
    (18 px, 700, 80 px wide, right-aligned).
  - the items (border-top 1 px `--gray-800` token, Look `--tile-divider-medium`; padding 4 px 0 8 px): the paged
    `FindingList` of the group's rows (at most 6 visible, then it scrolls), each row in the `compact` variant below.
- `product/finding-row/finding-row.tsx` — `variant?: 'default' | 'compact'`: `compact` drops the level badge; row
  padding 7 px 16 px 7 px 62 px, gap 12 px, a 15 px checkbox, the title 13 px over the short path (11 px, mono,
  `--text-tertiary`, `~` for the home), the size 13 px; an unticked row's title `--text-secondary` and size
  `--text-tertiary`; hover background `--surface-row-hover`. Its accessible name keeps the level (screen readers still
  hear it). `product/finding-list/` passes `variant`.
- Delete `screens/review/review-part.tsx`, `review-group.tsx`, `reviewZones`, `reviewParts`; `product/danger-zone/`'s
  `DangerZone` component and `PART_OF`, `PART_ORDER`, `DangerPart` (keep `ZONE_ORDER`, `GROUP_ICONS`, `BY_HAND`) and
  their stories, tests and baselines, if nothing else uses them; `knip` clean.
- `crates/core/locales/en.json` and `crates/core/src/texts.rs` — remove `core.danger.part.{safe,danger}.{name,meaning}`;
  UI fixtures that copy them follow.
- `app/ui/src/locales/en.json` — the `ui.screen.review.*` texts above; remove `ui.screen.review.{reaches,highest,
  subtitle}` and any other key the screen no longer reads.
- `fixtures/demos/scenarios/cleanup.toml` — the Review click becomes `{ role = "button", name = "Move to the Trash" }`
  (the confirmation's own "Move to Trash" stays the later click); the header comment updated.
- `e2e/` — `flows.spec.ts`: 039's "Review shows the safe part and the danger zone apart" and "Review names how far the
  selection reaches" go; new, negative first: "a Look tile shows how many of its items are not checked"; "the bar's
  button is inactive while nothing is ticked"; positive: "a tile's pill turns green when every item is checked";
  "unticking an item takes it out of the summary"; the High flows (offered mock) still tick the confirmation's High
  tick. `visual.spec.ts`: review with a selection, with a Look tile partly checked, with nothing selected (new
  baselines; the two-parts one removed); `axe`, `keyboard`.

## Tests (negatives first)

`review-tile.test.tsx`: negative "a Look tile's items start unticked"; "an untick-all keeps a Look category in the
list"; positive "the pill reads checked of count and turns green when all"; "the all-checkbox ticks the category
whole"; "a Safe tile's line names its group and its meaning". `review-screen.test.tsx`: negative "with nothing going
and nothing to check it shows the empty state"; positive "the summary's chips list each group with its size, Look with
its not checked count"; "the bar removes through the confirmation". `finding-row.test.tsx`: negative "a compact row
shows no level badge"; positive "its name still reads the level".

## Checks

`npx vitest run src/screens/review src/product`; test:cov:ui; lint:ts, lint:code, knip, the CSS guard;
`CARGO_TARGET_DIR=<target> cargo nextest run -p fresher-core -p fresher-sandbox` (keys, the demo trace);
`npm run check:texts`; `npx playwright test e2e/flows.spec.ts e2e/visual.spec.ts e2e/axe.spec.ts e2e/keyboard.spec.ts
--project webkit --project chromium` (look at every Review baseline against the mockup).

Commit: `feat(ui): Review with a summary card and a tile per category`.

## Conclusions

- Review: the summary card, the separator "What goes", a tile per category (`review-tile.tsx`), the bar ("Ready to
  remove"), Move to the Trash inactive while nothing is ticked. 039's two parts, `DangerZone` and
  `core.danger.part.*` are deleted. `FindingRow` variant `review` (ruling C-6).
- The `~` short path (fix round): `AppInfo.home`, the home the engine was given, and `shortPath(path, home)` at a
  component boundary; the full path stays in the tooltip and the accessible name.
- Picked tiles read "N picked" (08/02); a High tile uses the Look surface. The demo's `cleanup` scenario clicks "Move
  to the Trash".
