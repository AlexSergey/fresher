# 042/02/01 — Tokens, the button's sheen and the menu's hover

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

Every colour of the mockups becomes a token; the primary button gains the swoosh's sheen; a menu's pointed item fills
with the accent colour. Mockups: [`scan.dc.html`](../../../mockups/0.1.0/scan.dc.html) (the Review link of the bar:
the sheen), [`row-menu.dc.html`](../../../mockups/0.1.0/row-menu.dc.html) (the hover), and every other file for the
colours below.

## Read first

`app/ui/src/design/tokens/{primitive.css,semantic.css,contrast.test.ts}`, `design/primitives/button/` (the primary
variant), `design/primitives/menu/menu.module.css`, the CSS guard (`rg "raw colour|#[0-9a-f]{3,6}" scripts eslint*`
to find the rule that keeps raw colours out of components), `app/ui/e2e/contrast.spec.ts`.

## Files

- `primitive.css` — add exactly: `--blue-400: #5b7cfa; --violet-400: #b18cf5; --pink-400: #f07ab0; --iris-200:
  #9aa6ff; --iris-200-a0: rgb(154 166 255 / 0%); --iris-200-a25: rgb(154 166 255 / 25%); --iris-200-a45: rgb(154 166
  255 / 45%); --iris-600-a35: rgb(84 101 235 / 35%); --night-800: #1d2140; --gray-875: #1a1d23; --gray-650: #3a414d;
  --gray-200: #c9ced6; --gray-700-a0: rgb(49 55 65 / 0%); --green-900: #2b4a3b; --cyan-900: #26404a; --amber-900:
  #5a4620; --amber-925: #2e2616; --amber-950: #1e1b14; --amber-975: #17150f; --amber-200: #d9c79f; --green-400-a8:
  rgb(61 214 140 / 8%); --green-400-a12: rgb(61 214 140 / 12%); --cyan-400-a8: rgb(76 195 230 / 8%); --cyan-400-a12:
  rgb(76 195 230 / 12%); --amber-400-a10: rgb(245 181 68 / 10%); --amber-400-a14: rgb(245 181 68 / 14%);
  --red-400-a12: rgb(255 107 112 / 12%); --white-a3: rgb(255 255 255 / 3%); --floating-a92: rgb(28 31 37 / 92%)`.
- `semantic.css` — add:
  - `--surface-card: radial-gradient(120% 140% at 20% 0%, var(--night-800) 0%, var(--gray-900) 55%)`;
    `--surface-tile: linear-gradient(180deg, var(--gray-875), var(--gray-900))`; `--surface-tile-medium:
    linear-gradient(180deg, var(--amber-950), var(--amber-975))`; `--surface-floating: var(--floating-a92)`;
    `--surface-row-hover: var(--white-a3)`.
  - `--tile-border: var(--gray-750)`, `--tile-border-hover: var(--gray-650)`, `--tile-border-safe: var(--green-900)`,
    `--tile-border-low: var(--cyan-900)`, `--tile-border-medium: var(--amber-900)`, `--tile-border-high:
    var(--red-400-a32)`; `--tile-divider-medium: var(--amber-925)`; `--tile-line-medium: var(--amber-200)`.
  - `--level-safe-chip: var(--green-400-a8)`, `--level-low-chip: var(--cyan-400-a8)`, `--level-medium-chip:
    var(--amber-400-a10)`, `--level-high-chip: var(--red-400-a12)`; `--level-safe-icon-bg: var(--green-400-a12)`,
    `--level-low-icon-bg: var(--cyan-400-a12)`, `--level-medium-icon-bg: var(--amber-400-a14)`,
    `--level-high-icon-bg: var(--red-400-a12)`.
  - `--part-can-be-freed: var(--green-400)`, `--part-files-and-apps: var(--blue-400)`, `--part-mac-os:
    var(--violet-400)` (was plum: the other parts keep theirs), `--part-system-space: var(--pink-400)`;
    `--text-legend: var(--gray-200)`.
  - `--separator-line: linear-gradient(90deg, var(--gray-700), var(--gray-700-a0))`.
  - `--swoosh-start: var(--iris-700)`, `--swoosh-end: var(--iris-200)`, `--swoosh-track: var(--gray-750)`.
  - `--accent-sheen: linear-gradient(115deg, var(--iris-200-a0) 45%, var(--iris-200-a45) 100%), var(--accent-fill)`;
    `--accent-sheen-shadow: inset 0 0 0 1px var(--iris-200-a25), 0 4px 14px var(--iris-600-a35)`.
  - `--menu-item-active: var(--iris-700)`, `--menu-item-active-text: var(--text-on-fill)`.
- `design/primitives/button/button.module.css` — the primary variant: `background: var(--accent-sheen); box-shadow:
  var(--accent-sheen-shadow)`; hover and pressed keep their present change (a lighter or darker fill) over the sheen;
  disabled as now; the focus ring as now. Text stays `--text-on-fill`.
- `design/primitives/menu/menu.module.css` — `.item:hover, .item:focus` (and the active item the component marks):
  `background-color: var(--menu-item-active); color: var(--menu-item-active-text)`; its `.description` on the active
  item `color: var(--menu-item-active-text)`; the focus-visible ring stays.
- `contrast.test.ts` — new pairs at 4.5:1 or more: `--text-on-fill` on `--menu-item-active` and on `--accent-fill-top`;
  `--text-legend` on `--gray-900`; `--tile-line-medium` on `--amber-950`; `--text-tertiary` on `--gray-875`.

## Tests

Negative first in `contrast.test.ts` (a pair below the bar fails, as its existing negative cases do); positive the new
pairs. Button and menu unit tests unchanged (no behaviour change); their stories show the new look.

## Checks

`npx vitest run src/design`; `npm run lint:ts`, `npm run lint:code`, the CSS guard; `npx playwright test --project
gallery --project webkit --project chromium --update-snapshots` (every baseline with a primary button or an open menu
changes; look at each changed PNG, and at a sample of the unchanged); `npx playwright test e2e/contrast.spec.ts`.
Known flaky: chromium `category-details-app-leftovers-chosen` (rerun once).

Commit: `feat(ui): the mockups' colours as tokens, the button's sheen, the menu's hover`.

## Conclusions

- The mockups' colours as tokens; the mockup's `#313741` is `--border-solid` (the name `--border-strong` was taken),
  with `--border-divider`; the primary button's sheen (no shadow under increased contrast) and the menu's accent
  hover. White on the sheen's far corner is about 4.35:1; the label sits where it is above 4.5:1.
- Unused tokens were removed in the final review.
