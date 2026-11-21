# 029/01/04 — The confirmation

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** app/ui

The confirm dialog shows that one item takes several files, and lists them on demand: the user confirms the exact set
(deletion-safety, "Explicit confirmation only"). The design: [`../00-design.md`](../00-design.md), section 4.

## Files

- `app/ui/src/product/confirm-dialog/confirm-dialog.tsx` — an item row with `more` not empty: under its path, a
  disclosure button "and {count} more files that go with it" (`aria-expanded`), opening the list of `more` as
  `PathText` lines. Nothing changes for an item without `more`. No new component: the existing disclosure and
  `PathText` primitives.
- `app/ui/src/locales/en.json` — `confirm.more-files` with `{count}`.
- `app/ui/src/product/confirm-dialog/confirm-dialog.test.tsx` — negative: `shows no more-files line when an item has
  one path`; positive: `names how many more files go with an item`, `lists every other path on demand`.
- `app/ui/src/test/cleanup-plan.ts` — a plan with one together item (a model with two layers).
- `app/ui/src/product/confirm-dialog/confirm-dialog.stories.tsx` — the gallery state with a together item.
- `app/ui/e2e/gallery.spec.ts`, `visual.spec.ts`, `axe.spec.ts` — the new gallery state; the screenshot looked at
  before committing.

No performance run: the row is part of a dialog, not the scan screen, a virtual list or motion
([`../../../testing.md`](../../../testing.md), "Performance").

## Produces

The confirmation of a together item, for 03/03.

## Checks

- `npx vitest run src/product/confirm-dialog`; `npm run test:e2e:ui -- gallery visual axe` in WebKit and Chromium;
  the UI coverage at or above 90 %.

## Conclusions

Built, but the form differs from the design's sketch. The confirm dialog shows an item with other files as a native
button "and N more files that go with it" with `aria-expanded` and `aria-controls` (the UI has no disclosure primitive:
preflight F14). In a plain group the list opens in place under the row; in a virtual group (more than 50 items) the
row keeps its fixed height and the list opens below the virtual list, one item at a time (agent's call, ruling F14).
The list sits inside a `role="listitem"` wrapper (an axe finding in review), the toggle's name carries the item's
title, and an open list scrolls into view. Rows without other files render as before, so no existing baseline moved.
Keys `ui.confirm.more-files.{one,other}`, `ui.confirm.more-files-of` and `ui.confirm.more-list`. Two stories open the
toggles, so the axe and gallery specs cover the expanded state. Review: needed fixes (axe, name, keyboard test), fixed
in round 1 and re-reviewed.
