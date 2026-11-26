# 003 — Base UI: design

**Status:** ✅ approved · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui, app/tauri, core, engine

## Intent

The user's task for the draft (translated): "Create a draft of plan 003 for the base UI. If we have HTML/CSS/JS in
Tauri 2, think about a design system for tests and extension, lay the ground for animations, list the components we
will build, pick the base style."

The user's request when the draft was taken up (translated): "For 003 we also need to come up with a beautiful dark
design, add all kinds of beautiful animation effects so it looks really cool."

So this plan does two things at once: it builds the design system and the base screens the draft described, and it
raises the visual side well beyond the draft — a striking, premium dark look with motion that makes the app feel alive,
without ever making the moment of deletion feel like a game.

## Goals

1. **A visual language people remember**: Dark Premium ([`goal.md`](../../goal.md), section 6) made concrete as
   "Graphite and Iris" — layered graphite surfaces, native translucency in the sidebar, light that comes from the
   accent and from the danger levels, one large hero visual per screen.
2. **Motion that feels alive and calm**: a scan that visibly breathes, numbers that count up, rows that glide in and
   re-sort, a satisfying but serious "cleaned" moment — all under motion tokens, choreography rules and a measured
   performance budget.
3. **A design system that is easy to test and to extend**: tokens, primitives, components, each shown in every state
   in a gallery that Playwright photographs.
4. **The list of components**, built once and reused by every module.
5. **The base screens**, working on the `demo` module of plan [002](../002-base-architecture/readme.md): overview,
   the generic module page, review, settings, the missing-permission state.
6. **Large scans stay smooth**: the UI no longer keeps every finding in memory; lists are paged from the core and
   virtualised ([`known-issues/fixed/ui-findings-kept-in-memory.md`](../../known-issues/fixed/ui-findings-kept-in-memory.md)).
7. **Safety and accessibility are part of "done"** for every component: keyboard, VoiceOver, contrast, reduced motion.

## Non-goals

- Screens of real modules, the storage map (plan [012](../012-storage-map/readme.md)) and the history charts (plan
  [010](../010-history/readme.md)): they come with their plans and reuse these components.
- Removal itself (plan [008](../008-safe-removal/readme.md)): this plan builds the selection, the review screen, the
  confirmation dialog and the result summary as components with their states; plan 008 wires them to the core's
  cleanup plan. No file is removed by anything this plan builds.
- The settings feature (plan [005](../005-settings/readme.md)): this plan renders the settings the core describes, one
  typed row each; groups, reset and "Rescan to apply" are plan 005's.
- The permission flow (plan [006](../006-permissions/readme.md)): this plan builds `PermissionCard` and shows it when
  the core says access is denied.
- Onboarding (plan [014](../014-onboarding/readme.md)).
- A light theme: the token tiers allow one later; nothing in this plan is built for it.
- Sound and haptics.

## Design

### The platform

The UI runs in the system WebKit, whose version follows macOS and its Safari updates. The minimum is macOS 13, so the
baseline is **Safari 16.0**: the oldest WebKit the app can meet. Playwright's WebKit is newer, so the baseline is held
by a guard on the source, not by the tests' browser.

| Feature                                                                                   | Safari   | In this plan                                                         |
| ----------------------------------------------------------------------------------------- | -------- | -------------------------------------------------------------------- |
| `@layer`, `:has()`, `:focus-visible`, container queries, `<dialog>`, `inert`              | ≤ 16.0   | used                                                                 |
| Web Animations API (`element.animate`, `Animation.finished`), `ResizeObserver`, `IntersectionObserver` | ≤ 16.0 | used: FLIP, exits, the counting numbers                    |
| `mask-image`, `filter`, `mix-blend-mode`, `font-variant-numeric`, `ui-monospace`           | ≤ 16.0   | used                                                                 |
| `backdrop-filter`                                                                         | prefixed until 18 | written as `-webkit-backdrop-filter` and `backdrop-filter`, by hand |
| `color-mix()`                                                                             | 16.2     | not used: every tint is a precomputed primitive token                |
| `@property` (animated custom properties)                                                  | 16.4     | not used: the ring is SVG, the sweep is a rotation                   |
| CSS nesting                                                                               | 16.5     | not used in source (no CSS transformer is added for it)              |
| `linear()` easing                                                                         | 17.2     | not used: cubic-bezier curves only                                   |
| `@starting-style`, `transition-behavior: allow-discrete`                                  | 17.4–17.5 | not used: entrances by keyframes, exits by `usePresence`            |
| `text-wrap: balance`                                                                      | 17.5     | progressive enhancement only (headings)                              |
| Popover API                                                                               | 17.0     | not used: tooltips positioned by the component                       |
| View Transitions API, `content-visibility`                                                | 18.0     | not used: screen changes by `usePresence`, long lists by `VirtualList` |
| `prefers-reduced-transparency`                                                            | none     | not needed: the native material honours "Reduce transparency" itself |

- A **CSS guard** (a Vitest test that reads every `.css` file of `src/` as text) fails on any feature of the "not used"
  rows, on nesting, on a `backdrop-filter` without its `-webkit-` twin, and on a transition or keyframe of a property
  outside the motion budget (below).
- The **content security policy** stays as it is (`style-src 'self'`): no `<style>` element and no `style` attribute in
  markup. React's `style` prop writes through the CSSOM, which the policy allows; components use it only to set custom
  properties (`--i`, `--spot-x`) and transforms. The e2e build is served with the production policy, and every spec
  fails on a `securitypolicyviolation` event, so a breach is caught by the tests, not in the app.
- Plan [013](../013-release/readme.md) runs the app on the oldest supported macOS before a release; that is where the
  real Safari 16 is seen.

### The visual language: Graphite and Iris

The window is a dark, quiet instrument; light means something. Three ideas carry the whole look:

1. **Depth from light, not from shadow.** Surfaces get lighter as they come forward; each carries a hairline border
   and a faint highlight on its top edge, as if lit from above. Only overlays cast a real shadow.
