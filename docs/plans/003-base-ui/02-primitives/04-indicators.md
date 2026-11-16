# 003/02/04 — Indicators

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `design/primitives/tooltip/` — `Tooltip`: wraps one focusable child; shows after 500 ms of hover or at once on
  keyboard focus, hides on Escape, blur and pointer leave; `role="tooltip"` linked by `aria-describedby`; placed above
  or below the child (flips when it would leave the viewport), positioned by the component (no Popover API in the
  baseline), rendered in a portal on `document.body`; fades in with a `--distance-1` offset.
- `design/primitives/badge/` — `Badge`: `tone: 'neutral' | 'accent'`, `count` variant (tabular figures, capped
  display "999+" with the full number in its accessible text).
- `design/primitives/spinner/` — `Spinner`: an SVG arc rotated by a `transform` loop; `label` for assistive
  technology; under reduced motion and off, a static arc.
- `design/primitives/progress-bar/` — `ProgressBar`: `value: number | null` (null is indeterminate), `label`;
  determinate fill by `transform: scaleX` (`--duration-slow`); indeterminate: a soft Iris highlight sweeping by
  `translateX`; `role="progressbar"` with `aria-valuenow` when determinate.
- `design/primitives/skeleton/` — `Skeleton`: `shape: 'line' | 'block' | 'circle'`; a shimmer highlight crossing it by
  `translateX` on a 1,400 ms linear loop; `aria-hidden`; static under reduced motion and off.
- `design/primitives/divider/` — `Divider`: horizontal or vertical hairline (`--border-subtle`), `role="separator"`.
- `design/primitives/visually-hidden/` — `VisuallyHidden`: content for assistive technology only.

## Produces

The indicators used by data components, the shell and the product components; `VisuallyHidden` for every counting
number.

## Checks

- Negative: `the tooltip does not show before 500 ms of hover`, `the tooltip hides on Escape and keeps the focus on
  its child`, `the badge count shows 999+ but says the full number`, `the skeleton is hidden from assistive technology`,
  `an indeterminate progress bar has no aria-valuenow`.
- Positive: `the tooltip shows at once on keyboard focus`, `the tooltip flips below near the top edge`, `the progress
  bar reports its value`, `the spinner is still under reduced motion`, `visually hidden text is in the accessibility
  tree`.
- Stories: tooltip top, bottom, long text; badge neutral, accent, count; spinner small, medium; progress 0, 40, 100 %,
  indeterminate; skeleton line, block, circle; divider both; screenshots looked at (loops frozen by motion off).
