# 042/03/01 — The screen, its groups and tiles

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e, sandbox (demo trace)

Scan as the mockup draws it: a top bar, the disk card, the separator, one group per level with a heading and a line,
and two columns of tiles; the selection bar floats. The design: [`../00-design.md`](../00-design.md), section 7;
agent's calls 2, 4 and 19. Mockup: [`scan.dc.html`](../../../mockups/0.1.0/scan.dc.html).

## Read first

`screens/scan/{scan-screen.tsx,scan-zones.tsx,zones.ts,select-all-question.tsx,left-out-only.tsx,described-only.tsx,
phase.ts}` and their tests; `product/{category-card,category-grid,scan-hero,storage-map,selection-bar}/`;
`design/layout/toolbar/`; `app/ui/src/i18n/translate.ts` (the kinds of text arguments); `state/clock.ts`; the pieces
of plan 02; `e2e/{scan,flows,visual,axe,keyboard,zoom,layout}.spec.ts` and `e2e/support/cards.ts`;
`fixtures/demos/scenarios/{overview,cleanup}.toml` and `crates/sandbox/src/scenario/traced.rs`.

## Files

- `screens/scan/scan-screen.tsx` — top to bottom:
  - `Toolbar`: title `ui.screen.scan.title` "Scan" (13 px, 600); actions: `ui.screen.scan.scanned` "{when}" (12 px,
    `--text-tertiary`; the last scan's finish time as "2 minutes ago", agent's call 19) then the secondary button
    `ui.screen.scan.scan-again` "Scan again" after a finished or partial scan.
  - `DiskCard` for the first volume (`volumes[0]`); every other volume as a `compact` disk card under it.
  - the partial notice, the rescan notice, the coverage notices and the Trash lines as now (restyled only by tokens).
  - `SectionSeparator` `ui.screen.scan.separator` "What you can remove" after a scan.
  - `ScanGroups` (below), then `LeftOutOnly`, `DescribedOnly`, the ignored line, as now.
  - The `SelectionBar`.
  Main padding 20 px 36 px 0, gap 18 px.
- `app/ui/src/i18n/` — a text argument `{ kind: 'ago'; value: Timestamp }` formatted with `Intl.RelativeTimeFormat`
  ("now", "2 minutes ago", "3 hours ago", "yesterday", then days) against the clock the UI already has
  (`state/clock.ts`), refreshed each minute while shown; tests with a fixed clock.
- `screens/scan/scan-groups.tsx` (replaces `scan-zones.tsx`; `zones.ts` keeps `zonesOf` and `zoneSelectAll`) — one
  `section` per non-empty level in `ZONE_ORDER`, `aria-labelledby` its heading; gap 10 px:
  - heading row (flex, centred, gap 8 px): `GROUP_ICONS[level]` 16 px, stroke 2.2, `--level-<level>-fg`; `h2` 14 px
    600 `core.danger.<level>.zone`; the line 12 px `--text-tertiary` "· " + `core.danger.<level>.meaning`
    (`ui.screen.scan.group-line` "· {line}"); for Safe and Low only, at the row's end, a small checkbox
    `ui.danger-zone.select-all` "Select all" with its description, as today's zone (Medium and High have none:
    agent's call 4).
  - a `CategoryGrid` of `CategoryTile`s (two columns, gap 12 px).
  `SelectAllQuestion` and its texts `ui.danger-zone.ask.*` are deleted (no group that asks remains).
