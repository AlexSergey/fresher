# The scan's progress goes to about 16 %, then starts over

**Status:** fixed (the scanner passes on only the progress that rises) · **Updated:** 2026-11-26

## Symptom

Seen by the user on the 0.1.0 release build (the dry-run image, installed, a first scan of the real home folder): the
scan's progress bar rises to about 16 %, falls back to 0 and rises again to the end. The scan completes and its result
is right; the progress looks as if the scan ran twice. Seen again by the user on the release draft of 0.1.0 (the
image downloaded by the browser, installed in Applications): it happens every time, not once.

## Cause

The scanner, not the UI. A scan the user starts is a rescan (`WalkMode::Auto`, `crates/scanner/src/scanner.rs`): it
walks again what changed and the places outside the roots (always in full), reporting their progress, then checks a
sample of the copied folders. When that check finds a change the journal missed (`FullScanReason::CheckFailed`), the
scan becomes a full walk with the same `on_progress`, and the full walk's counters start again from zero. The engine
publishes both runs under one `expected`, so the bar rose by what the rescan walked and fell back to 0. The dev run
records of scans of the real home all finish as `full`; they hold no `scan-progress` event and no fallback reason, so
they do not show which fallback it was. The other suspects were ruled out: the engine publishes `ScanProgress` from the
one walk only and takes `expected` once at the start (`crates/engine/src/session.rs`, `estimate.rs`); the UI only
throttles the progress and turns `bytes` against `expected` into the percent. The other fallbacks (no cache, a journal
gap, too many changes) happen before the rescan reports anything.

A test reproduced it: `a_failed_check_never_lowers_the_progress_the_rescan_reported`
(`crates/scanner/src/rescan/check_tests.rs`) saw the bytes go from 20 to 0.

## Fix

One counter, the bar only rises: the scanner wraps `on_progress` so each count is the highest seen and nothing is
passed on until one of them rises (`rising` in `crates/scanner/src/scanner.rs`). After a fallback the bar holds at
what the rescan reached until the full walk passes it, then rises to the end. The rescan's counts are not added to the
full walk's, which would count the same folders twice.

## What was tried

Nothing else.
