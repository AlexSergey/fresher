# The confirmation of a long cleanup hides the choice of method

**Status:** fixed (the list takes the height the dialog leaves, one scroll only) · **Updated:** 2026-11-26

## Symptom

Found in the sandbox run by hand before 0.1.0 ([`../test-cases/sandbox-cleanup.md`](../test-cases/sandbox-cleanup.md),
SC-10, a cleanup of 300 items): the confirmation dialog shows two scroll bars, one on its list of items and one on the
dialog itself, and the "delete permanently" choice is below the fold until the dialog is scrolled. The user may
confirm without seeing how the items will be removed.

## Cause

The long group's list had a fixed height of 320 px, and the dialog's body (header and footer fixed, the body between
them scrolling) is shorter than the content at the window's minimum, 900 x 600. The list scrolled inside its own box
and the body scrolled as well, so the choice of method, last in the content, was below the fold.

## Fix

The content now fills the height of the body as a column: the group with a long list takes what the rest leaves
(`flex: 1 1 320px`, at least 96 px) and everything else keeps its size, so the list is the only scroll region and the
heading, the choice of method and the buttons stay in view. The shared dialog body gives its content the body's height
for this. Checked by a unit test (structure and layout rules) and an end-to-end case at 900 x 600 in Chromium and
WebKit.
