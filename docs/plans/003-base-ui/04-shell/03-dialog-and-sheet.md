# 003/04/03 — Dialog and sheet

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `design/layout/dialog/` — `Dialog`: a native `<dialog>` opened with `showModal()` (focus trap, Escape, the rest of
  the page inert), `open`, `onClose`, `title` (its accessible name through `aria-labelledby`), `description`
  (`aria-describedby`), `initialFocus` (a ref; the first focusable element otherwise), `closeLabel` for its close
  `IconButton`, `footer` for the buttons. On close the focus returns to the element that opened it.
  - Look: elevation 3, `--radius-lg`, at most 560 px wide (720 for `size="large"`), the body scrolls inside.
  - Backdrop: `::backdrop` black at 55 % with `-webkit-backdrop-filter` and `backdrop-filter: blur(16px)
    saturate(140%)` — the app's one live backdrop blur.
  - Motion: the backdrop fades in; the panel fades and scales from `--scale-enter` with `--duration-base` and
    `--ease-standard`, once; the exit fades over `--duration-fast` through `usePresence`, then `close()`. Never the
    settle curve: this dialog carries the confirmation.
  - Enter is never bound to an action by the dialog; a form's submit is the caller's choice.
- `design/layout/sheet/` — `Sheet`: the same native dialog, attached to the bottom edge of the main pane, full width
  of the content column, rising `--distance-3` on enter; for longer secondary content (details of a finding later).

## Produces

`Dialog` for `ConfirmDialog` (06/04) and every later modal; `Sheet` for detail views.

## Checks

- Negative: `Escape closes and calls onClose once`, `a click inside the panel does not close`, `Enter does not
  trigger the footer's buttons unless one has the focus`, `the page behind is inert while open`, `motion off closes
  without waiting for an animation`.
- Positive: `the title names the dialog`, `the initial focus goes to the given element`, `the focus returns to the
  opener on close`, `a click on the backdrop closes` (only when `dismissible`, the default for `Sheet`; `Dialog`
  defaults to not dismissible by backdrop), `the exit animation runs before close`.
- Stories: dialog open with a short body and with a long scrolling body; sheet open; screenshots looked at (WebKit
  draws the backdrop blur).
- The CSS guard passes (the prefixed pair is present).

## Conclusions

- Built as planned: `Dialog` and `Sheet` are thin wrappers of one internal `Modal` (`dialog/modal.tsx`), a native
  `<dialog>` opened with `showModal()` on mount and closed with `close()` on unmount, after `usePresence` has played the
  exit (a fade over `--duration-fast`); the focus then returns to the element focused when it opened. The owner closes
  it: Escape (`cancel`, its default prevented), the close button and, when `dismissible`, a click on the backdrop call
  `onClose` — once: a request while the exit plays is ignored. `Dialog` defaults to not dismissible, `Sheet` to
  dismissible. No key is bound to the footer.
- **Initial focus** without `initialFocus` is the first focusable element, which is the close button in the header.
  `Button` takes no `ref` yet, so `ConfirmDialog` (06/04) gives `Button` a `ref` prop to focus Cancel.
- **The backdrop** is `--surface-backdrop` (a new semantic token, `--black-a55`, a new primitive) with the prefixed
  blur pair. `::backdrop` inherits no custom property before Safari 17.4, so the primitive and semantic token blocks
  are also defined on `::backdrop` (`:root, ::backdrop`), the backdrop's durations and easings carry their values as
  fallbacks, and motion off stops its animation in the module (the motion utilities do not reach a `::backdrop`).
- **The sheet** is placed against the viewport: margins put it after the sidebar's 232 px and inside the content
  column (`max(gutter, (100vw − sidebar − 1,120 px) / 2)`, 24 px gutters under 1,000 px), attached to the bottom edge,
  at most 70 % of the window's height, rising `--distance-3` with `--ease-enter`.
- **Screenshots:** Playwright's WebKit does not draw `backdrop-filter` in its screenshots at all (checked on a plain
  page with a blurred overlay), so the stories show the dim but not the blur; the blur is a check by hand in 08.
- **E2E** (`e2e/layout.spec.ts`, WebKit and Chromium): Tab never reaches a button behind the open dialog; the focus
  starts on the close button and moves through the dialog. A native modal does not cycle the focus: past its last
  control the focus leaves the page (to the browser's chrome), never to the inert page behind.
