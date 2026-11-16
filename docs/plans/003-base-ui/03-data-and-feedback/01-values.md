# 003/03/01 — Values

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `design/data/size-text/` — `SizeText`: takes the already formatted `text` and an `accessibleText` (the caller
  formats with `formatSize` of `i18n/`, so `design/` stays free of it), `completeness: 'complete' | 'partial' |
  'unknown'`; partial shows a quiet `+` in `--text-tertiary` with a tooltip slot; unknown shows `—`, never `0`;
  `data-numeric` for tabular figures; `size: 'body' | 'callout' | 'title-3' | 'display' | 'hero'`.
- `design/data/animated-number/` — `AnimatedNumber`: `value: number`, `format: (n: number) => string`,
  `duration?: 'slow' | 'deliberate'`; shows `format(useAnimatedNumber(value))` in an `aria-hidden` span and
  `format(value)` in a `VisuallyHidden` copy; a value that drops (a new scan) jumps without counting down.
- `design/data/danger-badge/` — `DangerBadge`: `level: 'safe' | 'low' | 'medium' | 'high'`, `label` (the level's word,
  from the caller), `size: 'small' | 'medium' | 'large'`, `description?` (the large badge's sentence). Always the
  level's icon (`ShieldCheck`, `Shield`, `TriangleAlert`, `OctagonAlert`) and the word, on the level's tint with its
  border; the High badge carries `--glow-high` at rest; no pulse.
- `design/data/path-text/` — `PathText`: `path: string`; mono, footnote size; truncated in the middle so the start and
  the file name stay (a measured split: the component keeps the last segment whole and trims the middle with an
  ellipsis to fit its width, re-measured by `ResizeObserver`); the full path in its `Tooltip` and accessible text.

## Produces

The value components used by every card, row, chart and dialog.

## Checks

- Negative: `an unknown size never shows 0`, `a partial size says "at least" in its accessible text` (the caller's
  text, asserted to be used), `a falling value does not count down`, `the counting text is hidden from assistive
  technology`, `a danger badge without a label is a type error`, `a short path is not truncated`.
- Positive: `the number reaches its value after the duration` (fake clock), `motion off shows the value at once`,
  `each level has its own icon and word`, `a long path keeps its first folder and its last segment`, `the full path is
  in the tooltip`.
- Stories: size complete, partial, unknown, each size step; animated number final value; danger badge 4 levels × 3
  sizes; path short, long; screenshots looked at.

## Conclusions

- Built as written: `SizeText`, `AnimatedNumber`, `DangerBadge` (which also exports the `Level` type the charts use)
  and `PathText`, each with unit tests (negative first) and gallery stories; the screenshots were looked at.
- `SizeText`'s tooltip slot is a `hint` prop of the partial size: the plus shows it on hover. The plus is not
  focusable, because the accessible text already says "at least". The props are a union, so an unknown size takes no
  `text` and a complete one takes no `hint`.
- `AnimatedNumber` jumps on a fall by mounting its counter again at the new value; `useAnimatedNumber` is unchanged.
- `DangerBadge` takes `description` only with `size: 'large'` (a type error otherwise). Icons are 12, 14 and 20 px
  (small, medium, large), set by the badge's CSS over the 16 and 20 px `Icon`.
- `PathText` measures the mono cell with a hidden ten-character probe and the frame's width, then cuts by character
  count (`truncate.ts`, a pure function with its own tests): one measurement per resize, no search. A file name wider
  than the row is kept whole after the ellipsis and clipped by the frame.
- The danger badge stories show each level in its three sizes in one story (4 stories, not 12), as the button's
  stories do. No PathText story with its tooltip open: the tooltip opens on hover only and has its own stories; the
  unit test checks the full path is in it.
- No new icon names were needed: the four level icons were already in `icons.ts`.
