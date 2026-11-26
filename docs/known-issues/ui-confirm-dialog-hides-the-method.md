# The confirmation of a long cleanup hides the choice of method

**Status:** open · **Updated:** 2026-11-26

## Symptom

Found in the sandbox run by hand before 0.1.0 ([`../test-cases/sandbox-cleanup.md`](../test-cases/sandbox-cleanup.md),
SC-10, a cleanup of 300 items): the confirmation dialog shows two scroll bars, one on its list of items and one on the
dialog itself, and the "delete permanently" choice is below the fold until the dialog is scrolled. The user may
confirm without seeing how the items will be removed.

## Cause

Not investigated. The dialog is `app/ui/src/product/confirm-dialog/confirm-dialog.tsx`: its list of items grows with
the plan, and the dialog's own body scrolls as well, so the choice of method moves out of sight.

## What was tried

Nothing yet.

## Options left

- One scroll only: the list scrolls inside a bounded height, the method and the buttons always visible.
- Put the method next to the buttons, above any list.
