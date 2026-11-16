# 003/05 — Data flow

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** core, engine, app, app/ui

What the screens need from the core, and how the UI keeps it without holding every finding: two additive fields in the
core's contract, a store that keeps totals refreshed from the overview, findings paged on demand, the selection, and
the current screen. The design: [`../00-design.md`](../00-design.md), "Data flow". Closes
[`known-issues/ui-findings-kept-in-memory.md`](../../../known-issues/ui-findings-kept-in-memory.md).

Depends on plan 002 only; touches no design file, so it runs in wave 1 beside plan 01. Its steps run in order; 02
waits for 01/02 (both touch `app/ui/src/app.tsx` and the screenshots of plan 002).

## Steps

| #   | Step                                                             | Status   | Shared files                                                                                          |
| --- | ---------------------------------------------------------------- | -------- | ----------------------------------------------------------------------------------------------------- |
| 01  | [Two additions to the core](./01-core-additions.md)              | ✔️ done     | `crates/core/`, `crates/engine/`, `app/tauri/`, `app/ui/src/ipc/bindings.generated.ts`, `fixtures/golden/`, `app/ui/src/test/demo-scan.ts` |
| 02  | [Totals instead of findings](./02-store-totals.md)               | ✔️ done     | `app/ui/src/app.tsx` (the bare screen shows totals), the screenshots of plan 002, `docs/known-issues/` |
| 03  | [Findings by page](./03-findings-pages.md)                       | ✔️ done     | `app/ui/e2e/core-mock.ts`                                                                             |
| 04  | [Selection and navigation](./04-selection-and-navigation.md)     | ✔️ done     | —                                                                                                     |

## Conclusions

- **The core answers what the screens show**: `Overview.found` for the hero number and `FindingsQuery.order`
  (`found`, `largest`) for the lists. `largest` sorts per request; 100,000 findings in one category, one page of 100,
  take 2.9–5.4 ms on an Apple M3 Pro, so no sorted index is needed for plan 004.
- **The store keeps no finding**: it holds the overview (totals per category, `found`, the last scan), refreshed by
  `state/refresh.ts` at most 4 times a second while findings arrive and once more at the end of a scan; progress
  reaches React at most 10 times a second. A batch of findings changes nothing but the stale mark, so 1,000 batches
  leave the state as it was. The known issue
  [`ui-findings-kept-in-memory.md`](../../../known-issues/ui-findings-kept-in-memory.md) is fixed.
- **Findings are paged**: `state/findings.ts` keeps at most 10 pages of 100 per app, least recently used dropped; a
  scroll through 100,000 generated findings never holds more than 1,000. `useFindings(view)` gives a list its total,
  rows, range callback and failure. The e2e mock pages the same way, from one shared contract (`src/test/page-of.ts`).
- **Selection and navigation**: the selection (findings, and categories minus exclusions) lives in the core store
  and is emptied when a scan starts; its summary is a preview summed with the core's rule; nothing is pre-selected
  and selecting by level stops at Safe. The screen is a discriminated union in a small navigation store.
- **Left for the app frame (07/01)**: create the findings source and the navigation, call `follow(store)`, and
  provide `FindingsContext` and `NavigationContext`; the module list for `useScreen` comes from `get_modules`.
- Deviations, each in its step's notes: the stale mark is cleared when the overview is asked for (not when it
  arrives), so no event between the two is lost; the injected clock is `state/clock.ts`; the e2e mock gained the
  overview it answers after a scan (`scanned`) and a spec of its own paging (`e2e/core-mock.spec.ts`).
