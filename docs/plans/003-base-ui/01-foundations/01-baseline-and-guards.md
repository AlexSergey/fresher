# 003/01/01 — Baseline and guards

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui, repo

## Files

- `app/ui/src/test/css-guard.test.ts` — reads every `src/**/*.css` as text (`import.meta.glob('…', { query: '?raw' })`)
  and fails, naming the file and line, on:
  - a feature outside the baseline: `color-mix(`, `@property`, `linear(` as an easing, `@starting-style`,
    `transition-behavior`, `view-transition`, `content-visibility`, a `popover` selector;
  - nesting (a `{` inside a rule's block, or a selector starting with `&`);
  - `backdrop-filter` without a `-webkit-backdrop-filter` in the same block;
  - a `transition` or `transition-property` naming a property outside `transform`, `opacity`, `filter`,
    `stroke-dashoffset`, `background-color`, `border-color`, `color` (and `all` is refused);
  - a `@keyframes` step setting a property outside the same list.
  The list of allowed properties is one exported constant, so the design's budget lives in one place.
- `app/ui/vite.config.ts` — `build.target` and `build.cssTarget` stay `safari16`; a comment points to the design's
  baseline table.
- `eslint.config.ts` — `no-restricted-imports` per folder, as the design's boundary table says: `design/**` may not
  import `product`, `screens`, `state`, `ipc`, `i18n`; only `design/icons/**` may import `lucide-react`; `product/**`
  may import `ipc/bindings.generated` with `import type` only, never `ipc/index`, `state`, `screens`; `gallery/**` may
  not import `state` or `ipc/index`; nothing outside `src/ipc/` imports `@tauri-apps/api`.
- `knip.json` — entries for the later gallery and stories globs (`gallery.html`, `src/gallery/main.tsx`,
  `src/**/*.stories.tsx`), so knip sees them once they exist.

The folders the rules name do not exist yet; the rules are written now so every later step is held by them.

## Produces

The guard every CSS file of the plan passes, and the boundaries every later file is checked against.

## Checks

- Negative (`describe('negative cases')`, on fixture strings): `refuses color-mix`, `refuses @property`, `refuses
  nesting`, `refuses a backdrop-filter without its prefixed twin`, `refuses a transition of width`, `refuses transition
  all`, `refuses a keyframe that moves top`.
- Positive: `accepts a transition of transform and opacity`, `accepts the prefixed backdrop-filter pair`, `accepts the
  current styles` (the files under `src/`).
- A throwaway file in `src/design/` importing `src/state/store` fails `npm run lint:code`; removed before the commit.
- `npx vitest run src/test/css-guard.test.ts` and `npm run lint` pass.
