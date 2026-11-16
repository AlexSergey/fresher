# 003/03/02 — Charts and the scan sweep

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `design/data/storage-ring/` — `StorageRing`: `segments: readonly { key: string; value: number; level: Level;
  label: string }[]`, `total: number` (the share each segment takes), `state: 'empty' | 'scanning' | 'done' |
  'partial'`, `size` (232 px default), `children` (the centre). An SVG: the track (white 6 %, 12 px stroke), one arc per
  segment in its level's colour with 2.5° gaps and round caps, a tiny segment drawn at least 2° so it is visible; a
  blurred copy of the arcs underneath at 45 % as their glow (static `filter: blur(10px)`). An arc grows by
  `stroke-dashoffset` over `--duration-slow` (the design's named exception). `partial` dashes the track. The centre is
  an HTML layer over the SVG. The ring is `role="img"` with a label listing the segments and their sizes (from the
  caller); a finished scan plays one sheen (a soft white gradient arc rotated once over `--duration-deliberate`,
  opacity in and out).
- `design/effects/scan-sweep/` — `ScanSweep`: a comet-like arc (a conic gradient from Iris at 0 % to Iris at 70 %
  alpha, masked to the ring's band with `mask-image`) rotated by a `transform` loop of `--duration-sweep`; fades in
  over `--duration-base` when `active` turns on and out over `--duration-move` when it turns off. Under reduced motion
  it is a static highlight; under off, frozen at its first frame. Used inside `StorageRing` while `scanning`.
- `design/data/usage-bar/` — `UsageBar`: the same segment shape, one row, 8 px high, 2 px gaps, `--radius-full`
  ends; each segment's width set once per update and its entrance by `transform: scaleX` from its left edge; an
  optional `Legend` (dot, label, size) in a wrapping grid. `role="img"` with the same label as the ring.
- `design/data/stat-tile/` — `StatTile`: `label`, `value` (a `ReactNode`: usually `SizeText` or `AnimatedNumber`),
  optional `hint`; `loading` shows a `Skeleton` line.

## Produces

The hero visuals of the overview (06/01) and later the storage map (plan 012).

## Checks

- Negative: `an empty ring draws only the track`, `a segment of 0 is not drawn`, `a tiny segment is drawn at least 2
  degrees`, `the sweep is absent when not scanning`, `the sweep does not rotate under reduced motion`, `the sheen plays
  once, not on every update`.
- Positive: `segment angles add up to the share of the total`, `the arcs keep their order and colours`, `the ring's
  label lists every segment`, `the usage bar's legend matches its segments`, `the stat tile shows a skeleton while
  loading`.
- Stories: ring empty, scanning (sweep frozen), partly filled, full with 1 and with 12 segments, partial; usage bar
  empty, 1 and 8 segments, with legend; stat tile value, partial, loading; screenshots looked at.
- The CSS guard passes (the only non-compositor property is `stroke-dashoffset`).

## Conclusions

- Built as written: `StorageRing`, `ScanSweep`, `UsageBar` and `StatTile`, with unit tests (negative first) and
  gallery stories; the screenshots were looked at. The CSS guard passes: only `stroke-dashoffset` (the arcs' growth)
  leaves the compositor.
- The segment shape gained `size: string` (the formatted size, from the caller), since the legend shows a size and the
  charts' accessible names list each segment with its size. Both charts take a `label` (the name of the whole) and
  name themselves "label: segment size, segment size, …" (`segment.ts`, `chartLabel`).
- The geometry is a pure function (`storage-ring/arcs.ts`): each arc and the 2.5° gap after it take the segment's
  share; an arc is at least 2°; when the raised tiny arcs push past the circle, the excess comes out of the larger
  ones in proportion, so no segment is dropped (only past 80 segments, which the circle cannot hold at their least).
  One segment that is the whole total is a closed circle without a gap.
- Round caps reach half a stroke past each end, so a dash is shortened by one stroke; an arc shorter than its caps
  (about 6°) is drawn square-ended, or a 2° arc would show as a 6° dot.
- Arcs grow on mount by a `stroke-dashoffset` keyframe and move by a transition of `stroke-dashoffset` and `transform`
  (the start's rotation); under reduced motion the rotation does not animate. The glow is a second, blurred SVG
  (WebKit applies a CSS `filter` reliably to an outer `<svg>`, not to its children); it is off under increased
  contrast.
- `role="img"` is on the ring's SVG, not on its wrapper, so the centre (the Scan button while idle) stays reachable.
- The sheen plays when the state goes from `scanning` to `done` or `partial`, keyed by a counter, in full motion only
  (it rotates). The sweep and the sheen are masked to the band with `mask-image` and its `-webkit-` twin.
- The sweep's comet is `--accent-text` (Iris 400) from transparent over 120°, at 70 % opacity; no new tokens.
- `UsageBar` draws widths in percent with `flex-shrink`, so the 2 px gaps fit in a full bar; a segment enters by
  `scaleX` on mount (none under reduced motion); widths never animate.
- `StatTile` is a `role="group"` named by its label, `aria-busy` while loading.
- The stories share their segments through `design/data/chart-stories.ts`; `ScanSweep` has its own story (frozen).

