# 042/02/04 — The swoosh progress

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

The logo's swoosh as a progress line: a track, a gradient fill with a soft glow, and the logo's star at the head.
Mockup: [`scanning.dc.html`](../../../mockups/0.1.0/scanning.dc.html), the SVG of the section "Scanning".

## Read first

`design/primitives/progress-bar/` (the model: `role="progressbar"`, reduced motion), `design/motion/` and
`design/tokens/motion.css`, 02/01's `--swoosh-*` tokens.

## Files

- `design/data/swoosh-progress/swoosh-progress.tsx` (+ css, stories, test) — `SwooshProgress({ value, label })`,
  `value` 0 to 1 (clamped), `label` its accessible name:
  - An `svg` with `viewBox="0 0 830 226"`, filling its box (`width: 100%; height: 100%`, `preserveAspectRatio="none"`
    on the path layer), absolutely placed by its parent; `role="progressbar"`, `aria-label={label}`,
    `aria-valuemin=0`, `aria-valuemax=100`, `aria-valuenow` the rounded percent.
  - The path `M30 200 C220 110 480 70 800 50`, `pathLength="1000"`, stroke 14, round caps: the track
    (`--swoosh-track`); a glow copy (gradient stroke, `stroke-dasharray="{v*1000} 1000"`, `feGaussianBlur
    stdDeviation 6`, opacity 0.6); the fill (the same dash). The gradient: `x1 0 y1 1 x2 1 y2 0`, stops
    `--swoosh-start` and `--swoosh-end` (as CSS `stop-color: var(...)`). Ids from `useId` (several on a page).
  - The star: the logo's star path `M528 72 C529 81 531 83 540 84 C531 85 529 87 528 96 C527 87 525 85 516 84 C525 83
    527 81 528 72 Z` (centre 528, 84), fill `--text-primary`, moved to the fill's head: `getPointAtLength` on the path
    after layout (an effect), hidden while `value` is 0 or where the DOM has no `getPointAtLength` (unit tests).
  - The dash moves with `--duration-slow` and the enter easing; none under reduced motion (both the media query and
    `:root[data-motion='reduced']`).
- `gallery/registry.ts` — stories at 0 %, 62 % (the mockup), 99 %.

## Tests (negatives first)

Negative: "a value below 0 or above 1 is clamped"; "at 0 the star is hidden"; "without getPointAtLength the star stays
hidden and nothing throws". Positive: "its value is the progress bar's now"; "the fill's dash follows the value"; "two
on a page have their own gradient ids".

## Checks

`npx vitest run src/design/data/swoosh-progress`; lint:ts, lint:code, the CSS guard; `npx playwright test
e2e/gallery.spec.ts --project gallery --update-snapshots` for the stories (look at each), `e2e/motion.spec.ts` if it
lists the gallery's moving pieces.

Commit: `feat(ui): the swoosh progress`.

## Conclusions

- `SwooshProgress` with its glow and star head; `null` is indeterminate (a short sweeping dash). The fill moves by
  `stroke-dashoffset` (the CSS guard forbids a `stroke-dasharray` transition); a non-finite value counts as 0.
