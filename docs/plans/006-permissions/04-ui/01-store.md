# 006/04/01 — Access in the store

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** app/ui

## What exists already

- `app/ui/src/state/reduce.ts` — `CoreState.access: AccessStatus | null`; `access-changed` replaces it;
  `access-loaded` (the answer of `getAccessStatus`) fills it only while it is `null`; `scan-coverage` is ignored
  ("has its notices in plan 006").
- `app/ui/src/state/connect.ts` — `requestAccess(store)`: asks `getAccessStatus` once, on demand, when the store has
  no access (not on connect); the hooks call it.
- `app/ui/src/screens/permission/use-access.ts` — `useAccess()`: requests the access and returns only the `all-files`
  state; used by the overview to show `PermissionCard` when it is `denied`.
- The overview's `lastScan` (`Overview.lastScan.coverage: Coverage`) carries the coverage of the last finished scan,
  including a restored one; `scan-started` keeps the old `lastScan` in the store until the next overview.

## Files

- `app/ui/src/state/reduce.ts` — `CoreState.coverage: Coverage | null`, the coverage of the result on screen:
  `scan-coverage` of the followed scan sets it; `scan-started` clears it; `overview-loaded` with a finished (not
  scanning) overview sets it from `overview.lastScan?.coverage` when the store has none for that scan (so a restored
  result has its notices). Types from `bindings.generated.ts` only.
- `app/ui/src/state/access.ts` (new; pure functions, like `state/selection-summary.ts`) —
  `accessOf(status: AccessStatus | null, capability: Capability): Access | null`;
  `notSeen(coverage: Coverage | null): readonly NotSeen[]` — the `notEntered` entries and the `unreadable` entries
  that name a capability (`unlockedBy`), grouped by capability, with the folder count of the unreadable ones, in the
  core's order; `hasStaleResult(status, coverage): boolean` — a capability that limited the result is now `granted`
  (→ "Scan again"). No texts and no decisions the core already made.
- `app/ui/src/screens/permission/use-access.ts` — `useAccessStatus(): AccessStatus | null` (the whole status, with
  `requestAccess`) beside `useAccess()`, which stays for the overview until 04/02 replaces its card; and
  `useCoverage(): Coverage | null`.
- `app/ui/src/state/reduce.test.ts`, `app/ui/src/state/access.test.ts` (new), `use-access.test.tsx` — the tests below.

## Produces

What steps 02–04 render, without any of them calling the IPC module for the state: `CoreState.coverage`, `accessOf`,
`notSeen`, `hasStaleResult`, `useAccessStatus`, `useCoverage`.

## Checks

- Negative (first): `an_access_changed_event_does_not_touch_the_findings`, `scan_started_clears_the_coverage`,
  `a_coverage_of_another_scan_is_ignored`, `a_complete_coverage_has_nothing_not_seen`,
  `an_unreadable_count_without_a_capability_is_not_listed`, `a_grant_unrelated_to_the_last_scan_is_not_stale`.
- Positive: `access_changed_replaces_the_status`, `scan_coverage_sets_the_coverage`,
  `a_restored_overview_brings_its_coverage`, `not_seen_groups_by_capability`,
  `a_grant_that_limited_the_last_scan_makes_it_stale`.
- `npx vitest run app/ui/src/state app/ui/src/screens/permission`; coverage of `src/state` stays at 100 % of lines;
  ESLint and `tsc` clean.

## Conclusions

- Built as specified: `CoreState.coverage` (set by the followed scan's `scan-coverage`, cleared by `scan-started`, taken
  from `overview.lastScan.coverage` by an accepted finished overview when the store has none), `state/access.ts`
  (`accessOf`, `notSeen`, `hasStaleResult`), `useAccessStatus` and `useCoverage`.
- Decision: `NotSeen` is `{ capability, access, folders }`; `access` is the state the scan recorded for a place it did
  not enter, `null` for a capability known only from unreadable folders. A capability named by both is one entry with
  the folder count of its unreadable ones. Order: the places not entered first, then the others, each in the core's
  order.
- Decision: `hasStaleResult` is true when any capability in `notSeen` is `granted` in the current status.
- Decision: the `scan-started` case moved into a `startScan` function in `reduce.ts` to keep the reducer under the
  complexity limit; behaviour unchanged.
- The store holds no state waiting on `relaunch()`; nothing here calls it.
- Tests: 6 negative and 5 positive cases from the checks, in `reduce.test.ts`, `access.test.ts` (new) and
  `use-access.test.tsx`; `src/state` and `src/screens/permission` run green.
- Final review fix: `hasStaleResult` (no caller: the notices read the current access per capability) and `useAccess`
  (no caller once every screen reads `useAccessStatus`) are deleted with their tests; `use-access.test.tsx` keeps its
  cases on `useAccessStatus`. The app providers' test of a failed modules refresh now waits until the failure is
  handled (a third request starts only after it settles) before it checks the list.
