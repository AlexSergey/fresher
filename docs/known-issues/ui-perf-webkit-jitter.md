# The WebKit perf specs fail at times by one to three milliseconds

**Status:** open · **Updated:** 2026-11-21

## Symptom

`npm run test:perf --workspace app/ui` fails at times in the reference engine (WebKit) on a p95 of 20 to 22 ms where
the idle calibration reads 19 ms, or on a single frame over 50 ms in "dialog opens (gallery)". Seen twice on the
reference Mac while plan 027 was checked: once on the branch (three soft failures in the 50,000-finding scan and the
transitions), once on `main` (the scan test). A rerun with the same code passes with every row at 19 ms or less.

## Cause

WebKit's frame clock counts whole milliseconds and jitters ([`perf.spec.ts`](../../app/ui/e2e/perf.spec.ts), the
note above `expectWithinBudget`); the budget is `max(17 ms, idle)` of the same run, and the idle row itself moves by
a millisecond between runs. A longer pause of the Mac in one of three runs moves the median of a short scenario.

## What was tried

The budget already follows the run's own idle row. The specs run one at a time (`--workers 1`).

## Options left

Compare against the idle row plus one millisecond, or take the median of five runs for the scenarios of about 50
frames. Not done: the budget is the plan 003 contract and the failures do not repeat.
