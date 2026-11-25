# 042/02/05 — The slider

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

A whole-number slider from a minimum to a maximum, its ends labelled. Settings puts a number field beside it (06/03).
Mockup: [`settings.dc.html`](../../../mockups/0.1.0/settings.dc.html), the row "Old after".

## Read first

`design/primitives/{switch,text-field,checkbox}/` (the model of a primitive: label, description, disabled, forced
colours `design/primitives/forced.ts`), the tokens.

## Files

- `design/primitives/slider/slider.tsx` (+ css, stories, test) — `Slider({ label, min, max, value, onChange, disabled,
  describedBy? })`: a row (flex, centred, gap 14 px): the minimum (12 px, `--text-tertiary`), a native `input
  type="range"` (`step 1`, `aria-label={label}`, `accent-color: var(--accent-fill-top)`, flex 1), the maximum.
  `onChange(number)` on every input; a value outside the bounds is clamped for display only.
- `gallery/registry.ts` — stories: the mockup's (1 to 365, 90), disabled.

## Tests (negatives first)

Negative: "a disabled slider sends nothing"; "a value outside its bounds is shown at the nearest bound". Positive:
"shows its minimum and maximum"; "moving it sends the whole number"; "the arrow keys step by one".

## Checks

`npx vitest run src/design/primitives/slider`; lint:ts, lint:code; `npx playwright test e2e/gallery.spec.ts
e2e/primitives.spec.ts --project gallery --update-snapshots` for the stories (look at each).

Commit: `feat(ui): the slider`.

## Conclusions

- The `Slider` primitive with `onCommit` on release (pointer, key, change; ruling C-4), deduplicated; its focus ring
  is an outline. Native arrow keys are not tested in jsdom (ruling D-2).