2. **One accent, Iris** (a cool blue-violet), for focus, the primary action and the live scan. Green is never the
   accent: green means Safe.
3. **Danger levels glow in their own colour**, softly, always with an icon and a word. The glow says "look here", the
   word says what it means.

Texture: a static noise layer at 3 % opacity over the whole window takes the flatness off the gradients (a small SVG
file, rendered once, no animation). A soft ambient light — a wide radial glow of Iris from the top of the overview —
sits behind the hero.

#### The window

- **Overlay title bar** with a hidden title and the traffic lights inset into the sidebar; the top 52 px of the sidebar
  and of the main pane are the drag region (`data-tauri-drag-region`), except over their controls.
- **The sidebar is native translucency**: Tauri's window effect `sidebar` (the macOS material that follows the active
  state of the window), seen through a transparent webview. It needs a transparent window, which on macOS needs
  Tauri's `macOSPrivateApi` — acceptable because the app is not on the Mac App Store (plan 013, non-goals). The main
  pane is opaque graphite. "Reduce transparency" in the system settings turns the material opaque by itself.
- **The fallback** is a solid graphite sidebar with a faint vertical gradient: used in the browser, in every test and
  screenshot, and whenever the window effect is not applied. `main.tsx` sets `data-window="vibrant"` on `<html>` only
  inside the app, so the screenshots are deterministic.
- **Content width**: the main pane's content is at most 1,120 px wide, with 32 px gutters (24 px under 1,000 px).

#### Palette

Primitive tokens (the only place a colour value is written). Every tint and glow is precomputed, because `color-mix()`
is not in the baseline.

| Primitive          | Value                        | Primitive           | Value                         |
| ------------------ | ---------------------------- | ------------------- | ----------------------------- |
| `--gray-1000`      | `#08090B`                    | `--iris-300`        | `#A7B1FF`                     |
| `--gray-950`       | `#0E0F12`                    | `--iris-400`        | `#8B98FF`                     |
| `--gray-900`       | `#15171B`                    | `--iris-500`        | `#7C8CFF`                     |
| `--gray-850`       | `#1C1F25`                    | `--iris-600`        | `#5465EB`                     |
| `--gray-800`       | `#1F232A`                    | `--iris-700`        | `#4553DC`                     |
| `--gray-750`       | `#262B33`                    | `--green-400`       | `#3DD68C`                     |
| `--gray-700`       | `#313741`                    | `--cyan-400`        | `#4CC3E6`                     |
| `--gray-600`       | `#444B57`                    | `--amber-400`       | `#F5B544`                     |
| `--gray-500`       | `#5A616D`                    | `--red-400`         | `#FF6B70`                     |
| `--gray-400`       | `#8A919C`                    | `--red-600`         | `#D9363E`                     |
| `--gray-300`       | `#A3A9B4`                    | `--red-700`         | `#C42A30`                     |
| `--gray-100`       | `#ECEEF2`                    | `--white-a{4,6,10,16,36}` | white at 4 / 6 / 10 / 16 / 36 % |

Semantic tokens (what components use):

| Semantic token          | Value                                          | Use                                                 |
| ----------------------- | ---------------------------------------------- | --------------------------------------------------- |
| `--surface-window`      | `--gray-950`                                   | the main pane                                       |
| `--surface-sidebar`     | `--gray-1000` → `--gray-950` gradient (fallback); `rgba(14,15,18,0.45)` over the material | the sidebar |
| `--surface-raised`      | `--gray-900` + a top gradient of white 3.5 % → 0 over 48 % | cards, rows' hover, the hero panel     |
| `--surface-overlay`     | `--gray-850`                                   | dialogs, menus, the selection bar, toasts           |
| `--surface-hover`       | `--gray-800`                                   | a hovered row or control                            |
| `--surface-pressed`     | `--gray-750`                                   | a pressed control, a selected nav item              |
| `--border-subtle`       | white 6 %                                      | card hairlines                                      |
| `--border-default`      | white 10 %                                     | hovered cards, dividers                             |
| `--border-strong`       | white 16 %                                     | overlays, the toolbar's scrolled hairline           |
| `--border-control`      | white 36 % (3.35 : 1 on raised)                | checkbox, switch and field outlines (WCAG 1.4.11)   |
| `--text-primary`        | `--gray-100`                                   | 15.4 : 1 on raised                                  |
| `--text-secondary`      | `--gray-300`                                   | 7.6 : 1 on raised                                   |
| `--text-tertiary`       | `--gray-400`                                   | 4.96 : 1 on hover, the darkest surface text sits on |
| `--text-disabled`       | `--gray-500`                                   | disabled controls only (exempt from AA)             |
| `--accent-text`         | `--iris-400`                                   | links, the focus ring; 6.9 : 1 on raised            |
| `--accent-fill`         | `--iris-600` → `--iris-700` gradient           | the primary button; white text 4.73 : 1 at its top  |
| `--accent-glow`         | Iris 500 at 35 %                               | the primary button's hover glow, the scan sweep     |
| `--danger-fill`         | `--red-600` → `--red-700` gradient             | the permanent-delete button only; white 4.62 : 1    |
| `--level-<level>-fg`    | safe `--green-400`, low `--cyan-400`, medium `--amber-400`, high `--red-400` | a level's icon and word |
| `--level-<level>-tint`  | its fg at 14 % over raised (`#1B322B`, `#1D2F37`, `#342D21`, `#362327`) | a level's badge background |
| `--level-<level>-border`| its fg at 32 %                                 | a level's badge border                              |
| `--level-<level>-glow`  | its fg at 40 %                                 | a level's soft glow                                 |

Every level's fg is at least 5.3 : 1 on its own tint and on every surface. The Low level moves from the draft's
`#5BB8FF` to cyan `#4CC3E6`, so it no longer sits next to the Iris accent; tertiary text moves to `#8A919C` so it meets
AA on the hover surface too. A Vitest test computes the contrast of every listed pair from the token files and fails
below the bar (4.5 : 1 for text, 3 : 1 for control outlines and icons).

