# 003/01/05 — The gallery and visual tests

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `app/ui/gallery.html`, `app/ui/src/gallery/main.tsx` — the dev-only page: imports `design/index.css`, renders the
  gallery.
- `app/ui/src/gallery/story.ts` — `type Story = { readonly component: string; readonly state: string; readonly
  render: () => JSX.Element; readonly width?: number; readonly surface?: 'window' | 'raised' | 'overlay' }`;
  `stories(component, { state: render, … })` builds the list; every `*.stories.tsx` file exports it as
  `export const stories` (named, as the lint requires).
- `app/ui/src/gallery/registry.ts` — every story of the app, found with `import.meta.glob('../**/*.stories.tsx',
  { eager: true })`, sorted by component and state; a duplicate `component/state` throws.
- `app/ui/src/gallery/gallery.tsx` — a sidebar of components and a pane of their states; `#/<component>` shows one
  component, `#/<component>/<state>` one state alone on its surface (what Playwright photographs); every story sits in
  an element `data-story="<component>/<state>"`.
- `app/ui/src/gallery/tokens.stories.tsx` — the first stories, so the pipeline is proven: the palette swatches, the
  type scale, the spacing, radii and elevation samples, the four level colours with their tints and glows.
- `app/ui/vite.config.ts` — in mode `e2e` the build has two inputs (`index.html`, `gallery.html`) and writes to
  `dist-e2e/`; the default build is unchanged and writes only `index.html` to `dist/`. `vite preview` sends the app's
  production `csp` (read from `tauri.conf.json`, as the dev server already does with `devCsp`).
- `app/ui/e2e/support/csp.ts` — `failOnCspViolation(page)`: an init script that records `securitypolicyviolation`
  events; every spec asserts none fired.
- `app/ui/package.json` — `build:e2e` (`tsc --noEmit && vite build --mode e2e`); `test:e2e` unchanged.
- `app/ui/playwright.config.ts` — the web server runs `npm run build:e2e` and serves `dist-e2e/`; a `gallery` project
  (WebKit only) for `gallery.spec.ts`, the existing `webkit` and `chromium` projects for the other specs.
- `app/ui/e2e/support/motion.ts` — `motionOff(page)`: an init script that sets `data-motion="off"` on `<html>` before
  the app runs; every visual spec calls it first, so the existing screenshots are stable too (the core mock is left to
  plan 05, which runs at the same time).
- `app/ui/e2e/gallery.spec.ts` — reads the registry's list from the page (`window.__stories__` set by the gallery in
  the e2e build), then for each story opens `#/<component>/<state>` and takes
  `toHaveScreenshot('<component>/<state>.png')` of its element.
- `knip.json`, `.gitignore` — `dist-e2e/` ignored.

## Produces

The gallery every later component adds its stories to, without editing a shared file, and its screenshot test.

## Checks

- Negative: `registry: two stories with the same component and state are refused`, `gallery: an unknown hash shows the
  list, not an error`.
- Positive: `registry: finds a stories file by its glob`, `gallery: renders one story alone by its hash`.
- A throwaway `style="…"` attribute in `gallery.html` makes the gallery spec fail on the CSP; removed before the
  commit.
- `npm run build && test ! -e dist/gallery.html` (the app's build has no gallery).
- `npm run test:e2e:ui` passes; the token stories' screenshots are looked at in WebKit; the existing screen screenshots
  are unchanged with motion off.