- `product/category-tile/category-tile.tsx` (replaces `category-card`; css, stories, test) — on a `Tile` of the
  category's level:
  - top row (flex, gap 12 px, start): `CategoryIconBadge` (the category's icon by `iconOf`), the name (15 px, 600,
    `<category>.name`, an `h3`) over the line (12 px, `--text-tertiary`, `<category>.line`), the checkbox (16 px)
    `ui.category-card.select` "Select {name}" (key and text kept: the demo's target); limited or unavailable access
    text under the line as today.
  - bottom row (baseline, gap 8 px): the size (24 px, 700, honest), the count (12 px, `--text-tertiary`,
    `ui.category-card.count`), a spacer, `ui.category-card.details` "Details" (13 px, `--accent-text`, a ghost
    button), the `⋯` menu (28 px, `ui.category-card.options`), with the menu items as today.
  - no level badge, no share bar, no spotlight. `data-card`, `data-level`, `data-selected` kept (the e2e support finds
    cards by them).
- `product/category-grid/` — renders tiles; its two columns as the mockup.
- `product/selection-bar/` — floats as the mockup: absolute, bottom 18 px, centred; background `--surface-floating`,
  1 px border `--gray-700` token (`--border-default` if that is the one), radius 12 px, padding 8 px 8 px 8 px 18 px,
  shadow `0 10px 30px var(--black-a55)`, 13 px; "{n} selected" `--text-secondary`, the size 15 px 600, Clear (ghost),
  Review (primary).
- `screens/scan/left-out-only.tsx`, `described-only.tsx` — each a section with the group heading's look (h2 14 px 600
  and its line) over tiles, no level badge.
- Delete what nothing uses any more: `product/scan-hero/` (`ScanHero`, `ring-parts`, `ring-legend`, stories, tests),
  `product/category-card/`, `select-all-question.tsx`, the other volumes' `StorageMap` use on Scan (keep
  `product/storage-map/` if another screen imports it), their baselines; `knip` must stay clean.
- `app/ui/src/locales/en.json` — `ui.screen.scan.scanned`, `ui.screen.scan.separator`, `ui.screen.scan.group-line`;
  the deleted keys removed.
- `e2e/` — `scan.spec.ts`, `flows.spec.ts` (039's "Select all in the danger zone asks first…" tests go: replaced by
  negative "a Look before removing group has no Select all" and positive "Select all of Safe to remove ticks its
  tiles"), `visual.spec.ts` (scan: before a scan, finished, partial, failed, the banners, Full Disk Access denied; the
  question's test removed), `axe`, `keyboard`, `zoom`, `layout`, `contrast` as they cover Scan; `support/cards.ts` if
  its selectors change.
- `fixtures/demos/scenarios/*.toml` — targets still resolve ("Select Xcode build data", "Review", "Scan"); if one
  changed, update it and its trace.

## Tests (negatives first)

`scan-groups.test.tsx`: negative "an empty level has no group"; "Look before removing has no Select all"; positive
"groups come Safe, Usually safe, Look before removing, each with its core name and line"; "Select all of a group
selects its tiles". `category-tile.test.tsx`: negative "an unavailable category has no checkbox"; "a tile shows no level
badge"; positive "shows the icon, name, line, size, count, Details and the menu"; "the checkbox selects the
category". `scan-screen.test.tsx`: negative "before a scan there is no separator and no group"; positive "the disk card
then the separator then the groups"; "Scan again shows after a finished scan with when it was scanned". The i18n `ago`
tests: negative "a time in the future reads now"; positive minutes, hours, yesterday, days.

## Checks

`npx vitest run src/screens/scan src/product src/i18n`; `npm run test:cov:ui` (90 %); lint:ts, lint:code, knip, the
CSS guard; `npx playwright test e2e/scan.spec.ts e2e/flows.spec.ts e2e/visual.spec.ts e2e/axe.spec.ts
e2e/keyboard.spec.ts e2e/zoom.spec.ts e2e/layout.spec.ts --project webkit --project chromium` (update the Scan
baselines; look at each against the mockup); `CARGO_TARGET_DIR=<target> cargo nextest run -p fresher-sandbox` (the
demo trace).

Commit: `feat(ui): Scan with the disk card, groups and tiles`.

## Conclusions

- Scan rebuilt: the toolbar with "2 minutes ago" (`i18n/relative-time.ts` and an injected wall clock, ruling C-5),
  the disk card, compact cards for other volumes, the separator, `scan-groups.tsx`, `CategoryTile` in a two-column
  grid, the floating selection bar. Deleted: the cards, the danger zone's question, the storage map block.
- The selection bar's level badge is gone on every screen (accepted: the mockup has none; the confirmation names the
  level). The group's Select all description sits in a tooltip (accepted). The side inset stays 32 px (mockup 36).
- The e2e mock rewrites `lastScan.finished` so every baseline reads "2 minutes ago" on any day.
