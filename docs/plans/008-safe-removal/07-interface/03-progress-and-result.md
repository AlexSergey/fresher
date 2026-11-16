# 008/07/03 — Progress and the result

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `app/ui/src/product/cleanup-progress/` (new) — `CleanupProgress`: "Moving 12 of 128 to the Trash…" (or "Deleting…"),
  a plain determinate bar (the `UsageBar` of plan 003 if it fits, with no motion), the title of the item in hand, and
  **Stop** (`cancel_cleanup`), then "Stopping after this item…". Static by the safety rule: no counting animation, no
  glow, no stagger. Its stories: early, late, stopping.
- `app/ui/src/product/result-summary/result-summary.tsx` — the props from the core's `CleanupReport`:
  - a Trash cleanup: the title "Moved to the Trash", the number the size moved, then "You get the space back when you
    empty the Trash." and "Space freed on the disk now: about {size} (measured)" — or "could not be measured";
  - a permanent cleanup: "Freed", the measured space next to the estimate;
  - a cancelled one says so: "Stopped. {n} items were not touched.";
  - the skipped and failed items with their reasons (the core's texts); **Show in the Trash** (`reveal_in_trash` on
    the first moved item), "See this cleanup" (opens it in the Cleanups screen, 04), **Done**.
  The cleaned moment of 003 (the check mark, the number counting up, one light wave) plays once, after the removal is
  over. Stories: moved, deleted, some skipped, stopped, not measured.
- `app/ui/src/screens/review/review-screen.tsx` — the review screen shows the progress while `running` and the result
  when `finished`; **Done** returns to the overview.
- `app/ui/src/screens/app-frame.tsx` — the quit question (plan 003's `Dialog`): "A cleanup is running. Stop after the
  current item and quit?", **Keep cleaning** first, **Stop and quit** (`quit_after_cleanup`).
- `app/ui/src/locales/en.json` — `ui.cleanup.*`, `ui.result.*` additions.

## Produces

What the user sees between the confirmation and the end, and the honest result.

## Checks

Vitest, negative first:

- "the progress has no motion" (no animated number, no glow class), "stop is sent once and says stopping", "a trash
  result never says freed for the trash", "an unmeasured space says so", "a stopped cleanup says how many were
  untouched", "keep cleaning has the first focus and changes nothing".
- Positive: "progress follows the items", "the result lists skipped items with their reasons",
  "show in the trash asks the core", "stop and quit asks the core".

## Conclusions

- Built as written: `CleanupProgress` (new, stories early, late, stopping and deleting), `ResultSummary` on the core's
  `CleanupReport` (stories moved, deleted, some-skipped, stopped, not-measured), the review screen's progress and
  result (`screens/review/cleanup-panel.tsx`), and the quit question in the frame (`screens/quit-question.tsx`).
- Decided where the step was silent, each the simpler or the safer way:
  - **The bar is plan 003's `ProgressBar`** (determinate), its fill's transition switched off in the component's own
    stylesheet: the fill jumps. "Moving 12 of 128" counts the item in hand (done + 1).
  - **The figure under the title is the report's estimate** (`moved` for the Trash, `deleted` for a permanent
    cleanup), the measured space a sentence under it: "Space freed on the disk now: about 100 MB (measured)." for the
    Trash, "Measured on the disk: about 2.4 GB." for a permanent cleanup, or "… could not be measured."
  - **The count line adds emptied folders to the moved or deleted ones**; items a tool removed get their own line; a
    cleanup stopped because the history could not be saved shows the core's `core.cleanup.ended.stopped`.
  - **The list is "N items were not removed"**: skipped and failed items, from the `cleanup-items` events with the
    confirmed plan's titles; a title the UI does not know reads "An item of this cleanup".
  - **Show in the Trash reveals the first item moved**, its index kept by the state (`firstMoved`); a refusal (the
    core answers `internal` until plan 006/03) is the frame's toast with the core's text.
  - **"See this cleanup" is wired in step 04**, where the Cleanups screen exists.
  - **The quit question**: Keep cleaning first and focused, Escape and the close button keep cleaning; a cleanup that
    ends while the question is open closes it (the close is no longer held, the user closes again).
  - **From the engine's fix merged later (75c1ddd)**: a plan whose items were all left out has no confirm button —
    the dialog says "Nothing here can be removed", lists the reasons, and offers only "Back to the review" (story
    `ConfirmDialog/all-left-out`).
- Checks: every check of the step, plus "a cleanup stopped by the history says the core's text", "items a tool
  removed are counted apart", "show in the trash asks the core for the first moved item" and the review screen's whole
  run from the confirm to Done.
