# 011/02/03 — The UI shows why

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** app/ui

## Files

- `app/ui/src/state/reduce.ts` — on `module-finished`, the module's `leftOut` is kept with its outcome; a new scan
  clears it, as it clears the outcomes.
- `app/ui/src/screens/module/category-section.tsx` — under the category's findings, one quiet line per left-out
  count: `core.left-out.summary` (count and size as args) followed by the reason's text. Nothing is computed: the
  count, the size and both texts come from the core. A category with no findings but a left-out still shows the line.
- The component's stories and the gallery: a category with one left-out line, and with two.
- `app/ui/src/locales/en.json` — nothing new unless the line needs a label for a screen reader
  (`ui.module.left-out.label`: "Left out of this list").

## Checks

- Negative: `reduce.test.ts` — `a new scan clears the left-outs`, `a left-out of another scan is ignored`;
  `category-section` — `shows no line without a left-out`.
- Positive: `keeps the left-outs of a finished module`, `shows one line per reason with its count and size`,
  `shows the line for a category without findings`.
- `npx vitest run app/ui/src/state/reduce.test.ts app/ui/src/screens/module`; the UI e2e of the module screen gains
  one screenshot with the demo's left-out line (`npm run test:e2e`).

## Conclusions

Built (085cdec). `state/reduce.ts` keeps each module's `leftOut` (cleared by a new scan, ignored for another scan).
`product/left-out-lines` renders the core's `core.left-out.summary.one|other` and then `ProblemText` of the reason;
the category section takes the lines, and a category with left-outs and no findings is shown too (empty total).
New e2e screenshot `module-page-left-out`; the module page, zoom and banner baselines were updated and looked at.
A chromium keyboard test (Dialog, long body) fails with or without this change: it is left for the final wave.
Final review: `core.left-out.summary` is asked for with its count and the translator picks the form; the label key is
`ui.left-out-lines.label`; a category with only left-outs has a quiet row (no box, no disclosure) and leaves the page
once it is ignored. The chromium failure is a known issue
([`ui-keyboard-e2e-dialog-long-body`](../../../known-issues/fixed/ui-keyboard-e2e-dialog-long-body.md)).

