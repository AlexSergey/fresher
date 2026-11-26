# The rescan of the real home seems to become a full scan every time

**Status:** open · **Updated:** 2026-11-26

## Symptom

In the dev run records, every scan of the real home (`NO_COMMIT/dev-runs/*_mac.events.jsonl`) ends with
`kind: full`, and no rescan does. The scan that ran gives the right result, but the rescan runs first and its work is
lost: it loads the cache, walks again the changes and the places outside the roots, reads the check's sample, and then
a full walk reads everything anyway. Each scan the user starts takes longer than a full scan alone. The scan's progress
falling back to 0 (`ui-scan-progress-starts-over.md`) came from this fallback; the bar now holds instead.

## Cause

Not investigated. The records hold neither the fallback reason (`WalkSummary::fallback`) nor the progress, so they do
not show which step gives up. The progress the user saw rise and fall points to the sampled check
(`FullScanReason::CheckFailed`, `crates/scanner/src/rescan/check.rs`), the only fallback after the rescan has walked
anything: a change the journal did not report, or a difference the check sees that is not a change.

## What was tried

Nothing yet.

## Options left

- Log which check fails: the fallback reason in the run record, and for `CheckFailed` the folder and what differed.
- Find why: a journal that misses changes in the real home, or a check that compares something the walk does not
  keep the same (times, folded sizes, entries that come and go).