**Increased contrast** (`prefers-contrast: more`): borders one step stronger, secondary text becomes primary, glows and
the noise layer are off.

#### Typography

System fonts only: SF Pro through `system-ui` (WebKit picks Text or Display by size), SF Mono through `ui-monospace`
for paths. Nothing is downloaded. Every number on screen uses tabular figures (`font-variant-numeric: tabular-nums`),
so a counting number never jitters. Sizes are in `rem` on a 13 px root (the macOS body size), so page zoom scales them.

| Token          | Size / line | Weight | Tracking  | Use                                                  |
| -------------- | ----------- | ------ | --------- | ---------------------------------------------------- |
| `--type-label` | 11 / 14     | 600    | +0.06 em, uppercase | section labels in the sidebar and above lists |
| `--type-caption` | 11 / 14   | 500    | +0.01 em  | badges, hints                                        |
| `--type-footnote` | 12 / 16  | 400    | 0         | secondary lines, paths (mono)                        |
| `--type-body`  | 13 / 18     | 400    | 0         | the default                                          |
| `--type-body-strong` | 13 / 18 | 600  | 0         | row titles, buttons                                  |
| `--type-callout` | 15 / 20   | 500    | −0.005 em | card titles                                          |
| `--type-title-3` | 17 / 22   | 600    | −0.01 em  | sizes on cards, dialog titles                        |
| `--type-title-2` | 22 / 28   | 600    | −0.015 em | page titles                                          |
| `--type-title-1` | 28 / 34   | 700    | −0.02 em  | the result summary's title                           |
| `--type-display` | 40 / 44   | 700    | −0.025 em | the freed space, stat heroes                         |
| `--type-hero`  | 56 / 60     | 700    | −0.03 em  | the one hero number per screen (inside the ring)     |

#### Spacing, radii, elevation

- **Spacing** on a 4 px grid: `--space-0-5` 2, `--space-1` 4, `--space-1-5` 6, `--space-2` 8, `--space-3` 12,
  `--space-4` 16, `--space-5` 20, `--space-6` 24, `--space-8` 32, `--space-10` 40, `--space-12` 48, `--space-16` 64.
- **Radii**: `--radius-xs` 4 (badges), `--radius-sm` 6 (controls, rows), `--radius-md` 10 (cards), `--radius-lg` 14
  (panels, dialogs), `--radius-xl` 20 (the hero panel), `--radius-full` (pills, the switch).
- **Sizes**: sidebar 232 px, toolbar 52 px, a control 28 px (small 24, large 36), a list row 44 px, a finding row 52 px.
- **Elevation** (each level adds to the one before):

| Level | Token            | Recipe                                                                                     | Use                          |
| ----- | ---------------- | ------------------------------------------------------------------------------------------ | ---------------------------- |
| 0     | —                | the window surface                                                                         | the main pane                |
| 1     | `--elevation-1`  | `--surface-raised`, 1 px `--border-subtle`, `inset 0 1px 0` white 4 %                      | cards, the hero panel        |
| 2     | `--elevation-2`  | border `--border-default`, `0 8px 24px -8px` black 50 %                                    | a hovered card, a tooltip    |
| 3     | `--elevation-3`  | `--surface-overlay`, border `--border-strong`, `0 24px 64px -16px` black 70 %, `0 0 0 1px` black 60 % | dialogs, the selection bar, toasts |

- **Glow tokens**: `--glow-accent` (1 px Iris ring at 45 % and a 20 px Iris halo at 30 %), `--glow-<level>` (the same
  recipe in the level's colour, used at rest only by the High badge and by a selected High row's edge).
- **The focus ring**: `0 0 0 2px var(--surface-window), 0 0 0 4px var(--accent-text), 0 0 16px 2px` Iris 400 at 45 % —
  a crisp ring with a soft glow, on `:focus-visible` only, drawn as a box shadow so it follows the radius.
- **Z-index**: content 0, sticky 10, the selection bar 40, toasts 50, dialogs 100, tooltips 200.

### Motion

Motion explains what happened and shows that work is alive. It is never decoration on the screens where the user
decides what to remove.

#### Tokens

| Token               | Value                              | Use                                                                 |
| ------------------- | ---------------------------------- | ------------------------------------------------------------------- |
| `--duration-press`  | 80 ms                              | the press scale of buttons and cards                                |
| `--duration-fast`   | 120 ms                             | hover, focus, exits, tooltips                                       |
| `--duration-base`   | 200 ms                             | entrances, dialogs, the selection bar                               |
| `--duration-move`   | 320 ms                             | FLIP moves, the nav indicator, the sweep fading out                 |
| `--duration-slow`   | 480 ms                             | a number retargeting during a scan, the ring's segments             |
| `--duration-deliberate` | 800 ms                         | the first fill of the hero number, the freed-space count, the sheen |
| `--duration-sweep`  | 1,600 ms (loop)                    | one turn of the scan sweep                                          |
| `--duration-breath` | 4,000 ms (loop)                    | the ambient light breathing during a scan                           |
| `--ease-standard`   | `cubic-bezier(0.2, 0, 0, 1)`       | things that move on screen                                          |
| `--ease-enter`      | `cubic-bezier(0, 0, 0.2, 1)`       | things that arrive                                                  |
| `--ease-exit`       | `cubic-bezier(0.4, 0, 1, 1)`       | things that leave                                                   |
| `--ease-settle`     | `cubic-bezier(0.34, 1.36, 0.64, 1)` | a small overshoot, for small controls only                         |
| `--ease-linear`     | `linear`                           | loops                                                               |
| `--distance-1/2/3`  | 4 / 8 / 16 px                      | how far an entering element travels                                 |
| `--scale-press`     | 0.97 (cards 0.99)                  | pressed                                                             |
| `--scale-enter`     | 0.98                               | a dialog arriving                                                   |
| `--stagger`         | 30 ms, at most 8 steps (240 ms)    | lists and cards entering                                            |

The tokens are CSS custom properties; `design/tokens/motion.ts` mirrors them for the hooks, and a test checks the two
agree.

#### Choreography rules

