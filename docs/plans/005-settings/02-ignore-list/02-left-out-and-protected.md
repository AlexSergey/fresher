# 005/02/02 — Left out and protected

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** engine, e2e

## Files

- `crates/engine/src/results.rs` — an `Ignored` filter applied to `page`, `finding` and `totals`: a finding of an
  ignored category, or at or below an ignored path, is left out.
- `crates/engine/src/cleanup/pending.rs` — a plan is made from the visible findings only.
- `crates/engine/src/cleanup/context.rs` — `Protection::default().with_ignored(paths)`; a list it refuses blocks the
  cleanup (`CleaningBlocked`, settings), failing closed.
- `crates/e2e/tests/ignore.rs` — on a fixture home: scan, ignore a category and an item, the findings and totals leave
  them out, restore brings them back without a rescan.

## Checks

- Negative: an ignored finding is not in a page, a total or `get_finding`; selecting it is `UnknownFinding`; a
  finding that holds an ignored path is left out of the plan with the reason "ignored"; a shared prefix is not
  "below" (`/a/cache` does not hide `/a/cache2`).
- Positive: restore brings the findings back; ignoring nothing changes no answer.
