# 006/04/01 — Access in the store

**Status:** ✅ approved · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `app/ui/src/state/reduce.ts` — the store holds `access: AccessStatus` and `coverage: Coverage | null` of the last scan;
  `access-changed` replaces `access`; `scan-coverage` sets `coverage`; `scan-started` clears it. Types from
  `bindings.generated.ts` only.
- `app/ui/src/state/connect.ts` — `getAccessStatus()` on connect.
- `app/ui/src/state/selectors.ts` (new, or the place plan 003 keeps selectors) — `accessOf(status, capability)`,
  `notSeen(coverage)` (the `not_entered` and `unreadable` entries that have a capability, grouped by capability, for the
  notices), `hasStaleResult(access, coverage)` (a capability that limited the last scan is now granted → "Scan again").
  Pure; no texts and no decisions the core already made.

## Produces

What steps 02–04 render, without any of them calling the IPC module for the state.

## Checks

- Negative: `an_access_changed_event_does_not_touch_the_findings`, `scan_started_clears_the_coverage`,
  `a_complete_coverage_has_nothing_not_seen`, `a_grant_unrelated_to_the_last_scan_is_not_stale`.
- Positive: `access_changed_replaces_the_status`, `not_seen_groups_by_capability`,
  `a_grant_that_limited_the_last_scan_makes_it_stale`.
- `npx vitest run app/ui/src/state`; coverage stays at 100 % of lines.