1. **One lead motion at a time.** During a scan the ring and the hero number lead; everything else moves quietly or
   not at all.
2. **Container, then content, then numbers.** A surface fades in (`--duration-base`), its children follow with the
   stagger, numbers count last.
3. **Exits are faster than entrances** (`--duration-fast` against `--duration-base`) and never travel more than
   `--distance-1`.
4. **Motion follows cause.** A row that moves came from somewhere (FLIP); a removed row leaves where it was and the
   rest close the gap; nothing appears from off-screen without a reason.
5. **Loops only while work runs**, and they stop the moment it stops. Nothing loops at rest.
6. **Nothing lasts longer than 800 ms**, except the loops tied to running work.
7. **The settle curve only on small controls** (switch thumb, checkbox mark, segmented indicator) — never on surfaces,
   lists, dialogs or numbers.
8. **The safety rule.** In the confirmation dialog and during a removal: no settle curve, no glow pulse, no counting
   number, no stagger. The dialog fades and scales in once (`--duration-base`, `--ease-standard`); the size and the
   highest danger level are static, so the user reads exactly what they confirm.
9. **Numbers count only when they grow** (the hero total, stat tiles, the freed space). Sizes in lists do not animate.
   The counting text is hidden from assistive technology; a visually hidden copy holds the real value.
10. **Reduced motion** (`prefers-reduced-motion: reduce`): no translation, scale or rotation; opacity fades of at most
    `--duration-fast`; loops become a static state with text; numbers jump to their value.
11. **Motion off** (`data-motion="off"` on `<html>`): every duration is 0 and every loop is paused at its first
    frame — for screenshots and tests. Playwright's helper sets it before the page loads.

The motion mode is one value, `full | reduced | off`, read by `useMotion()`; the CSS reads the same attribute and the
media query. A later user setting ("Reduce motion in the app", plan 005) would set the same attribute.

#### The animation set

| Moment                    | What moves                                                                                                    | Properties                     | Duration / easing                     |
| ------------------------- | ------------------------------------------------------------------------------------------------------------- | ------------------------------ | ------------------------------------- |
| App start                 | sidebar items stagger in; the hero panel fades and rises `--distance-2`                                        | opacity, transform             | base, enter, stagger                  |
| Screen change             | the old screen fades out; the new one fades in and rises `--distance-1`; the nav indicator slides to the item | opacity, transform             | fast out, base in; move, standard     |
| Idle overview             | the empty ring track with the **Scan** button in its centre; a static Iris glow behind it                       | —                              | —                                     |
| Scan starts               | the button presses, fades out; the hero number fades in at 0; the sweep fades in                               | transform, opacity             | press, fast, base                     |
| Scanning                  | a comet-like sweep circles the ring; the ambient light breathes; the hero number counts toward each new total; "Looked at N items" ticks; module chips light up, then get a check | transform (rotate), opacity; text | sweep 1,600 linear; breath 4,000; number slow |
| Segments grow             | each category's arc grows to its share                                                                         | `stroke-dashoffset` (exception) | slow, standard                        |
| Cards and rows arrive     | fade and rise `--distance-1`, staggered within a batch                                                         | opacity, transform             | base, enter, stagger                  |
| Re-sort                   | cards and rows glide to their new place (FLIP)                                                                 | transform                      | move, standard                        |
| Scan finishes             | the sweep fades out; one soft sheen passes over the ring once; the number settles on the final value            | opacity, transform             | move; deliberate                      |
| Hover                     | cards: border brightens, a pointer-following spotlight fades in, lift of 1 px; rows: background lightens        | opacity, transform; colour (exception) | fast, standard               |
| Press                     | scale to `--scale-press`                                                                                       | transform                      | press                                 |
| Primary button hover      | its glow fades in (a pseudo-element with the shadow, not an animated shadow)                                   | opacity                        | fast                                  |
| Focus                     | the focus ring and its glow appear                                                                             | box-shadow (no transition)     | instant                               |
| Toggle a checkbox         | the mark draws itself                                                                                          | `stroke-dashoffset` (exception) | fast, settle                         |
| Switch, segmented control | the thumb or indicator slides                                                                                   | transform                      | base, settle                          |
| Selection bar             | rises `--distance-3` and fades in when the first item is selected; its total counts quickly                    | opacity, transform             | base, enter; total 200 ms             |
| Confirmation dialog       | backdrop fades and blurs; the panel fades and scales from 0.98 — once                                         | opacity, transform; backdrop blur (static) | base, standard; exit fast  |
| Removed rows (plan 008)   | fade out in place, then the rest close the gap (FLIP)                                                          | opacity, transform             | fast, then move                       |
| The cleaned moment        | the check mark draws; "Freed" counts from 0; one soft green light wave expands behind the number and fades — once | `stroke-dashoffset`, opacity, transform | fast; deliberate, enter      |
| Skeletons                 | a soft highlight sweeps across                                                                                 | transform                      | 1,400 ms linear loop while loading    |
| Toasts                    | rise `--distance-2` and fade in; fade out                                                                      | opacity, transform             | base, enter; fast, exit               |
| Tooltip                   | fades in after 500 ms of hover, offset `--distance-1`                                                          | opacity, transform             | fast                                  |
| Toolbar on scroll         | a hairline and a soft shade appear when content scrolls under it (an `IntersectionObserver` sentinel)          | opacity                        | fast                                  |

**No particles and no canvas.** The scan feels alive from the sweep, the breathing light, the ticking numbers and the
rows gliding in; particles would cost frames on a long scan and pull the eye from the numbers.

#### Performance budget

- **60 fps**: on the reference Mac, during a scan streaming 50,000 findings (batches of 256 every 50 ms) and during
  every transition, the 95th percentile frame is at most 17 ms and no frame exceeds 50 ms. Measured in step 08/01 and
  written in the conclusions.
- **Compositor-only by default**: only `transform`, `opacity` and `filter` are transitioned or animated. Two named
  paint-only exceptions, each on small elements: `stroke-dashoffset` (the ring's arcs, the checkbox mark, the cleaned
  check mark), and `background-color`, `border-color` and `color` on hover (`--duration-fast`). Never `width`,
  `height`, `top`, `left`, `margin`, `padding`, grid tracks or `box-shadow`. The CSS guard enforces it.
