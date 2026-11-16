# 008/07/04 — The Cleanups screen

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** app/ui

"What did I clean, and can I get it back?"

## Files

- `app/ui/src/state/navigation.ts` — the screen kinds `cleanups` and `cleanup { id }`.
- `app/ui/src/screens/frame-sidebar.tsx` — **Cleanups** after Review.
- `app/ui/src/product/cleanup-row/` (new) — `CleanupRow`: the date, the method, the count, the size, the state ("In the
  Trash: 12 of 128", "Deleted", "Stopped", "Interrupted"); `CleanupItemRow`: the item's title, its category, its
  outcome or skip reason, and **Put back** when it is in the Trash. Stories for every state.
- `app/ui/src/screens/cleanups/` (new):
  - the list (`get_cleanups`, paged, newest first) under "Waiting in the Trash from cleanups: {size}" — with "This is
    what Fresher moved. The Trash may hold more of your own." — and an empty state ("Nothing cleaned yet");
  - one cleanup (`get_cleanup`): its items (a virtual list past 50, as the dialog does), **Put back all still in the
    Trash**, **Show in the Trash**; a put back answers inline per item ("Put back", "Something new is in its place —
    it stays in the Trash", "No longer in the Trash");
  - an interrupted cleanup explains: "Fresher stopped during this cleanup. Items marked Unknown may be in the Trash."
- `app/ui/src/locales/en.json` — `ui.screen.cleanups.*`.

## Produces

The history and the undo of the product's most serious action.

## Checks

Vitest, negative first:

- "an empty history says nothing cleaned yet", "put back is offered only for items in the trash",
  "a permanent cleanup offers no put back", "an occupied place is explained and the item stays",
  "a refused put back shows the cores text", "a read only journal shows its reason".
- Positive: "the list pages newest first", "put back all sends the cleanup without items",
  "a put back updates its row", "show in the trash asks the core".

## Conclusions

- Built as written: the screen kinds `cleanups` and `cleanup { cleanup }`, **Cleanups** in the sidebar after Review
  (the `History` icon added to the allowlist), `CleanupRow` and `CleanupItemRow` with stories for every state, the
  list (`screens/cleanups/cleanups-screen.tsx`, `use-cleanups.ts`) and one cleanup (`cleanup-screen.tsx`,
  `use-cleanup.ts`); the result's "See this cleanup" opens it.
- Decided where the step was silent, each the simpler or the safer way:
  - **Paged by "Show older cleanups"**, 50 at a time, appended; the history is read again each time the screen opens,
    so a cleanup that just ended is listed.
  - **An item is named by the last name of its path** (a tool's item by its category): `CleanupItemView` has no
    title. Its second line is its category and what became of it — where it is now for an item moved to the Trash
    (`core.cleanup.trash.*`), its outcome or skip reason otherwise, the last put back's text when there is one.
  - **A row's state**: Interrupted, Stopped (cancelled or stopped), Deleted, "In the Trash: n of m", or "No longer in
    the Trash" once nothing of it is there.
  - **A put back answers inline** with the core's text per item (`PutBackItem.text`), and the cleanup is read again;
    the row's **Put back** goes once it answered. A refused command (`NotInTrash`, `UnknownCleanupItem`) is a notice
    with the core's text; an unreachable core is the frame's error.
  - **Show in the Trash** on one cleanup reveals its first item still in the Trash; with none, neither it nor "Put
    back all" is offered.
  - **The history's dates are dates only** (the core's `date` argument, `dateStyle: medium`); the fixtures sit at noon
    UTC so a screenshot shows the same day in any time zone.
- Checks: every check of the step, plus "an item a crash left unknown says so", "a skipped item shows why, without
  the instruction it did not keep", "a cleanup with nothing in the Trash offers no put back", "a long cleanup lists
  its items in a virtual list" and "All cleanups goes back to the list".
