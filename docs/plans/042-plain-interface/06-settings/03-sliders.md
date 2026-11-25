# 042/06/03 — The sliders

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

A whole-number setting is a slider from its minimum to its maximum (the core's bounds) with its number field and unit
beside it; a typed number outside the bounds keeps its refusal text as today. Mockup:
[`settings.dc.html`](../../../mockups/0.1.0/settings.dc.html), the row "Old after".

## Read first

`product/settings-row/{settings-row.tsx,number-field.tsx,texts.ts}` and their tests, 02/05's `Slider`,
`e2e/visual.spec.ts` ("the days field of Logs and crash reports", "the number field of Temporary files").

## Files

- `product/settings-row/number-row.tsx` (new; or inside `settings-row.tsx` if it stays small) — for `kind: 'number'`:
  the label and description over a row (flex, centred, gap 14 px): `Slider` (`min`, `max`, the stored value;
  `aria-label` `ui.settings-row.slider` "{label}, {unit}") and a compact `NumberField` (52 px wide, right-aligned,
  background `--surface-window`, border `--border-default`, radius 6 px, padding 5 px 7 px) followed by its unit
  (13 px, `--text-secondary`, the existing unit text). Moving the slider sends the value (debounced to the release of
  the pointer or 300 ms, so one drag is not dozens of writes); typing keeps today's checks; either updates the other.
  The limits' description stays on the field.
- `app/ui/src/locales/en.json` — `ui.settings-row.slider`.
- `e2e/` — `visual.spec.ts` the two number baselines (now sliders), `flows.spec.ts` "dragging the slider saves the
  number" (positive) and "a number typed outside its limits is refused and the slider does not move" (negative),
  `keyboard`, `axe`.

## Tests (negatives first)

`settings-row.test.tsx`: negative "a read-only file disables both the slider and the field"; "a typed number outside
the limits is not sent and the slider keeps the stored value"; positive "the slider shows the core's bounds"; "moving
it sends one value on release"; "typing a valid number moves the slider".

## Checks

`npx vitest run src/product/settings-row`; test:cov:ui; lint:ts, lint:code; `npx playwright test e2e/visual.spec.ts
e2e/flows.spec.ts e2e/keyboard.spec.ts e2e/axe.spec.ts --project webkit --project chromium` (look at each baseline).

Commit: `feat(ui): whole-number settings as sliders`.

## Conclusions

- A whole-number setting is a slider sent on release beside a number field; the field still sends a valid typed number
  at once (ruling: no change of behaviour). The slider keeps its moved value until the new value arrives (final
  review).
