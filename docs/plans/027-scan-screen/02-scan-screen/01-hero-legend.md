# 027/02/01 — The hero's ring and legend

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** app/ui, e2e

The hero draws the home volume as one ring, the cleanable part split by danger level, Free as the gap, with a legend
beside it whose "i" explains each part, and ring and legend highlight each other (feedback items 3 to 6 and 13). The
stat tiles, the hero's usage bar and the home volume's map block go. The design:
[`../00-design.md`](../00-design.md), sections 2.1 to 2.4 (agent's calls 3, 4, 13 and 16; decisions 2, 3, 4, 8).

## Files

- `app/ui/src/product/sizes.ts` — `addSizes(a, b)`: bytes added, the weakest completeness wins (moved from
  `meaning-list.tsx`'s `plus`, which then imports it until 02/03 deletes that file).
- `app/ui/src/product/scan-hero/ring-parts.ts` (new) — `levelShares`, `RingPart`, `RingAbout`, `ringParts`,
  `ringSegments` exactly as in the design, section 2.2. Labels: a level's `core.danger.<level>.zone`, a part's
  `partName(kind)`. Tones: a level's own; a part's `partTone`. Keys: `level-<level>`, `<kind>-<index in the
  volume's parts>`.
- `app/ui/src/product/scan-hero/ring-legend.tsx` and `ring-legend.module.css` (new) —
  `RingLegend { parts, highlighted, onHighlight, purgeable, scanning }`: a list (`ul`, labelled
  `ui.scan-hero.legend` "Parts of the disk") of rows as in the design, section 2.4: swatch (`data-tone`, Free an
  outline, `unknown` hatched as the usage bar's), label, size, the "i" `IconButton` (`Info`,
  `ui.scan-hero.about` "What is {name}?", `aria-expanded`, `aria-controls`) and the explanation region.
  `onPointerEnter`/`onPointerLeave` and `onFocus`/`onBlur` (focus within the row) call `onHighlight`.
- `app/ui/src/product/storage-map/part-reasons.tsx` (new) — the remainder's reasons list (`Reason` of
  `part-line.tsx`, with the unreadable folders' count and the Full Disk Access line), used by `part-line.tsx` and the
  legend.
- `app/ui/src/product/scan-hero/scan-hero.tsx` and `scan-hero.module.css` —
  - props: `volume: null | VolumeMap` and `levels: ReadonlyMap<DangerLevel, Sizes>` added; the rest unchanged;
  - the caption above the layout (`ui.scan-hero.caption` "Space on {name}, {kind} disk"), absent without a volume;
  - the ring: `segments = ringSegments(parts)`, `total` = their sum, `gapLabel = core.storage.part.free.name`,
    `highlighted` and `onHighlight` from a `useState` of the hero, `label` `ui.scan-hero.ring` "Space on {name}"
    (or "Space on the disk" without a volume — `ui.scan-hero.ring-unknown`); state as today (`empty` before a scan
    and after a failure);
  - the side: the legend (not before a scan or after a failure), the disk-changed note under it when `diskChanged`,
    the scan's progress under it while scanning, the error state after a failure as today. `StatTile`, `UsageBar`,
    `Count` and the `.stats` CSS go; the centre's `Found` stays.
- `app/ui/src/product/scan-hero/story-data.ts`, `scan-hero.stories.tsx` — a volume and levels for every state; a
  `legend-open` state (one "i" open) and a `highlighted` state.
- `app/ui/src/screens/overview/overview-screen.tsx` — passes `volume={volumes[0] ?? null}` and
  `levels={levelShares(meaning.groups, totals)}`; the map blocks start at the second volume
  (`volumes.slice(1)`, all collapsible). `useStorageMeaning(0)` is asked as today.
- `app/ui/src/design/data/stat-tile/` — deleted (component, CSS, story) with its baselines
  (`app/ui/e2e/screenshots/gallery.spec.ts/StatTile/`).
- `app/ui/src/locales/en.json` — new `ui.scan-hero.caption`, `ui.scan-hero.legend`, `ui.scan-hero.about`,
  `ui.scan-hero.ring-unknown`; changed `ui.scan-hero.ring`; removed `ui.scan-hero.stat.*`, `ui.scan-hero.usage`.
- `app/ui/src/product/storage-map/core-keys.test.ts` — the changed your-folders description; the zone keys of every
  level exist in the core's file.

## Produces

`ScanHero`'s `volume` and `levels` props; `levelShares`, `ringParts`; `RingLegend`; `addSizes`; `PartReasons`.

## Checks

- `ring-parts.test.ts` (new), negatives first:
  - `describe('negative cases')`: `it('leaves out a share whose category the totals do not know')`; `it('draws no
    level with nothing in it')`; `it('drops a part of zero size')`; `it('falls back to the totals while the groups
    hold only parts')`.
  - `describe('positive cases')`: `it('sums the category shares and the apps' category shares by the totals'
    level')`; `it('keeps the weakest completeness')`; `it('puts the levels first, Safe to High, then the parts in
    the core's order, Free last')`; `it('names a level by its zone and a part by the core')`.
- `ring-legend.test.tsx` (new), negatives first: no explanation until its "i" is pressed; no row highlighted while
  nothing is; Free has no level text; then: the "i" shows the core's description (a level: Explained's description
  and its meaning; the remainder: its reasons; Free: the purgeable note), pointer and focus on a row call
  `onHighlight`, a highlighted key marks its row.
- `scan-hero.test.tsx` updated: negatives first (no legend before a scan or after a failure; no caption without a
  volume), then the caption, the legend's rows, and a hover on an arc marking its legend row (through
  `fireEvent.pointerEnter`).
- `npm run test --workspace app/ui -- src/product/scan-hero src/product/storage-map src/screens/overview`
- `npm run lint:ts`; `npm run knip`; `npx eslint` and `npx prettier --check` on the changed files.
- E2E: in `app/ui/e2e/flows.spec.ts` (or a new `scan.spec.ts`), negatives first: `the legend shows no explanation
  until its i is pressed`; positives: `hovering a ring segment highlights its legend row, and the reverse`, `the i of
  Not measured lists the core's reasons`. Then `npm run test:e2e:update --workspace app/ui -- gallery.spec.ts
  visual.spec.ts contrast.spec.ts zoom.spec.ts` (look at every changed PNG) and
  `npm run test:e2e --workspace app/ui` (all three projects green).

## Conclusions

Not written yet.
