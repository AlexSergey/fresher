# 003/05/01 — Two additions to the core

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** core, engine, app, app/ui

## Files

- `crates/core/src/` (the overview and the findings query) —
  - `Overview` gains `found: Sizes`: every finding of the current scan summed, the weakest completeness winning (the
    same rule as the per-category totals);
  - `FindingsQuery` gains `order: FindingOrder`, a new enum `found | largest` (kebab-case on the wire, exported with
    `ts-rs`); `found` is today's order.
- `crates/engine/src/results.rs`, `crates/engine/src/engine.rs` — `totals` also returns `found` (one pass, no second
  walk over the findings); `page` sorts the matching findings by allocated size, largest first, ties by id, when the
  order is `largest`.
- `app/tauri/tests/bindings.rs` run with `UPDATE_BINDINGS=1` → `app/ui/src/ipc/bindings.generated.ts` (reviewed diff).
- `fixtures/golden/demo.json` — the overview's `found` (`UPDATE_GOLDEN=1`, reviewed diff).
- `app/ui/src/test/demo-scan.ts`, `app/ui/src/state/`, `app/ui/src/ipc/` — the overview fixtures carry `found`; every
  `getFindings` call passes an `order`.

## Produces

`Overview.found` for the hero number (06/01) and `order: 'largest'` for the module page's lists (06/03).

## Checks

- Negative (Rust): `found_is_zero_and_complete_without_findings`, `found_takes_the_weakest_completeness`,
  `found_saturates_at_the_largest_size`, `largest_order_breaks_ties_by_id`,
  `an_offset_past_the_end_gives_an_empty_page_in_both_orders`.
- Positive (Rust): `found_sums_every_category`, `largest_order_pages_through_every_finding_once`,
  `found_order_is_the_order_of_the_scan`.
- `cargo nextest run -p fresher-core -p fresher-engine -p fresher-app` and `npm run test:e2e:core` pass; the bindings
  test passes; `npx vitest run src/state src/ipc` passes.
- Measured: `largest` on 100,000 findings in one category, one page of 100 — written below (if above 20 ms on the
  reference Mac, a sorted index is noted for plan 004).

## Measured

`largest` on 100,000 findings in one category, one page of 100 at offset 500, release build on an Apple M3 Pro:
2.9 ms at best, 5.4 ms at worst over 20 runs, under the 20 ms bar, so no sorted index is noted for plan 004. The
engine sorts references to the matching findings on each request and clones only the page.
