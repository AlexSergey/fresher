# 008/07/05 — Flows and screenshots

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `app/ui/e2e/core-mock.ts` — the cleanup commands and events: a plan view per selection, a cleanup streamed item by
  item and held until released (like the held scan), refusals on demand (`PlanGone`, `PlanMismatch`,
  `CleaningBlocked`), a history and put back; `core-mock.spec.ts` covers the new paging.
- `app/ui/e2e/flows.spec.ts` — the flows, each checking the CSP refused nothing:
  1. select → review → confirm (Trash) → progress → result → Cleanups → put back one item;
  2. the permanent path (the checkbox, the red button, "Freed");
  3. Stop mid-cleanup → the stopped result;
  4. a refused echo and an expired plan → the text and "Review it again";
  5. left-out items in the dialog;
  6. closing the window during a cleanup → the question → Keep cleaning.
- `app/ui/e2e/visual.spec.ts`, `gallery.spec.ts` — screenshots of every new state and story (WebKit; the screens also
  in Chromium); `axe.spec.ts`, `keyboard.spec.ts`, `zoom.spec.ts` — the new screens and the dialog's new parts.
- `app/ui/e2e/screenshots/` — the new images, each looked at before the commit.

## Produces

The interface proved on every path a user can take through a cleanup.

## Checks

- `npm run test:e2e:ui` passes in WebKit and Chromium; no WCAG 2.1 A or AA violation; every focus stop reached with a
  visible ring; the dialog keeps and returns the focus; nothing cut at 125 % and 150 %.
- `npm run pre-push` passes.

## Conclusions

- Built as written. The mocked core's cleanup part is its own file, `e2e/support/cleanup-mock.ts`, plugged into
  `core-mock.ts` through three names on `globalThis` (its channel, an answer to change, the cleanup's `invoke`): a plan
  per selection over the scan's findings (computed in Node with the unit tests' `planOf`), the echo compared field by
  field with the shown plan (`PlanGone`, `PlanMismatch`), a cleanup streamed item by item and held after `hold` items
  until `releaseCleanup` or Stop, `findings-removed` and the overview after it, a history with put back, and
  `requestQuit`. Scripts: `cleaning`, `cleaningHeld`, `cleaningLeftOut`, `cleanupsEmpty`. `core-mock.spec.ts` checks
  the plan of a selection, the left-out finding and the history.
- **The flows** (`flows.spec.ts`, WebKit and Chromium, each checking the CSP): select → review → confirm (Trash) →
  progress → result → "See this cleanup" → put back one item → All cleanups; the permanent path; Stop mid-cleanup;
  an expired plan and a refused echo with "Review it again"; the left-out items; the quit question with Keep
  cleaning. The flows of plan 003 that expected the disabled button now expect it enabled.
- **Screenshots** (each looked at): 13 screen states in both engines (`cleanup-confirm`, `-permanent`, `-left-out`,
  `cleanup-refused`, `cleanup-progress`, `cleanup-quit`, `cleanup-result`, `-stopped`, `cleanups`, `cleanups-empty`,
  `cleanup-detail`, `-interrupted`), 23 new stories in the gallery with their focus looks, the Cleanups screen and the
  left-out confirmation at 125 % and 150 %; every screen screenshot of plan 003 again, since the sidebar has
  **Cleanups** now; the two renamed `ResultSummary` stories' old images removed.
- **axe** (the confirmation, a cleanup under way with the quit question, the result, the history and one cleanup),
  **keyboard** (the confirmation opened by keyboard starts on Cancel and gives the focus back; the screen-change walk
  includes Cleanups) and **zoom** extended.
- Found and fixed by the flows: WebKit does not focus a button on a click, so the confirmation gave the focus back to
  nothing; the review screen now focuses **Move to Trash…** before it asks the core, and again after "Review it
  again".
- `npm run test:e2e:ui`: 213 passed in WebKit and Chromium (run on a local config on port 4421, not committed).
