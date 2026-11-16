# 008/07/01 — The IPC and the state

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `app/ui/src/ipc/index.ts` — the nine commands of the design, section 10, typed with the generated types; the only
  place their Rust names are spelled.
- `app/ui/src/ipc/mock.ts` — `mockCore` answers them; `src/test/cleanup-plan.ts` (new) builds plan views, reports and
  histories as the core sends them, shared by the unit and e2e tests.
- `app/ui/src/state/cleanup.ts` (new) — the cleanup's state as a discriminated union:
  `idle`, `preparing`, `confirming { plan }`, `running { cleanup, method, total, done, current }`,
  `finished { report }`, `refused { refusal }`; actions for each command's answer and each event
  (`cleanup-started`, `cleanup-items`, `cleanup-finished`, `quit-requested`); `confirmationOf(plan, method)` builds the
  echo from exactly the plan view the dialog shows (count, allocated bytes, highest level, method) — no sum is made in
  the UI.
- `app/ui/src/state/reduce.ts`, `store.ts` — the slice wired; `findings-removed` drops those findings from the paged
  cache (`state/findings.ts`) and marks the overview stale; `cleanup-finished` empties the selection
  (`state/selection.ts`).

## Produces

The state the confirmation (02), the progress and result (03) and the quit dialog read; the mocked core 05 drives.

## Checks

Vitest, negative first:

- "a refused prepare shows its text and returns to idle", "an event of another cleanup is ignored",
  "items before started are ignored", "the echo never differs from the plan view" (a table of views: for every one, the
  echo carries its own figures).
- Positive: "prepare then confirm runs then finishes", "items advance the done count",
  "findings removed leave the pages", "a finished cleanup empties the selection", "quit requested opens the question".
- `npm run lint`, `npm run test:ui`, coverage ≥ 90 %.

## Conclusions

- Built as written: the nine commands in `ipc/index.ts` (`prepareCleanup`, `discardCleanup`, `confirmCleanup`,
  `cancelCleanup`, `getCleanups`, `getCleanup`, `putBack`, `revealInTrash`, `quitAfterCleanup`), the slice in
  `state/cleanup.ts` with `confirmationOf`, wired into `state/reduce.ts`; the fixtures in `src/test/cleanup-plan.ts`
  (plan views, reports, the events of a cleanup, a history, the refusals and the core's texts they need).
- Decided where the step was silent, each the simpler or the safer way:
  - **The commands are sent from `state/cleanup-actions.ts`** (`prepare`, `confirm`, `discard`, `stop`, `quitAfter`),
    as `state/connect.ts` sends the scan: a screen calls one function with the store.
  - **`running.cleanup` is `null` from the confirm until the core names the cleanup**: the core's answer and its
    `cleanup-started` may come in either order; the dialog closes and the progress shows at once.
  - **A cleanup this UI did not start is followed from its `cleanup-started`** (a reload during a cleanup), with no
    plan, so no titles; items and the end of a cleanup the UI does not follow are ignored.
  - **`refused` behaves as `idle`**: the screen shows the core's text and a new prepare may start; an unreachable core
    is the frame's error and the cleanup shows nothing.
  - **The skipped and failed items are collected from `cleanup-items`** with the titles of the confirmed plan: the
    report carries only counts, and the result lists each one with its reason (03).
  - **A refused discard is not shown**: the plan is gone either way (`PlanGone`); an unreachable core is.
  - **`findings-removed` drops every page and asks the rows on screen again at once** (`state/findings.ts`): every
    page after a removed finding moved; filtering the pages in place would show a row twice until the refresh.
  - **The selection the core takes** is built by `cleanupSelectionOf` in `state/selection.ts`: ids only.
- Checks: every check of the step, plus "an unreachable core on prepare", "confirm, stop and a second stop do
  nothing out of their state", "the core may name the cleanup before its answer", "the skipped items are kept with
  their titles"; the IPC test covers every new command's arguments.
