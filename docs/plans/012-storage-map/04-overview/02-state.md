# 012/04/02 — The state

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** app/ui

## Files

- `app/ui/src/state/store.ts`, `app/ui/src/state/reduce.ts` — the store keeps the map (`storageMap: null |
  StorageMap`) as it keeps the overview: totals, not rows. Actions `storage-map-loaded`; a scan's start keeps the last
  map until the first answer of the new scan replaces it.
- `app/ui/src/state/refresh.ts` — the overview refresher asks `get_overview` and `get_storage_map` together, on the same
  beat (at most 4 times a second while stale, once more at the end of a scan), still never two requests at once; a
  failure of either dispatches `command-failed` as today. No new event (design, section 6).
- `app/ui/src/state/storage.ts` (new) — `useStorageMeaning(volume)` and `useStoragePlace(node)`: the two views' data,
  asked when the view is shown and again when the map's phase or state id changes; a place view keeps at most 10 pages
  of 100 rows, least recently used dropped, as `state/findings.ts`; a `StaleNode` refusal reopens the volume's first
  root.

**Built:** `useStoragePlace(node, volume)` takes the volume, so a `StaleNode` refusal reopens that volume's first root.

## Produces

The data the bar (04) and the views (04, 05) render.

## Checks

- Negative first: `refresh.test.ts` — `a_failed_map_does_not_stop_the_overview`, `no_second_request_while_one_is_pending`;
  `storage.test.ts` — `a_stale_place_reopens_the_root`, `no_view_is_asked_before_it_is_shown`,
  `an_eleventh_page_drops_the_least_recently_used`.
- Positive: `refresh.test.ts` — `the_map_is_asked_with_the_overview`, `the_end_of_a_scan_asks_once_more`;
  `reduce.test.ts` — `a_loaded_map_is_kept`, `a_new_scan_keeps_the_last_map_until_its_first_answer`;
  `storage.test.ts` — `a_new_state_asks_the_view_again`.
- `npx vitest run src/state`.

## Conclusions

Built in `ca4d636`, fixed in `8174814`, reviewed approved.

- `storageMap` in the store, asked beside the overview; `useStoragePlace`, `useStorageMeaning` with a page cache of 10
  pages of 100 rows. Deviation: `useStoragePlace(node, volume)` takes the volume so a stale node reopens that volume's
  first root (the node-only signature was ambiguous).
- Final wave: the refetch key is the content of the volume's map plus the phase (`mapKeyOf`); the phase alone left
  the views stale after an ignore. Tests fail on the old code.
- Measured: 864 UI tests at the step, 926 at the end.
