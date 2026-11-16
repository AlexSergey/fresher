# 008/07/02 — The confirmation

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** app/ui, core (texts)

## Files

- `app/ui/src/product/confirm-dialog/confirm-dialog.tsx` — `ConfirmContent` (the UI's own type until now) is replaced
  by the generated `CleanupPlanView`; new parts, each from the core's texts:
  - a group's "how" line: "Goes to the Trash", "Emptied; the folder stays", "Removed by Docker — cannot be put back";
  - "These will not be removed" — the left-out items with their reasons, not in the count;
  - for the Trash: "You get the space back when you empty the Trash.";
  - the permanent checkbox's description: "Frees the space now. You cannot put these items back."; hidden when the
    plan has no path item (`permanentAvailable`).
  It stays static (003's safety rule): Cancel first, no key confirms, no counting. `onConfirm(method)` hands the
  method to the screen, which sends the echo of `confirmationOf`.
- `app/ui/src/product/confirm-dialog/*.stories.tsx` — Trash, permanent, a long list, left-out items, a tool group.
- `app/ui/src/screens/review/review-screen.tsx` — "Move to Trash…" enabled: it sends the selection's ids
  (`prepare_cleanup`), shows "Checking each item…" while the core rehearses, opens the dialog with the plan; Cancel
  sends `discard_cleanup`; a refusal (`PlanGone`, `PlanMismatch`, `CleaningBlocked`, `TooManyItems`) is shown with the
  core's text and the way on ("Review it again"). The note "removal comes in a later version" goes.
- `app/ui/src/locales/en.json` — `ui.confirm.*` additions.
- `crates/core/locales/en.json` — the texts of 02/02 reviewed in place (plain English, rule 7).
- `docs/hacks/README.md` — the hack "Review cannot remove yet" removed.

## Produces

The moment of confirmation, showing exactly the core's plan and sending back exactly what it showed.

## Checks

Vitest and Testing Library, negative first:

- "the dialog shows no sum of its own" (the figures are the view's, even when a test feeds a view whose groups do not
  add up), "left out items are listed and not counted", "the permanent choice is off at every opening",
  "no key confirms", "cancel discards the plan", "a refused confirm shows the cores text".
- Positive: "confirm sends the echo of the shown plan", "the permanent choice turns the button red and sends permanent",
  "a tool group says it cannot be put back", "the trash sentence is shown for the trash only".
- The gallery stories render; `npm run test:ui`, lint and coverage pass.

## Conclusions

- Built as written: `ConfirmDialog` takes the generated `CleanupPlanView` (`ConfirmContent` and
  `src/test/confirm-content.ts` are gone); a group per category and way of going, keyed by both, with its "how" line,
  "These will not be removed" with each reason, the Trash sentence and the permanent choice's description. The review
  screen's **Move to Trash…** sends the selection's ids, shows "Checking each item…" (the button busy) while the core
  rehearses, and opens the dialog on the plan; Cancel discards it; a refusal shows the core's text in a notice with
  **Review it again**. The hack "Review cannot remove yet" is removed. Stories: trash, permanent, highest-high,
  long-list, left-out, tool-group.
- Decided where the step was silent, each the simpler or the safer way:
  - **The "how" lines are the UI's chrome texts** (`ui.confirm.how.*`), chosen by the group's `ActionKind` and the
    method: the core's dictionary has no text per action, and a `PlanGroup` does not name its tool, so a tool's line
    reads "Removed by its own tool, such as Docker. It cannot be put back." The core's texts of 02/02 were read and
    left as they are; `crates/core` is frozen.
  - **The "how" line follows the method**: ticking the permanent choice turns "Goes to the Trash." into "Deleted
    permanently." and the lines red, so no line says Trash once the button says delete.
  - **A plan of tool items only** has no permanent choice and no Trash sentence (`permanentAvailable` is false); it
    is titled "Remove 1 item?", and its button is "Remove", red: it cannot be put back.
  - **A left-out item's reason** is the core's `core.cleanup.skip.<kind>` with its instruction where the core has one
    (`product/cleanup-texts.ts`, `tTextIfKnown` in `i18n`): "Quit Safari and try again." for an item in use, the
    removability's instruction for one that cannot be removed.
  - **The dialog is mounted only while the plan is shown**, so every opening is a new dialog: the permanent choice
    is off by construction, besides the reset the component already does.
- Checks: every check of the step (the dialog's in its own test, cancel, no key, the echo, the permanent path and
  the refusals on the review screen), plus "a plan of tool items only offers no permanent choice and no Trash" and
  the texts' own test.
