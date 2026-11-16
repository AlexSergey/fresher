# 003/01/02 — Tokens and base styles

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `app/ui/src/design/base/layers.css` — the one cascade order: `@layer reset, tokens, base, components, utilities;`,
  imported first.
- `app/ui/src/design/tokens/primitive.css` — the primitive palette of the design (grays, Iris, the four level hues,
  the reds, white at 4 / 6 / 10 / 16 / 36 %) and every precomputed tint, border and glow colour; the only file with
  colour values.
- `app/ui/src/design/tokens/semantic.css` — surfaces, borders, text, accent, danger fill, `--level-<level>-{fg,tint,
  border,glow}`, type tokens (size, line height, weight, tracking for the eleven steps), spacing, radii, sizes,
  elevation 1–3, glow tokens, the focus ring, z-index; the `prefers-contrast: more` overrides.
- `app/ui/src/design/base/reset.css` — a small reset (box sizing, margins, form controls inherit fonts, images block,
  `button` without browser chrome, `:focus-visible` uses `--focus-ring`, `:focus:not(:focus-visible)` none).
- `app/ui/src/design/base/typography.css` — `html { font-size: 13px }`, `system-ui` and `ui-monospace` stacks,
  antialiasing, `tabular-nums` on `[data-numeric]` and on every size, `text-wrap: balance` on headings inside
  `@supports`.
- `app/ui/src/design/base/effects.css` — utilities shared by components: `.u-visually-hidden`, the noise layer
  (`html::after`, fixed, `pointer-events: none`, the SVG at 3 %, off under increased contrast) and `noise.svg` (an
  `feTurbulence` tile, under 2 kB, inlined by Vite as a `data:` URL the CSP allows).
- `app/ui/src/design/index.css` — imports the files above in layer order.
- `app/ui/src/design/tokens/contrast.test.ts` — parses `primitive.css` and `semantic.css` as text, resolves each
  semantic token to its colour (alpha composited over the surface it is used on) and checks the design's pairs.
- `app/ui/src/main.tsx` — imports `design/index.css` instead of `styles.css`; `app/ui/src/styles.css` is removed and
  the bare screen of plan 002 keeps working on the new base (its class names move to a module file next to `app.tsx`).

## Produces

Every token later components use; the base look of every page.

## Checks

- Negative: `fails a text pair below 4.5 to 1` and `fails a control outline below 3 to 1` (on a fixture token set),
  `refuses a colour value outside primitive.css` (a grep over the other `.css` files for `#` and `rgb(`).
- Positive: every pair of the design passes — primary, secondary and tertiary text on window, raised, overlay and hover;
  accent text on raised and overlay; white on the accent fill's top and the danger fill's top; each level's fg on its
  tint and on raised; the control border on raised (≥ 3 : 1).
- The CSS guard passes on the new files.
- `npm run test:e2e:update --workspace app/ui` rewrites the two screenshots of plan 002 on the new base; both are
  looked at (graphite background, system font) before they are committed.
