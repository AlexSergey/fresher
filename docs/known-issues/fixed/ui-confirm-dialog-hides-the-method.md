# The confirmation of a long cleanup hides the choice of method

**Status:** fixed (the choice of method and the High tick sit with the buttons, out of the scroll) · **Updated:** 2026-11-26

## Symptom

Found in the sandbox run by hand before 0.1.0 ([`../../test-cases/sandbox-cleanup.md`](../../test-cases/sandbox-cleanup.md),
SC-10, a cleanup of 300 items): the confirmation dialog shows two scroll bars, one on its list of items and one on the
dialog itself, and the "delete permanently" choice is below the fold until the dialog is scrolled. The user may
confirm without seeing how the items will be removed.

## Cause

The High tick and the choice of method were the last part of the dialog's body, the region between the fixed header
and footer that scrolls. At the window's minimum, 900 x 600, any plan longer than the body (a long group's virtual
list of a fixed 320 px, but also a plain list of some 30 items, or several short groups) pushed them below the fold.
A long group scrolled twice: its virtual list inside its own box, and the body around it.

## Fix

The High tick and the choice of method (with its line on when the space comes back) moved from the body into the
dialog's footer, above the buttons (`confirm-dialog.tsx`, `.actions`): the footer does not scroll, so for any plan
(one item, a plain list, several categories, a virtual list of hundreds) they and the buttons are always in sight,
and the items scroll in the one region above them. A long group's virtual list keeps its own scroller, so for it the
body does not scroll: the content fills the body's height as a column and the group takes what the rest leaves
(`flex: 1 1 320px`, at least 96 px). For this the shared dialog body gives its content the body's height, and it is
positioned, so a visually hidden text in a scrolled-out row is clipped by the body instead of scrolling the dialog.

Checked by a unit test (the tick and the choice in the footer, out of the list) and an end-to-end case at 900 x 600
in Chromium and WebKit for a long plan, a plan of 30 items and a plan of three categories with the High tick: the
ticks and the confirm button wholly in view, and one scrolling region in the dialog.

## Left as it was

- A long plan (a virtual list) with the High tick, at the window's minimum: the footer with both ticks is taller, so
  the body may be too low for the group's heading and the list's 96 px, and it scrolls as well as the list. The ticks
  and the buttons stay in sight; only the two scroll bars come back, in this case only. No end-to-end case covers it.
