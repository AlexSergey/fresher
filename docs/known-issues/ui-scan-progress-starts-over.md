# The scan's progress goes to about 16 %, then starts over

**Status:** open · **Updated:** 2026-11-26

## Symptom

Seen by the user on the 0.1.0 release build (the dry-run image, installed, a first scan of the real home folder): the
scan's progress bar rises to about 16 %, falls back to 0 and rises again to the end. The scan completes and its result
is right; the progress looks as if the scan ran twice. Seen again by the user on the release draft of 0.1.0 (the
image downloaded by the browser, installed in Applications): it happens every time, not once.

## Cause

Not investigated. To check first:

- whether one scan publishes two runs of `scan-progress` whose `bytes` start again from zero: the walk of the scan roots
  and a second walk (the places walked outside the roots, the measure of the apps) each with its own counter, under
  one `expected` (`crates/engine/src/session.rs`);
- whether `expected` changes during the scan (a first scan has no previous one to estimate from);
- how the UI turns `bytes` and `expected` into the percentage (`app/ui/src/state/progress.ts` and the scan screen).

The development run record of a scan (`npm run dev`, `NO_COMMIT/dev-runs/*.events.jsonl`) holds every
`scan-progress` event and shows which of these it is.

## What was tried

Nothing yet.

## Options left

- One counter for the whole scan, so the bar only rises.
- If the scan has two phases on purpose, show them as two named steps instead of one bar that starts over.
