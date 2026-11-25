# 042/02/03 — The disk card

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

The compact card of one volume: a ring of the core's five parts with "can be freed" in its centre, the disk's name and
free space, and a two-column legend, each row with its "i". Mockup: [`scan.dc.html`](../../../mockups/0.1.0/scan.dc.html),
the section "Your disk".

## Read first

`app/ui/src/design/data/storage-ring/{storage-ring.tsx,arcs.ts}` (segments, gaps, minimum arc, the highlight),
`design/data/segment.ts` (`Tone`), `product/scan-hero/{scan-hero.tsx,ring-legend.tsx}` (the legend's "i" and the
hover link of plan 027, to reuse, not copy), `product/sizes.ts` (`honestSize`, `sizeLabel`), 01/03's `SummaryPart`,
the tokens of 02/01.

## Files

- `design/data/segment.ts` — tones `can-be-freed`, `files-and-apps`, `mac-os`, `system-space` (fills
  `--part-<tone>`); Free stays the gap (`--tone-free-outline`).
- `design/data/storage-ring/` — props for this card if missing: `diameter` (180 default for the card, 96 compact),
  `thickness` (16 / 10), track `--swoosh-track`; gaps of 3 units of a 465-unit circle (r 74) and a minimum arc of 6
  units for a part above zero; segments start at the top and go clockwise. Existing callers keep their look (defaults
  unchanged).
- `product/disk-card/disk-card.tsx` (+ `disk-card.module.css`, `disk-legend.tsx`, stories, test, `story-data.ts`) —
  `DiskCard({ volume, phase, failure, onScan, compact = false, preview? })`, `phase: ScanPhase` (`idle | finished |
  partial | failed`; scanning is plan 03's card):
  - `section` labelled `ui.disk-card.label` "Your disk"; flex, centred, gap 40 px, padding 22 px 28 px, radius 18 px,
    1 px border `--tile-border`, background `--surface-card`.
  - The ring: segments in this order: files-and-apps, mac-os, system-space, can-be-freed (Free is the track); its
    accessible name `ui.disk-card.ring` "{name}: {parts}" with every part's name and size, e.g. "Macintosh HD: 3.8 GB
    can be freed, 331 GB your files and apps, …". Centre: the can-be-freed size (30 px, 700, letter-spacing -0.5 px)
    over `ui.disk-card.can-be-freed` "can be freed" (12 px, `--part-can-be-freed`). Before the first scan (`idle`) the
    centre holds the primary button `ui.scan-hero.scan` "Scan" (`onScan`).
  - Beside it (flex 1, column, gap 10 px): a row with the volume's name (15 px, 600) and `ui.disk-card.free-of` "{free}
    free of {capacity}" (13 px, `--text-secondary`, the free size bold in `--text-primary`); the legend: a grid of two
    columns, gap 2 px 18 px; each row padding 6 px 8 px, radius 8 px, 13 px `--text-legend`, a 10 px swatch radius
    3 px (Free: a 1.5 px outline `--tone-free-outline`), the core's name (`SummaryKind`'s `name_key`), the size bold,
    and an "i" button (`ui.disk-card.about` "What is {name}?") opening the core's description
    (`description_key`) as the legend's "i" does today; Free's "i" adds the purgeable note when the core gives it.
    Order: Can be freed, Your files and apps, macOS, System space, Free. The Can be freed row is highlighted
    (background `--level-safe-chip`, name `--text-primary`, size `--part-can-be-freed`). Before the first scan its
    size reads `ui.disk-card.not-scanned` "Not scanned yet".
  - Hover or focus of a legend row highlights its arc, and of an arc its row (the ring's `highlighted`/`onHighlight`).
  - `failed`: the legend's place holds the existing `ErrorState` (`ui.scan-hero.failed`, the core's text,
    `ui.scan-hero.try-again` → `onScan`).
  - The disk-changed note (`core.storage.changed`) under the legend when the core says so.
  - `compact`: ring 96 px / 10 px, centre size 17 px, padding 14 px 18 px, gap 24 px; the same legend.
- `app/ui/src/locales/en.json` — the `ui.disk-card.*` texts above.
- `gallery/registry.ts` — stories: finished (the mockup's numbers), before a scan, failed, compact, a part too small
  to see, a highlighted row.

## Tests (negatives first)

`disk-card.test.tsx`, negative: "a part of zero size draws no arc but keeps its legend row"; "before a scan Can be freed
reads Not scanned yet and the centre is Scan"; "a failed scan shows the core's text and Try again, no legend".
Positive: "the legend lists the five parts in the core's order with their names and sizes"; "the ring's name reads
every part"; "pointing at a row highlights its arc, and the other way"; "the i of a part shows the core's
description"; "Free's i adds the purgeable note"; "compact draws the smaller ring".

## Checks

`npx vitest run src/product/disk-card src/design/data`; lint:ts, lint:code, knip, the CSS guard; `npx playwright test
e2e/gallery.spec.ts --project gallery --update-snapshots` for the new stories (look at each against the mockup);
`npx playwright test e2e/axe.spec.ts e2e/contrast.spec.ts` if they cover the gallery.

Commit: `feat(ui): the disk card with its five parts`.

## Conclusions

- `DiskCard` with its ring of the summary's parts, the centre, the two-column legend with an "i" each and the hover
  link both ways; the compact card's caption wraps inside the ring's hole (fix round). The ring keeps its `size` prop.
- The free line bolds the free size by finding it in the translated sentence (a missing match shows the plain line).
