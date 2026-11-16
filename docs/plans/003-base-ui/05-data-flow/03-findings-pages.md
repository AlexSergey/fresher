# 003/05/03 — Findings by page

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `app/ui/src/state/findings.ts` — `createFindingsSource(getFindings, { pageSize: 100, maxPages: 10, clock })`:
  - a cache per query key (`module`, `category`, `order`) of pages by number, least recently used dropped beyond
    `maxPages` in all;
  - `ensure(query, first, last)` loads the pages that range needs (one request per page, never twice at once);
  - `total(query)` from the last page answered; `rowAt(query, index)` → the finding or `undefined` while its page loads;
  - a new scan (`scan-started`) drops every page; during a scan, when the overview changes, the pages in view are asked
    again at most twice a second, so rows move to their new order; a refused page keeps the old rows and reports the
    failure once.
- `app/ui/src/state/use-findings.ts` — `useFindings(query)` → `{ total, rowAt, onRangeChange, failure }` on
  `useSyncExternalStore`, made for `VirtualList`'s props.
- `app/ui/src/test/large-scan.ts` — a generator of a scan of N findings in M categories with sizes spread over four
  orders of magnitude and every danger level; used by unit tests, the e2e mock and the performance spec (08/01).
- `app/ui/e2e/core-mock.ts` — `get_findings` answered from the script's findings by `module`, `category`, `order`,
  `offset` and `limit`, the same contract as the engine's, so the e2e tests page for real.

## Produces

The data of every finding list (06/03, the review screen), with at most 1,000 findings in memory.

## Checks

- Negative: `does not request a page twice while it loads`, `drops the least recently used page past ten`, `a new scan
  drops every page`, `a refused page keeps the rows it had and reports once`, `a range past the total requests
  nothing`.
- Positive: `loads the pages a range needs`, `rowAt gives undefined until its page arrives`, `the pages in view are
  refreshed at most twice a second during a scan`, `largest order pages are cached apart from found order pages`.
- `npx vitest run src/state` passes; a unit test with 100,000 generated findings scrolls the whole range and never
  holds more than 1,000.

## Notes

- The source's API: `ensure(view, first, last)`, `rowAt(view, index)`, `total(view)` (`undefined` before the first
  answer), `failure(view)`, `held()` (the findings in the cache, for the memory checks and 08/01), `subscribe` and
  `version` for `useSyncExternalStore`, and `follow(store)`: a change of the store's scan drops every page, a change
  of its overview asks the rows on screen again (at most twice a second). A view is `{ module, category, order }`.
- The rows on screen are also asked again for the overview that follows the end of a scan, not only during it, so a
  list shows the final order.
- `useFindings` reads the source from `FindingsContext`; the app frame (07/01) creates the source with
  `createFindingsSource(getFindings, { clock: systemClock, maxPages: 10, pageSize: 100 })`, calls `follow(store)` and
  provides it.
- `src/test/page-of.ts` is the engine's `get_findings` contract over a list of findings, shared by the unit tests and
  `e2e/core-mock.ts` (called in Node through Playwright's `exposeFunction`); `e2e/core-mock.spec.ts` checks that the
  mock pages as the engine does.
