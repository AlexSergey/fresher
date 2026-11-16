# 003/01/04 — Icons

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `app/ui/package.json`, `package-lock.json` — `lucide-react` as a production dependency, at an exact version
  (`save-exact`), its licence (ISC) checked.
- `app/ui/src/design/icons/icons.ts` — the only file importing `lucide-react`: re-exports by name the icons the app
  uses; this step adds the first set: `ShieldCheck`, `Shield`, `TriangleAlert`, `OctagonAlert` (the danger levels),
  `Check`, `Minus`, `X`, `Search`, `ChevronRight`, `ChevronDown`, `LayoutGrid`, `Settings`, `ListChecks`, `RotateCw`,
  `Square`, `Trash2`, `Info`, `CircleAlert`. Later steps add the names they need, one line each.
- `app/ui/src/design/icons/icon.tsx` — `Icon` sets the defaults of the design: size 16 (props `size: 16 | 20 | 24`),
  stroke 1.75, `currentColor`, `aria-hidden` unless a `label` is given (then `role="img"` and `aria-label`).
- `app/ui/src/design/icons/icon.test.tsx`.

## Produces

`Icon` and the named icons for every later component.

## Checks

- Negative: `is hidden from assistive technology without a label`, `does not accept a size outside the scale` (a type
  test with `// @ts-expect-error`).
- Positive: `has the role img and its label when labelled`, `draws with the current colour`.
- `npm run build` shows only the imported icons in the bundle (the bundle grows by a few kilobytes, not by the whole
  set); `npm run knip` passes; the licence check of the production dependencies passes.
