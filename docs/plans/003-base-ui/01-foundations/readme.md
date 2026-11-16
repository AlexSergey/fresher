# 003/01 — Foundations

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

Everything the components stand on: the Safari 16.0 baseline and its guards, the import boundaries, the tokens and the
base styles of "Graphite and Iris", the motion system (tokens, modes, hooks), the icons, and the gallery with its
screenshot tests. The design: [`../00-design.md`](../00-design.md), "The platform", "The visual language", "Motion",
"The design system", "Testing".

Depends on nothing in this plan. Runs in parallel with plan 05.

## Steps

| #   | Step                                                    | Status   | Depends on | Shared files                                                                         |
| --- | ------------------------------------------------------- | -------- | ---------- | ------------------------------------------------------------------------------------ |
| 01  | [Baseline and guards](./01-baseline-and-guards.md)      | ✔️ done    | —          | `eslint.config.ts`, `app/ui/vite.config.ts`, `knip.json`                              |
| 02  | [Tokens and base styles](./02-tokens-and-base.md)       | ✔️ done    | 01         | `app/ui/src/main.tsx`, `app/ui/src/app.tsx`, `app/ui/src/styles.css` (removed), the existing screenshots |
| 03  | [The motion system](./03-motion.md)                     | ✔️ done    | 02         | —                                                                                    |
| 04  | [Icons](./04-icons.md)                                  | ✔️ done    | 01         | `app/ui/package.json`, `package-lock.json`                                           |
| 05  | [The gallery and visual tests](./05-gallery.md)         | ✔️ done    | 02, 03     | `app/ui/package.json`, `app/ui/playwright.config.ts`, `app/ui/vite.config.ts`, `knip.json`, `app/ui/e2e/visual.spec.ts` (calls the motion helper) |

Step 04 is independent of 02 and 03 and may run beside them.

## Conclusions

Done on 2026-11-15, one commit per step.

### What was measured

- **Contrast**, computed by `design/tokens/contrast.test.ts` from the token files. Every pair of the design passes;
  no token moved:

  | Pair                                   | window | raised | overlay | hover |
  | -------------------------------------- | ------ | ------ | ------- | ----- |
  | `--text-primary`                       | 16.50  | 15.45  | 14.21   | 13.57 |
  | `--text-secondary`                     | 8.11   | 7.60   | 6.99    | 6.67  |
  | `--text-tertiary`                      | 6.03   | 5.65   | 5.20    | 4.96  |
  | `--accent-text`                        | —      | 6.86   | 6.31    | —     |

  White on the accent fill's top 4.73, on the danger fill's top 4.62; the control border on raised 3.34. Each level's
  fg on its tint / on raised: Safe 7.28 / 9.57, Low 6.77 / 8.76, Medium 7.50 / 9.89, High 5.32 / 6.48. The precomputed
  tints (`#1B322B`, `#1D2F37`, `#342D21`, `#362327`) are exactly each fg at 14 % over raised. Under the raised
  surface's top sheen (white 3.5 %) primary text is 14.19 and tertiary 5.19.
- **Icons**: the whole allowlist (18 icons) and `Icon` add 7.8 kB of JavaScript (3.0 kB gzipped) when imported;
  nothing when not imported. `lucide-react` 1.70.0 (ISC, no dependencies) — the newest release, 1.73.0, was one day
  old, so a nine-day-old one was pinned.
- **The noise tile** is 324 bytes and is inlined by Vite as a `data:` URL (the CSP's `img-src data:`).
- **The app's build** still writes only `index.html` to `dist/` (CSS 10.5 kB, 3.2 kB gzipped); `build:e2e` writes the
  app and the gallery to `dist-e2e/`.

### The Safari 16.0 baseline, checked

What the foundations use, with the Safari version that brought it: `@layer` (15.4), `:focus-visible` (15.4), `:not()`
with a selector list (9), `inset` (14.1), flex `gap` (14.1), `clip-path: inset()` (13.1), `rgb()` with a space and a
`/ alpha` (12.1), `ui-monospace` (13.1), `font-variant-numeric` (9.1), `prefers-contrast` (14.1),
`prefers-reduced-motion` (10.1), the Web Animations API with `Animation.finished` (13.1), `Array.prototype.toSorted`
(16.0). `text-wrap: balance` (17.5) is inside `@supports`. The CSS guard refuses the rest of the design's table:
`color-mix()` (16.2), `@property` (16.4), nesting (16.5), `linear()` (17.2), `@starting-style` (17.5),
`transition-behavior` (17.4), View Transitions and `content-visibility` (18.0), the Popover API (17.0), and
`backdrop-filter` without its `-webkit-` twin.

### Decided while building

- **The CSS guard and the colour check read the stylesheets from the disk** (`node:fs`), not with
  `import.meta.glob('…?raw')`: Vitest empties every CSS import, even with `?raw`, so the plan's way would have checked
  empty strings and always passed.
- **Primitives added** to keep every colour value in `primitive.css`: `--white`, `--white-a0`, `--white-a3-5` (the
  raised sheen), `--white-a52` (the control border under increased contrast), `--black-a50/60/70` (the shadows),
  `--gray-950-a45` (the sidebar's veil), the Iris alphas of the glows and the focus ring, and each level's 30 / 32 /
  40 / 45 % alphas.
- **Composite tokens are split** where one custom property cannot hold the recipe: `--elevation-N-{surface,border,
  shadow}`, `--accent-fill-{top,bottom}` and `--danger-fill-{top,bottom}` beside their gradients (the contrast test
  reads the tops), `--surface-raised-sheen` beside `--surface-raised`, `--text-on-fill`, `--z-texture` (300, the noise
  layer above everything). Motion adds `--scale-press-card`, `--stagger-steps` and `--loop-state` (`running`, or
  `paused` under reduced and off), which loops read for `animation-play-state`.
- **Motion off** zeroes every duration token except the two loops, which stay at their length and are paused, so a
  loop stands at its first frame; a rule in the `utilities` layer zeroes transition durations and delays and pauses
  animations under `[data-motion='off']`. An unknown `data-motion` value is ignored: the system setting decides.
- **The import boundaries** use `@typescript-eslint/no-restricted-imports` (for `allowTypeImports`) with one config per
  folder. `src/test/setup.ts` may import `@tauri-apps/api/mocks` (it clears the mocks after each test).
- **knip and a design system without consumers**: the hooks' tests import from `design/motion/index.ts`, the icon test
  lists every allowlisted icon by name (a name the pinned version no longer has fails there), and a type is exported
  only once something uses it. The cost: an icon no component uses is still "used" by that test, so removing an icon
  means removing it from the test too.
- **The CSP check runs before each screenshot**, not after the test: in WebKit, the production policy also blocks the
  `<style>` Playwright injects to take a screenshot. The gallery spec opens a fresh document per story for the same
  reason. A throwaway `style` attribute in `gallery.html` failed the spec with `style-src-attr blocked inline`.
- `smoke.spec.ts` got the CSP check too (two hooks at the top), and `vitest.config.ts` leaves `src/gallery/main.tsx`
  out of the coverage, like `src/main.tsx`. `dist-e2e/` is ignored by `.gitignore`, which knip follows.

### Learned

- Vitest's fake timers drive `requestAnimationFrame` and `performance.now()` (a frame every 16 ms), so the tweens are
  tested on the fake clock without a hand-written frame loop.
- happy-dom delivers `MutationObserver` records in a microtask: a test that sets `data-motion` awaits an `act`.
- Two overlapping `act` calls (finishing several animations with `Promise.all`) leave React's test renderer in a state
  where later tests' effects never run; the helpers finish animations one at a time.
