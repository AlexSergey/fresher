# 003/07/04 — Review

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `app/ui/src/screens/review/review-screen.tsx` — answers "What exactly am I about to remove?":
  - `Toolbar` with the title and **Clear selection**;
  - the summary at the top: the count, the total size and the highest level as a large `DangerBadge` with its
    sentence;
  - the selection grouped by category, largest first: each group's heading with the category's name, level, size and
    its "if removed" sentence; under it its findings (a `FindingList` over the selected findings of that category, or
    over the whole category minus its exclusions — `useFindings` with the category, rows the user ticked off shown
    unticked), each row can be unticked here;
  - **Move to Trash…** (primary): disabled until plan 008 wires the cleanup, with a plain line under it saying removal
    comes in a later version; its `ConfirmDialog` and `ResultSummary` are complete in the gallery.
  - an `EmptyState` when nothing is selected, with a way back to the overview.
- `docs/hacks/README.md` — "Review cannot remove yet": what (the button disabled), where, why (no cleanup command until
  plan 008), if cleaned up (enabling it without a core plan would show the dialog with the UI's own sums).
- `locales/en.json` — `ui.screen.review.*`.

## Produces

The last screen before the confirmation; plan 008 enables its button and feeds `ConfirmDialog` from the core's plan.

## Checks

- Negative: `an empty selection shows the empty state`, `the remove button is disabled and says why`, `unticking the
  last item of a group removes the group`, `the highest level never shows by colour alone`.
- Positive: `groups follow size order and show what is lost`, `the summary matches the selection bar's`, `unticking a
  row updates the summary`.
- `npx vitest run src/screens/review` passes.

## Conclusions

Done on 2026-11-16. `ReviewScreen` replaces the placeholder in `app/ui/src/screens/review/` (the screen, its groups in
`review-groups.ts` and `review-group.tsx`, its styles), with tests (negative first) and a look at WebKit screenshots of
the empty, selected and unticked states. No new dependency, token or icon; 12 `ui.screen.review.*` keys; the hack
"Review cannot remove yet" is in `docs/hacks/README.md`.

Decided while building (agent's calls, to check):

- **`Picked` is the finding itself** (`state/selection.ts`): a finding picked one by one must be listed here with its
  title and path, and no command fetches a finding by id. Only the findings the user ticked one by one (and the ones
  ticked off inside a selected category) are kept, never a page, so the store still holds no list of findings. The
  selection's tests build their picks with `findingOf`.
- **The toolbar** holds only **Clear selection** (shown while something is selected); the title is the screen's one
  `h1` in a `PageHeader` under it, as the other screens have it, with a one-line subtitle.
- **Groups** come from `reviewGroups(selection, totals)`: each category's part of the selection summed with the store's
  `summarize` (so the summary, the groups and the selection bar agree), largest first, ties by category id; a group
  with nothing left (every finding ticked off, or the last picked one unticked) is not shown, and with nothing left at
  all the screen shows its empty state.
- **A selected category** lists all its findings (`useFindings` with the category, no module, largest first), the ones
  ticked off inside it unticked; unticking toggles its exclusion. As a shown group is never empty, a total of 0 with
  no failure is passed to `FindingList` as `undefined` (its first page has not answered). **Findings picked one by
  one** are listed from the selection, largest first, all ticked; unticking one takes it off the list.
- Each list is as tall as its rows, at most six (`--review-rows`), and scrolls past them.
- **The summary**: the count, the total (`SizeText` at display size, static), the highest level as a large
  `DangerBadge` with its sentence (preceded by a hidden "Highest danger level:"); **Move to Trash…** is disabled with
  its note beside it. The screen renders no `ConfirmDialog` until plan 008 (nothing could open it); its tests check
  that the button opens no dialog and that Escape or Enter sends the core nothing but reads.
- The screen renders no `SelectionBar`: its summary panel already says what the bar would.
