# 003/02/01 — Buttons

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `design/primitives/button/` — `Button`: `variant: 'primary' | 'secondary' | 'ghost' | 'danger'`, `size: 'small' |
  'medium' | 'large'` (24 / 28 / 36 px), `busy` (a `Spinner` replaces the leading icon, the label stays, the button is
  `aria-disabled` and announces "busy" through `aria-busy`), optional leading `icon`. A real `<button>`, `type="button"`
  unless given.
  - Primary: the Iris fill gradient, a top highlight hairline, white text; its hover glow is a `::after` holding
    `--glow-accent`, faded in by opacity (`--duration-fast`), never an animated shadow.
  - Secondary: raised surface, `--border-default`, primary text. Ghost: no fill until hover. Danger: the danger fill —
    used only by the permanent-delete choice.
  - Press: `transform: scale(var(--scale-press))` for `--duration-press`; disabled: no hover, no press,
    `--text-disabled`.
- `design/primitives/icon-button/` — `IconButton`: a square button with one `Icon`, a required `label` (its accessible
  name and the text of its `Tooltip` from 02/04), `variant: 'ghost' | 'secondary'`.

## Produces

`Button` and `IconButton` for every later component and screen.

## Checks

- Negative: `does not fire when disabled`, `does not fire when busy`, `is not a submit button by default`, `an
  icon button without a label is a type error` (`// @ts-expect-error`), `the danger variant has no glow`.
- Positive: `fires on click, Enter and Space`, `shows the spinner and keeps its label when busy`, `has its label as its
  accessible name`, `the primary glow fades in on hover` (the class and the token; motion `off` makes it instant).
- Stories: every variant × size × rest, hover (`data-hover` forces the state), pressed, focus, disabled, busy;
  screenshots looked at.
