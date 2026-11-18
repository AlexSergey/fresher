# A category whose items are all left out shows "0 bytes / 0 items"

**Status:** open · **Updated:** 2026-11-18

## Symptom

When every item of a category is left out, for example Xcode build data while Xcode is open, the category's row shows
"0 bytes / 0 items" beside its left-out line. The user reads it as "nothing here" although the line says why items were
left out. The visual screenshot `module-page-dev-caches` shows it.

## Cause

The totals count only offered items, and the UI renders them even when they are zero. This is existing UI behaviour; the
developer caches module (plan 018) makes it common, as Xcode is often open.

## What was tried

Nothing: the change is deferred.

## Options left

- Hide the totals when the category has no offered item and show only the left-out line.
- Show the left-out size in place of the zero.
