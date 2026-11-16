# 003/02/02 — Toggles

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `design/primitives/checkbox/` — `Checkbox`: a real `<input type="checkbox">` (visually replaced, still the focus and
  click target), `checked: boolean | 'mixed'` (sets `indeterminate`), optional `label` and `description` (linked by
  `aria-describedby`), `disabled`. The box: `--border-control` outline at rest, Iris fill when checked; the check mark
  is an SVG path drawn by `stroke-dashoffset` (`--duration-fast`, `--ease-settle`), the mixed state a bar. Hover
  brightens the outline.
- `design/primitives/switch/` — `Switch`: `<button role="switch" aria-checked>`, a pill track and a thumb that slides
  with `transform` (`--duration-base`, `--ease-settle`); on: Iris track with a faint glow.
- `design/primitives/segmented-control/` — `SegmentedControl<T extends string>`: a `radiogroup` of options; one
  indicator element under the selected option slides with `transform` and `scaleX` (`--ease-settle`), measured once per
  change; arrows move the choice, as native radios do.

## Produces

`Checkbox` (rows, cards, settings, the confirmation's permanent choice), `Switch` (settings), `SegmentedControl`
(sorting and views later).

## Checks

- Negative: `a disabled checkbox ignores clicks and Space`, `the mixed state is not checked for assistive technology`
  (`aria-checked="mixed"` through `indeterminate`), `a disabled switch keeps its value`, `the segmented control does not
  move past the last option with the arrow key`.
- Positive: `toggles on click and Space`, `links its description`, `the switch reports aria-checked`, `the segmented
  control moves the choice with the arrows and Home and End`, `the indicator does not animate under motion off`.
- Stories: unchecked, checked, mixed, disabled, focus, with label and description; switch off, on, disabled, focus;
  segmented 2 and 3 options, each selected, focus, disabled; screenshots looked at.
