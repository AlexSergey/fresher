# 003/05/02 — Totals instead of findings

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `app/ui/src/state/reduce.ts` — `CoreState` loses `groups`; it keeps `overview: Overview | null` (totals per
  category, `found`, the last scan), the status, progress, module outcomes, the dictionary, the failure and `seen`.
  `findings-added` no longer copies findings: it marks the overview stale (`stale: true`); `overview-loaded` stores the
  overview and clears `stale`, under the existing rules that an older overview never undoes a newer event.
- `app/ui/src/state/refresh.ts` — `createOverviewRefresher(store, { interval: 250, clock })`: when the state turns
  stale it asks `getOverview` at most once per interval, never two requests at once, and always once more after
  `scan-finished`; the clock is injected (tests use a fake one).
- `app/ui/src/state/progress.ts` — `scan-progress` reaches the state at most 10 times a second (the last one always),
  so React renders no more than that during a scan.
- `app/ui/src/state/connect.ts` — starts the refresher with the store.
- `app/ui/src/app.tsx` — the bare screen of plan 002 lists the category totals (name, size, count, highest level)
  instead of findings, until 07/01 replaces it; its screenshots are updated and looked at.
- `docs/known-issues/ui-findings-kept-in-memory.md` — closed: the store keeps totals; the lists are paged (05/03).
  Its row in `docs/known-issues/README.md` follows the folder's convention for a closed issue.

## Produces

`useCore((s) => s.overview)` for the overview, the sidebar's module sizes and the hero; a store whose memory does not
grow with the scan.

## Checks

- Negative: `a findings batch does not grow the state` (the state after 1,000 batches holds no finding),
  `an overview of an older scan is ignored`, `no second overview request while one is pending`, `progress beyond 10 a
  second is dropped except the last`, `an event of another scan does not mark the overview stale`.
- Positive: `findings mark the overview stale and one refresh follows within 250 ms` (fake clock), `scan-finished always
  refreshes once more`, `the bare screen shows the totals of the demo scan`.
- `npx vitest run src/state` passes; `npm run test:e2e:ui` passes with the updated screenshots.

## Notes

- The stale mark is cleared when the refresher **asks** for the overview (a new action, `overview-requested`), not
  when the answer arrives: an event that comes while a request is pending marks the overview stale again, so one more
  request follows. Clearing it on `overview-loaded` would lose that event. `overview-loaded` keeps the existing rule
  that an older overview changes nothing: it is ignored, status and totals alike, when it predates the scan the
  events tell of.
- `module-finished` and `scan-finished` mark the overview stale as well as `findings-added` (the design's list);
  `scan-finished` is what makes the last request always follow.
- The injected clock is `state/clock.ts` (`Clock`, `systemClock`); the tests' fake one is `src/test/fake-clock.ts`.
- `e2e/core-mock.ts` gained `scanned`: the overview `get_overview` answers once `start_scan` was called, so the e2e
  tests see the totals of the streamed scan.