- **No layout thrash**: the hooks read layout once, then write (FLIP's "first, last, invert, play"); `VirtualList`
  positions rows from their index, so a re-sort measures nothing.
- **`will-change`** is set by the hooks only while an animation runs, plus on the running loops; never more than four
  elements at once.
- **One live `backdrop-filter`** at a time (the dialog's backdrop), blur at most 16 px; the selection bar and toasts are
  solid.
- **The store renders at most 10 times a second** during a scan (progress throttled), and the overview is refreshed at
  most 4 times a second.
- **The DOM stays small**: a list renders at most about 60 rows, whatever its length; at most 10 pages of 100 findings
  are cached.
- **The bundle**: the main entry at most 200 kB of JavaScript and 40 kB of CSS, gzipped (checked in 08/01).

### The design system

#### Folders and boundaries

```
app/ui/src/
├── design/          generic, knows nothing of the product; every text comes in by props
│   ├── tokens/      primitive.css, semantic.css, motion.css, motion.ts
│   ├── base/        layers.css, reset.css, typography.css, effects.css, noise.svg
│   ├── motion/      use-motion, use-presence, use-animated-number, use-flip-list, use-spotlight
│   ├── icons/       icons.ts — the allowlist of lucide-react icons the app uses
│   ├── primitives/  button/, checkbox/, switch/, …            one folder per component
│   ├── data/        size-text/, storage-ring/, virtual-list/, …
│   ├── feedback/    notice/, toast/, announcer/, …
│   ├── layout/      app-shell/, sidebar/, dialog/, …
│   └── effects/     ambient-light/, scan-sweep/, spotlight/
├── product/         the app's components: take the core's types, render texts through i18n
├── screens/         overview/, module/, review/, settings/, app-frame.tsx — wire state to product
├── modules/         optional custom views per module (none in this plan); the generic module page is the default
├── state/  ipc/  i18n/  locales/  test/                  (from plan 002)
└── gallery/         the dev-only gallery: every component in every state
```

A component folder holds `<name>.tsx`, `<name>.module.css`, `<name>.test.tsx` and `<name>.stories.tsx` (kebab-case
file names, as the lint already requires).

Import boundaries, enforced by ESLint (`no-restricted-imports` per folder):

| Folder      | May import                                                                   | Never                                      |
| ----------- | ---------------------------------------------------------------------------- | ------------------------------------------ |
| `design/`   | `design/`, React, `lucide-react` (only in `design/icons/`)                    | `product/`, `screens/`, `state/`, `ipc/`, `i18n/` |
| `product/`  | `design/`, `i18n/`, types from `ipc/bindings.generated` (`import type`)       | `ipc/index`, `state/`, `screens/`          |
| `screens/`  | everything above, `state/`, `ipc/`                                           | `@tauri-apps/api`                          |
| `gallery/`  | `design/`, `product/`, `test/`                                               | `state/`, `ipc/index`                      |
| any but `ipc/` | —                                                                         | `@tauri-apps/api`                          |

#### CSS

- **Plain CSS with CSS Modules** (built into Vite), and one global cascade: `@layer reset, tokens, base, components,
  utilities`. Components use semantic tokens only, so a theme is one file.
- **Three token tiers**: primitive (`--gray-900`) → semantic (`--surface-raised`) → component (`--button-height`,
  defined in the component's module, from semantic tokens).
- No nesting, no `color-mix()`, no `@property` (the baseline); the CSS guard holds it.

#### Native elements first

`<dialog>` for modals (focus trap, Escape, `inert` on the rest), real `<button>`, `<input type="checkbox">` (with
`indeterminate` for the mixed state), `<select>` restyled, `<details>` where a disclosure is enough. No headless
component library until a component proves it needs one.

#### Icons

`lucide-react` (ISC), the one new dependency, pinned exactly. Only `design/icons/icons.ts` imports it and re-exports
the icons the app uses by name, so the bundle holds only those and an unused icon fails knip. Icons are 16 px (20 in
cards, 24 in the hero), stroke 1.75, `currentColor`.

The danger levels' icons escalate in shape, like signage: Safe `ShieldCheck`, Low `Shield`, Medium `TriangleAlert`,
High `OctagonAlert` — always with the level's word.

#### Motion hooks

All in `design/motion/`, small and tested with a fake clock and a fake `requestAnimationFrame`:

- `useMotion(): MotionMode` — `full | reduced | off`, from `data-motion` and the media query, live.
- `usePresence(open, { exit })` — keeps an element mounted until its exit animation (WAAPI) has finished; used by
  dialogs, toasts, the selection bar and screen changes (there is no `@starting-style` or View Transitions in the
  baseline).
- `useAnimatedNumber(target, { duration })` — a `requestAnimationFrame` tween that retargets smoothly from the value on
  screen; `reduced` and `off` return the target at once.
- `useFlipList(keys, { positions? })` — FLIP for keyed children: measures before and after a reorder (or takes the
  positions from `VirtualList`), then plays a transform with WAAPI.
- `useSpotlight(ref)` — writes the pointer position into `--spot-x` and `--spot-y` once per frame while hovered.

Stagger is CSS only: a component sets `--i` on each child and the keyframe's delay is
`calc(min(var(--i), 8) * var(--stagger))`.

### Components

Every component: every state in the gallery, a screenshot per state, unit tests (negative first), its accessible name
asserted, keyboard use checked, `reduced` and `off` motion checked.

| Group       | Component        | States in the gallery                                                                                                  |
| ----------- | ---------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Primitives  | Button           | primary, secondary, ghost, danger × small, medium, large × rest, hover, pressed, focus, disabled, busy                 |
|             | IconButton       | rest, hover, pressed, focus, disabled; with its tooltip                                                                |
|             | Checkbox         | unchecked, checked, mixed, disabled, focus; with a label and a description                                             |
|             | Switch           | off, on, disabled, focus                                                                                               |
|             | SegmentedControl | 2 and 3 options, each selected, focus, disabled                                                                        |
|             | TextField        | empty, filled, focus, invalid with its message, disabled                                                               |
|             | SearchField      | empty, with text and the clear button, focus                                                                           |
|             | Select           | closed, focus, disabled                                                                                                |
|             | Tooltip          | top, bottom; long text wraps                                                                                           |
|             | Badge            | neutral, accent, count                                                                                                 |
|             | Spinner          | small, medium                                                                                                          |
|             | ProgressBar      | 0 %, 40 %, 100 %, indeterminate                                                                                        |
|             | Skeleton         | line, block, circle                                                                                                    |
|             | Divider          | horizontal, vertical                                                                                                   |
|             | VisuallyHidden   | (not visual; unit tests only)                                                                                          |
| Data        | SizeText         | complete, partial (`42 GB+`), unknown (`—`), each with its accessible text                                             |
|             | AnimatedNumber   | final value (motion off); unit tests of the tween                                                                      |
|             | DangerBadge      | Safe, Low, Medium, High × small, medium, large (the large one with its sentence)                                       |
|             | PathText         | short, long (middle truncation), with the full path in its tooltip                                                     |
|             | StorageRing      | empty, scanning (sweep frozen), partly filled, full with 1 and with 12 segments, partial                               |
|             | UsageBar         | empty, 1 segment, 8 segments, with a legend                                                                            |
|             | StatTile         | value, partial value, loading                                                                                          |
|             | ListRow          | rest, hover, selected, focus, disabled                                                                                 |
|             | TreeRow          | collapsed, expanded, nested two levels                                                                                 |
|             | VirtualList      | 0, 10 and 100,000 rows; scrolled to the middle                                                                         |
| Feedback    | Notice           | info, warning, danger; with an action                                                                                  |
|             | EmptyState       | with an icon, a sentence and an action                                                                                 |
|             | ErrorState       | with the core's text and "Try again"                                                                                   |
|             | Toast            | info, success, error; with an action                                                                                   |
|             | Announcer        | (not visual; unit tests of its rate)                                                                                   |
| Layout      | AppShell         | with the fallback sidebar, narrow and wide                                                                             |
|             | Sidebar, NavItem | rest, hover, active (the indicator), with a count, focus                                                               |
|             | Toolbar          | at rest, scrolled (hairline)                                                                                           |
|             | PageHeader       | title only, with a subtitle and actions                                                                                |
|             | ScrollArea       | short, overflowing                                                                                                     |
|             | Surface          | elevation 1, 2, 3                                                                                                      |
|             | Dialog           | open with a short and a long body                                                                                      |
|             | Sheet            | open from the bottom                                                                                                   |
| Effects     | AmbientLight     | rest, breathing (frozen)                                                                                               |
|             | ScanSweep        | frozen at its first frame                                                                                              |
|             | Spotlight        | on a card, pointer at the centre                                                                                       |
| Product     | ScanHero         | idle (Scan in the ring), scanning, finished, partial, failed, nothing found                                            |
|             | ScanProgress     | starting, running with modules, one module failed, cancelled                                                           |
|             | CategoryCard     | each danger level, partial size, selected, hover                                                                       |
|             | ModuleSection    | finished, scanning, failed                                                                                             |
|             | FindingRow       | each danger level, long path, selected, focus                                                                          |
|             | FindingList      | loading (skeleton rows), a page, empty, failed                                                                         |
|             | SelectionBar     | 1 item Safe, many items with High, partial total                                                                       |
|             | ConfirmDialog    | Trash, permanent (the red button), a long list, the highest level High                                                 |
|             | ResultSummary    | all removed, some skipped with reasons                                                                                 |
|             | PermissionCard   | denied, unknown                                                                                                        |
|             | SettingsRow      | switch, number, choice, folders (read-only list), invalid, read-only file                                              |

#### Key product components

- **ScanHero** — the overview's hero panel (`--radius-xl`, elevation 1, the ambient light behind it). Left: the
  `StorageRing` (232 px, 12 px stroke), one arc per category, coloured by the category's highest danger level, 2.5°
  gaps, round caps, with a blurred copy of the arcs under them at 45 % as their glow. In its centre: "Found", the hero
  number, "in N items · M categories". Right: the stat tiles and the `UsageBar` with its legend. Idle: the ring's empty
  track holds the primary **Scan** button. Scanning: the sweep, the counting number, `ScanProgress` below. Finished:
  the sheen, then rest.
- **CategoryCard** — elevation 1; the level's icon in a tinted circle with its soft glow, the category's name and its
  one-line description from the dictionary, the size (`--type-title-3`, tabular) and the count; along the bottom edge a
  2 px share bar (`transform: scaleX`). Hover: border, spotlight, 1 px lift. A checkbox selects the whole category.
  Cards are sorted by size and glide when the order changes.
- **FindingRow** — checkbox, title, `PathText` in mono below it, `DangerBadge`, size on the right. A selected High row
  shows a 2 px edge in the level's colour with its glow.
- **SelectionBar** — floats at the bottom of the module and review screens (elevation 3, solid): the count, the total
  size, the highest danger level as a `DangerBadge`, "Review". Nothing above Safe is ever pre-selected.
- **ConfirmDialog** — the title says the action ("Move 128 items to the Trash?"); first the highest danger level
  (`DangerBadge` large, with its sentence), the total size and the count, then the exact list grouped by category with
  one plain sentence per category of what is lost, then "Delete permanently instead — this cannot be undone", a
  checkbox off by default, which turns the confirming button red and changes its label. Buttons: **Cancel** has the
  initial focus; the confirming button is never the default for Enter. Static: see the safety rule.
- **ResultSummary** — the cleaned moment: the check mark draws, "Freed" and the freed size count up from 0, one green
  light wave behind it; then what was moved, what was skipped and why, and "Open Trash".

### Data flow

#### Two additions to the core

The base screens need two answers the core does not give yet; both are additive and land in one step (05/01):

- **`Overview.found: Sizes`** — the sum of every finding of the current scan (the weakest completeness wins), so the
  hero number is the core's, not a sum made in the UI.
- **`FindingsQuery.order: FindingOrder`** — `found` (the order the scan found them, today's behaviour) or `largest`
  (allocated size, largest first; ties by id). The module page lists largest first. The engine sorts the matching
  findings per page request; plan 004 may index it if the measurement asks for it.

The bindings are regenerated; the golden report and the mocked core follow.

#### The store keeps totals, not findings

- `state/` keeps what the core says about the whole: the engine status, progress, module outcomes, the overview
  (totals per category and `found`), the dictionary, the last failure. The `groups` of findings are removed.
- During a scan the store refreshes the overview on `findings-added`, `module-finished` and `scan-finished`, at most
  4 times a second and always once at the end. This closes
  [`known-issues/fixed/ui-findings-kept-in-memory.md`](../../known-issues/fixed/ui-findings-kept-in-memory.md).
- **Findings are paged**: `state/findings.ts` keeps a page cache per query (`module`, `category`, `order`), pages of
  100, at most 10 pages, least recently used dropped. `useFindings(query, range)` gives `VirtualList` the rows it
  shows (or placeholders while a page loads). A new scan clears the cache; during a scan the visible pages are
  refreshed at most twice a second, and rows glide to their new places.
- **Selection** (`state/selection.ts`): findings, and whole categories minus the findings ticked off inside them (so
  selecting a large category loads none of its findings), cleared when a scan starts. The selection bar's total is a
  sum of the core's sizes for the selected items (the core's weakest-completeness rule, tested), a preview only: the
  confirmation dialog shows only what the core's cleanup plan says (plan 008).
- **Navigation** (`state/navigation.ts`): the current screen is a discriminated union (`overview`, `module { id }`,
  `review`, `settings`); no router.

### Screens

| Screen        | Answers                                | Content                                                                                       |
| ------------- | -------------------------------------- | --------------------------------------------------------------------------------------------- |
| Overview      | "Where did my space go?"               | `ScanHero`; the category cards by size; a `Notice` for a partial scan; `PermissionCard` when access is denied |
| Module page   | "What did this module find?"           | generic, from the module's descriptor: its categories as sections, each a `FindingList` (largest first); the selection bar |
| Review        | "What exactly am I about to remove?"   | the selection grouped by category, with what each costs; "Move to Trash…" opens `ConfirmDialog` (wired by plan 008) |
| Settings      | "What can I change?"                   | the core's settings, one `SettingsRow` each (generic; plan 005 adds the rest)                 |
| No permission | "Why do I see less?"                   | `PermissionCard`, shown on the overview, never blocking                                       |

The sidebar: the app's name, **Overview**, a "Modules" label with one item per module and its found size (live during
a scan), **Review** with the selection's count, and **Settings** at the bottom. The toolbar of each screen holds its
title and the **Scan** / **Stop** button.

Until plan 008, the "Move to Trash…" button of the review screen is disabled with a plain note; the dialog, its states
and the result summary are complete in the gallery and photographed. This is recorded in `docs/hacks/`.

### Accessibility

A requirement of every component, checked when it is built:

- **Keyboard**: everything is reachable and operable without a pointer; the focus ring is always visible; a dialog
  traps focus and returns it where it came from; lists use a roving focus (arrows, Home, End, Page Up and Down, Space
  to select).
- **VoiceOver**: real elements and roles first; labels from the dictionary; a virtual list's rows carry
  `aria-setsize` and `aria-posinset`; one polite `Announcer` speaks the scan's progress at most every 5 seconds and its
  end once.
- **The confirmation dialog** reads the count, the total size and the highest danger level before its buttons; the
  safe choice has the initial focus; the removing button is never the default for Enter.
- Meaning never rests on colour alone; contrast meets WCAG AA (tested from the tokens); sizes in `rem`, so page zoom
  scales the UI; `prefers-reduced-motion` and `prefers-contrast` are honoured.

### Testing

- **Unit tests** (Vitest, Testing Library) for every component and hook, negative cases first; coverage stays at or
  above 90 %.
- **The gallery**, a page of our own instead of a component workshop tool: `gallery.html`, built only in the e2e build
  mode (`npm run build:e2e` into `dist-e2e/`), never in the app's `dist/`. Each `*.stories.tsx` exports its states;
  the gallery finds them with `import.meta.glob`, so adding a component edits no shared file.
- **Visual tests**: Playwright photographs every story of the gallery in WebKit (the reference engine) and every
  screen in WebKit and Chromium, with motion off. A changed screenshot is looked at before it is committed.
- **Keyboard and flow tests**: Playwright on the screens, the core mocked as in plan 002.
- **Guards**: the CSS guard, the contrast test, the motion-token mirror test, the import boundaries.
- **Performance**: a Playwright spec run by hand (`npm run test:perf --workspace app/ui`), not in the hooks: it streams
  a 50,000-finding scan through the mocked core and samples frame times; the numbers go into the conclusions.

### Dependencies

| Package        | Licence | Where                | Why                                                                     |
| -------------- | ------- | -------------------- | ----------------------------------------------------------------------- |
| `lucide-react` | ISC     | `app/ui` (production) | consistent, tree-shakeable icons; drawing and maintaining our own costs more |

No animation, component, router, state or CSS library. No font files.

### Tauri changes

- `tauri.conf.json`: `titleBarStyle: "Overlay"`, `hiddenTitle: true`, the traffic lights inset into the sidebar,
  `transparent: true`, `windowEffects: { effects: ["sidebar"], state: "followsWindowActiveState" }`;
  `app.macOSPrivateApi: true`. The exact keys are checked against the pinned Tauri version in step 04/01.
- `app/tauri/Cargo.toml`: the `macos-private-api` feature of `tauri`.
- `capabilities/default.json`: `core:window:allow-start-dragging` and `core:window:allow-internal-toggle-maximize` for
  the drag region (double-click zooms, as in every Mac window); nothing else. The capability test is updated to allow
  exactly these two.

### Parallel work

Plan 05 (data flow) touches no design file and runs in parallel with the foundations. After the foundations, the
primitives, most of the data components and the native window run in parallel; each plan writes its own folders and
its own stories, so the gallery needs no shared edits. The order and the files each plan owns:
[`readme.md`](./readme.md), "Order and parallel work".

## Open questions

None.

## Decisions

- 2026-11-15 — **React stays** (the user).
- 2026-11-15 — The user asked to turn the draft into the full plan and to raise the design: a beautiful dark design
  with animation effects that look really cool (the intent above). The agent takes the recommended choice everywhere
  and lists each under "Agent's calls"; the plan stays 📝 draft until the user has reviewed it.
- 2026-11-15 — **An automated accessibility checker is allowed as a dev-only dependency** (the user):
  `@axe-core/playwright` (MPL-2.0) runs in the e2e tests only and never ships in the app; step 08/02 adds `axe.spec.ts`.
- 2026-11-15 — **The translucent sidebar with Tauri's `macOSPrivateApi` is accepted** (the user): the app is
  distributed outside the Mac App Store.
- 2026-11-15 — **The plan is approved, with every agent's call below confirmed** (the user).

## Agent's calls (to check)

From the draft:

1. **Plain CSS with CSS Modules and three token tiers**; no utility framework, no CSS-in-JS.
2. **Native elements, no headless library**, until a component needs one.
3. **Our own gallery plus Playwright screenshots** instead of a component workshop tool.
4. **No animation library**: CSS transitions and keyframes, the Web Animations API and five small hooks of our own
   (`useMotion`, `usePresence`, `useAnimatedNumber`, `useFlipList`, `useSpotlight`). No effect of the set needs a
   library: springs are replaced by one calm settle curve, and FLIP and the counting numbers are a few dozen lines each.
5. **System fonts; `lucide-react` the only new dependency** (ISC), imported in one allowlist file.
6. **Dark only.**
7. **Our own store and no router**: the screen is a discriminated union.
8. **macOS 13 stays the minimum**, so the baseline is Safari 16.0.
9. **A generic module page** by default; a custom view per module is optional (`src/modules/<id>/`), none in this
   plan.
10. **Accessibility is part of "done" for every component**; the removing button is never the default action.
11. **Target version 0.2.0**, set in the last step.

Made while expanding the draft:

12. **The palette "Graphite and Iris"** as in the tables above: the draft's graphite deepened a step, the accent Iris
    `#7C8CFF` kept for glows with `#8B98FF` for text and focus and a deep Iris fill for the primary button (white text
    at AA); Low moved to cyan `#4CC3E6`, away from the accent; tertiary text `#8A919C`, AA on every surface.
13. **Native translucency for the sidebar** through Tauri's window effect, which needs a transparent window and
    `macOSPrivateApi` (fine outside the Mac App Store, which plan 013 already excludes); a solid fallback everywhere
    else, and in every test.
14. **An overlay title bar with inset traffic lights**; the drag region adds exactly two window permissions to the
    capability (`allow-start-dragging`, `allow-internal-toggle-maximize`).
15. **The Safari 16.0 baseline is held by a CSS guard**: no `color-mix()`, `@property`, nesting, `linear()`,
    `@starting-style`, View Transitions, Popover or `content-visibility`; `-webkit-backdrop-filter` written by hand.
    (The draft said `color-mix()` works on Safari 16; it arrived in 16.2.) No CSS transformer is added for nesting.
16. **Motion tokens, the eleven choreography rules and the animation set** as above; the settle curve only on small
    controls; nothing loops at rest.
17. **The safety screens are static**: no counting, settle, stagger or glow pulse in the confirmation dialog; Cancel
    has the initial focus; permanent deletion is a separate checkbox that turns the button red.
18. **The performance budget**, with two named paint-only exceptions (SVG strokes, hover colours) and one live backdrop
    blur; measured by hand in 08/01 and written in the conclusions, not gated in CI (frame timing on shared runners is
    noise).
19. **No particles and no canvas.**
20. **Two additive core changes**: `Overview.found` and `FindingsQuery.order` (`found`, `largest`), so the hero number
    and the order of the lists come from the core.
21. **The store keeps no findings**: totals from `get_overview`, refreshed at most 4 times a second during a scan;
    lists paged (100 per page, at most 10 pages cached) and virtualised. Closes the known issue.
22. **`VirtualList` of our own**, fixed row heights, list semantics with `aria-setsize` and `aria-posinset`, roving
    focus.
23. **The selection bar's total is a sum of the core's sizes made in the UI**, a preview; the confirmation dialog
    renders only the core's cleanup plan. Until plan 008 the dialog takes a `ConfirmContent` prop type of the UI's,
    which plan 008 fills from its plan (or replaces with a generated type), and the review screen's "Move to Trash…"
    is disabled, recorded in `docs/hacks/`.
24. **The folders `design/`, `product/`, `screens/`, `gallery/`**, with import boundaries enforced by ESLint; `design/`
    takes every text by props.
25. **The gallery is built only in the e2e build mode** (`dist-e2e/`); the app's `dist/` has no gallery.
26. **Gallery stories are photographed in WebKit only**, the screens in WebKit and Chromium: the reference engine for
    every state, half the images.
27. **The motion mode `full | reduced | off`** on one attribute; Playwright sets `off` before the page loads.
28. **Sizes in `rem` on a 13 px root**, so page zoom scales the UI; whether Tauri's zoom hotkeys need a permission is
    checked in 04/01 (if they do, the hotkeys are left for plan 005).
29. **The settings screen in this plan is generic**: one typed `SettingsRow` per setting the core describes, saved
    through `update_setting`; groups, reset and "Rescan to apply" stay in plan 005.
30. **The danger levels' icons escalate in shape**: shield with a check, shield, triangle, octagon — always with the
    word.
31. **The noise texture is a small SVG file** at 3 % opacity, rendered once; off under increased contrast.
32. **A performance spec run by hand** (`npm run test:perf`), outside the hooks and CI.
