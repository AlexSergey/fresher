# 042/07/01 — The sidebar's logo

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

The logo, 22 px, beside "Fresher" at the top of the sidebar. Mockup: [`scan.dc.html`](../../../mockups/0.1.0/scan.dc.html),
the first block of the `nav` (its inline SVG is the 22 px drawing to build from).

## Read first

`screens/frame-sidebar.tsx` (`ui.app.title`), `screens/app-frame.module.css` (`.name`), the mockup's sidebar SVG and
`docs/mockups/0.1.0/logo.svg`, the tokens (`--swoosh-start`, `--swoosh-end`, `--gray-950`), `design/icons/` (how an
inline SVG component is written here).

## Files

- `design/brand/logo-mark.tsx` (+ css, stories, test) — `LogoMark({ size = 22 })`: the mockup's sidebar SVG as a
  component, `viewBox="0 0 100 100"`, `aria-hidden`: the leaf (`M8 92 C8 46 32 10 72 8 L88 8 Q92 8 92 12 L92 54 C92
  80 70 92 44 92 Z`) filled with the body gradient (`#2a2f38` to `#0e0f12`, `x1 0 y1 0 x2 0.4 y2 1`), the swoosh band
  clipped to the leaf (`M6 96 C28 60 56 44 96 34 L96 48 C64 56 40 70 20 96 Z`) filled with the band gradient
  (`--swoosh-start` to `--swoosh-end`, `x1 0 y1 1 x2 1 y2 0`), the rim (the leaf's path, no fill, the band gradient,
  `stroke-opacity 0.5`, `stroke-width 1`, `vector-effect non-scaling-stroke`), the star (`M34 17 C35 28 37 30 48 31
  C37 32 35 34 34 45 C33 34 31 32 20 31 C31 30 33 28 34 17 Z`) in `--text-primary`. The body colours become tokens
  (`--logo-body-top: #2a2f38` as a primitive `--gray-725`, and `--gray-950`). Gradient and clip ids from `useId`.
- `screens/frame-sidebar.tsx` — the name block: flex, centred, gap 9 px, 16 px 600, padding 0 10 px 18 px: `LogoMark`
  then `ui.app.title` "Fresher".
- `gallery/registry.ts` — the mark's story.
- Baselines: every screenshot with the sidebar changes (`visual`, `zoom`, `keyboard`, `contrast` where they shoot the
  frame).

## Tests (negatives first)

`logo-mark.test.tsx`: negative "the mark is hidden from assistive technology"; "two marks have their own ids";
positive "draws the leaf, the band, the rim and the star". `app-frame.test.tsx`: positive "the sidebar shows the
logo beside the name".

## Checks

`npx vitest run src/design/brand src/screens`; lint:ts, lint:code, knip, the CSS guard; `npx playwright test --project
gallery --project webkit --project chromium --update-snapshots` for the sidebar's baselines (look at each; at 2x the
rim must be one device pixel).

Commit: `feat(ui): the logo beside the name in the sidebar`.

## Conclusions

- `LogoMark` (`design/brand/`) beside "Fresher"; it keeps the mockup's small-size drawing (ruling: thicker band,
  bigger star), while `logo.svg` stays the large-size source. 148 baselines were taken again from scratch.
