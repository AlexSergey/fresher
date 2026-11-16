# 003/02 — Primitives

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

The small controls every screen is built from, each a native element styled with the tokens, in every state of the
design's component table, with its micro-interactions: press, hover glow, the drawn check mark, the sliding thumb.
The design: [`../00-design.md`](../00-design.md), "Components", "Motion".

Depends on plan 01. Every component lives in `app/ui/src/design/primitives/<name>/` with its `.tsx`, `.module.css`,
`.test.tsx` and `.stories.tsx`; every text comes in by props. The steps write separate folders: 02 and 04 run in
parallel first, then 01 (its busy state uses `Spinner`, its icon button `Tooltip`), then 03 (the search field's clear
button is an `IconButton`).

## Steps

| #   | Step                                               | Status   | Depends on | Shared files                                       |
| --- | -------------------------------------------------- | -------- | ---------- | -------------------------------------------------- |
| 01  | [Buttons](./01-buttons.md)                         | ✔️ done    | 04         | `design/icons/icons.ts` (new names only)           |
| 02  | [Toggles](./02-toggles.md)                         | ✔️ done    | —          | —                                                  |
| 03  | [Fields](./03-fields.md)                           | ✔️ done    | 01         | `design/icons/icons.ts` (new names only)           |
| 04  | [Indicators](./04-indicators.md)                   | ✔️ done    | —          | —                                                  |

Every step: unit tests negative first; the accessible name asserted; keyboard use tested; `reduced` and `off` motion
tested where the component moves; its stories photographed and looked at.

## Conclusions

Done on 2026-11-15, one commit per step, in the order 02, 04, 01, 03.

### What was built

Fifteen components in `design/primitives/`: `Checkbox`, `Switch`, `SegmentedControl`, `Tooltip`, `Badge`, `Spinner`,
`ProgressBar`, `Skeleton`, `Divider`, `VisuallyHidden`, `Button`, `IconButton`, `TextField`, `SearchField`, `Select`.
106 unit tests (UI coverage after the plan: 99.0 % lines, 94.8 % branches, 97.3 % functions), 86 gallery stories
photographed in WebKit and looked at, and a new keyboard spec in WebKit and Chromium. No icon was added: the fields
use `CircleAlert`, `Search`, `X` and `ChevronDown` from the allowlist.

### Decided while building

- **Forced states for the screenshots.** A screenshot cannot hover, press or focus, so every primitive passes
  `data-hover`, `data-pressed` and `data-focus` (the type `Forced`, `design/primitives/forced.ts`) to its native
  element, and its stylesheet draws them as `:hover`, `:active` and `:focus-visible`. The plan named `data-hover`; the
  other two follow it. The app never sets them.
- **Every control has a name.** `Checkbox`, `Switch`, `TextField`, `SearchField` and `Select` take a required `label`
  and a `hideLabel` that hides it visually (a row or a toolbar whose title names the control), instead of an optional
  label: a control without a name is a type error.
- **The segmented control has equal segments**, as macOS draws them, so its indicator is placed by CSS from the chosen
  index (`translateX` by its own width, `--segment-index` and `--segment-count`), not measured and scaled: nothing to
  read from the layout, nothing stale after a page zoom, and no `scaleX` stretching its corners.
- **The tooltip takes a render function**, `(describedBy) => element`, rather than cloning its child: the link is
  explicit and the child keeps its own props. `defaultOpen` shows it from the start (the gallery uses it);
  `repeatsName` keeps it from describing a child it already names, so an icon button's label is not read twice. Its
  placing is a pure function (`place.ts`), tested without a layout. A focus that a pointer press caused does not show
  it at once; any other focus does (keyboard, or a script moving the focus).
- **The keyboard of native controls is tested in the engines.** happy-dom does not turn Space or Enter into a click,
  so `e2e/primitives.spec.ts` (a new spec, in the `webkit` and `chromium` projects) presses them on the gallery's
  stories: the checkbox, the switch, the button, the icon button's tooltip on focus, the tooltip's Escape and hover,
  the segmented control's tab stop. The unit tests cover everything the components do in JavaScript.
- **Motion is checked in the stylesheets.** As in plan 01, the unit tests read a module's CSS with `node:fs` and
  assert the transition, its token and the reduced rule. What travels (the switch's thumb, the segmented indicator,
  the progress fill) has `transition: none` under reduced motion (`:root[data-motion='reduced']` and the media query),
  colour changes stay; the press scale is already 1 there. Loops (the spinner, the indeterminate sweep, the skeleton's
  shimmer) run on `animation-play-state: var(--loop-state)`; the spinner turns once per `--duration-deliberate`, the
  skeleton keeps its own `--skeleton-loop: 1400ms`. An indeterminate progress bar has a faint Iris tint, so it still
  reads as working when its sweep stands still.
- **A busy button** is `aria-busy` and `aria-disabled` (it keeps the focus), ignores clicks and prevents a busy submit
  button from submitting again; a disabled one is natively `disabled`. Its focus ring is redeclared after the variants,
  whose own shadows (the top hairline) would otherwise hide the base ring.
- **Shared boxes.** `IconButton` reuses `button.module.css` (made square by its own sheet), `SearchField` and `Select`
  reuse `text-field.module.css`. A story that needs layout has its own `*.stories.module.css`.
- **Small calls**: the count badge takes its full accessible text (`label`, "1,204 items") because `design/` formats no
  number; badges use `--radius-xs` as the design says; a labelled spinner is an image with its label; Escape on an
  empty search is left to whatever else listens (a dialog closing); a select ignores a change while disabled and a
  value that is not one of its options; the invalid field's border and message use `--level-high-fg`.

### Learned

- WebKit treats a focus moved by script after load as `:focus-visible`, so the icon button's tooltip story shows its
  real focus ring without a forced attribute.
- A CSS module's class is `string | undefined` under `noUncheckedIndexedAccess`; a component prop that takes a class
  (`Icon`'s `className`) needs `?? ''`.
