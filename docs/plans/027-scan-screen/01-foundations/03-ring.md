# 027/01/03 — The ring draws a disk

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** app/ui, e2e

The storage ring learns what the Scan screen's ring needs: a colour of its own for every part of a disk, the Free
slot left undrawn as the gap where the ring does not close with its label, and hover linking. The hero keeps calling
it as today until 02/01. The design: [`../00-design.md`](../00-design.md), sections 2.1, 2.3 and 2.4 (agent's calls
6 and 8).

## Files

- `app/ui/src/design/data/segment.ts` — `Tone` as in the design, section 2.3 (`plain` → `your-folders`; `system` →
  `mac-os`, `swap-and-sleep`, `other-volume`, `disk-own`); its comment rewritten.
- `app/ui/src/design/tokens/semantic.css` — `--part-your-folders`, `--part-mac-os`, `--part-swap-and-sleep`,
  `--part-other-volume`, `--part-disk-own`, replacing `--tone-plain` and `--tone-system` (every use updated; the
  token story in `app/ui/src/gallery/tokens.stories.tsx` lists them). Each at least 3:1 against `--elevation-1-surface`;
  distinguishable from each other and from the four level colours.
- `app/ui/src/design/data/storage-ring/arcs.ts` — an `Arc` gains `drawn: boolean`: a segment of tone `free` takes its
  slot (and its gap) like any other but is `drawn: false`. `arcsOf` keeps every other rule (least sweep, the excess
  cut, the whole circle for a single segment; a single `free` segment draws nothing).
- `app/ui/src/design/data/storage-ring/storage-ring.tsx` —
  - undrawn arcs are not rendered; `gapLabel?: string`: when given and there is an undrawn arc, a label at the middle
    of its slot, just outside the band (the ring's box grows by the label's room only when a label is given, so the
    hero's layout and today's stories stay as they are without it), `aria-hidden` (the legend carries it);
  - `highlighted?: null | string` and `onHighlight?: (key: null | string) => void`: each drawn arc's
    `onPointerEnter`/`onPointerLeave` call `onHighlight(arc.key)` / `onHighlight(null)` (`pointer-events: stroke` on
    the arcs only); while `highlighted` names a key, every other arc has `data-dimmed`; the gap label has
    `data-highlighted` when the free segment's key is highlighted.
- `app/ui/src/design/data/storage-ring/storage-ring.module.css` — `data-tone` rules for the new tones;
  `.arc[data-dimmed] { opacity: 0.35 }` with an opacity transition over `--duration-fast` (none under reduced
  motion); the gap label (`--text-secondary`, footnote type; `--text-primary` when highlighted).
- `app/ui/src/design/data/usage-bar/usage-bar.module.css` — the same tones (the other volumes' bars).
- `app/ui/src/product/storage-map/parts.ts` — `partTone`: `your-folders` → `your-folders`; `mac-os`,
  `swap-and-sleep`, `other-volume`, `disk-own` → their own tones; the rest unchanged; `explainedTone` unchanged.
- `app/ui/src/screens/overview/overview-screen.tsx` — the one line that passes `'plain'` as a second volume's
  `explained` passes `explainedTone(totals, null)` instead (`plain` no longer exists; a level colour, the highest of
  the counted categories, as the home's fallback).
- `app/ui/src/design/data/storage-ring/storage-ring.stories.tsx` — new states `disk-with-free-gap` (levels, parts,
  free, `gapLabel="Free"`) and `disk-highlighted` (one key highlighted); the other stories unchanged.
- Tests: `arcs.test.ts`, `storage-ring.test.tsx`, `product/storage-map/storage-map.test.tsx` (tones), and the
  contrast spec's tone list (`app/ui/e2e/contrast.spec.ts`) if it enumerates the tones.
- Baselines: the new ring stories; any story or screen whose colours changed (`StorageMap`, `UsageBar`, the overview
  with a second volume) regenerated and looked at.

## Produces

`StorageRing`'s `gapLabel`, `highlighted`, `onHighlight`; the part tones; `partTone` per kind — for 02/01.

## Checks

- `arcs.test.ts`, negatives first: `describe('negative cases')` — `it('draws nothing for a free segment')`; `it('a
  single free segment draws no arc')`; `it('never passes the full circle with a free slot among tiny arcs')`;
  `describe('positive cases')` — `it('leaves the free slot where it falls, the next arc after it')`; `it('a ring of
  parts and free closes only with the gap')` (drawn sweeps + gaps + free slot = 360).
- `storage-ring.test.tsx`, negatives first: no gap label without `gapLabel` or without a free segment; no arc dimmed
  while nothing is highlighted; then: the label at the free slot; pointer enter and leave on an arc call
  `onHighlight` with its key and `null`; a highlighted key dims every other arc.
- `npm run test --workspace app/ui -- src/design/data src/product/storage-map`
- `npm run lint:ts`; `npm run knip`; `npx eslint` and `npx prettier --check` on the changed files.
- `npm run test:e2e:update --workspace app/ui -- gallery.spec.ts visual.spec.ts contrast.spec.ts` (look at every
  changed PNG: only tones of the neutral parts and the new stories may change), then
  `npm run test:e2e --workspace app/ui -- gallery.spec.ts visual.spec.ts contrast.spec.ts axe.spec.ts`.

## Conclusions

Not written yet.
