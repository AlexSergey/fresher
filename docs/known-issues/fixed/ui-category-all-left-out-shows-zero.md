# A category whose items are all left out shows "0 bytes / 0 items"

**Status:** fixed (option (a): the header hides the totals when nothing is offered and something is left out) ·
**Updated:** 2026-11-26

## Symptom

When every item of a category is left out, for example Xcode build data while Xcode is open, the category's row shows
"0 bytes / 0 items" beside its left-out line. The user reads it as "nothing here" although the line says why items were
left out. The visual screenshot `module-page-dev-caches` shows it.

## Cause

The totals count only offered items, and the UI renders them even when they are zero. This is existing UI behaviour; the
developer caches module (plan 018) makes it common, as Xcode is often open.

## What was tried

Nothing before the fix: the change was deferred.

## Options left

- (a) Hide the totals when the category has no offered item and show only the left-out line. Chosen by the user.
- (b) Show the left-out size in place of the zero.

## Fix

Option (a). A category's details (`app/ui/src/screens/category/category-screen.tsx`) show the name and the group but no
size and no item count in the header when the category has no offered item and at least one left-out count; the note
under it says "Everything found here was left out. The lines below say why.", and the left-out lines, folded, sit at
the bottom of the screen. A category with neither findings nor left-outs keeps its empty state. Held by
`category-screen.test.tsx` and the developer caches flow in `e2e/flows.spec.ts`.
