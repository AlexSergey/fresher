# 006/02/04 — Coverage of a scan

**Status:** ✔️ done · **Version:** 0.4.0 · **Date:** 2026-11-16 · **Modules:** core, engine

Needs plan 004/03: the walk takes folders it must not enter, each marked as skipped by the user or for access (the
latter makes its parents' sizes `Partial`). If plan 004 names that input differently, this step uses its name; the
behaviour below is what matters.

## Files

- `crates/core/src/coverage.rs` (new) — `Coverage { not_entered: Vec<NotEntered>, unreadable: Vec<UnreadableCount> }`,
  `NotEntered { capability, access }`, `UnreadableCount { reason: Unreadable, folders: u64, unlocked_by:
  Option<Capability> }`, `Coverage::is_complete()`. No path anywhere in it.
- `crates/core/src/event.rs` — `Event::ScanCoverage { scan, coverage }`, sent right before `ScanFinished`.
- `crates/core/src/command.rs` — `LastScan.coverage: Coverage`.
- `crates/engine/src/coverage.rs` (new) — before the walk: `AccessProbe::guarded_places(roots)`; a place whose
  capability is `Granted` is walked; a place whose opt-in switch is on is tried with `try_place` (`Granted` → walked);
  every other place is skipped without being touched and becomes a `NotEntered`. After the walk: the summary's
  `Unreadable::Guarded` folders become an `UnreadableCount` unlocked by `AllFiles`; `NoPermission` and `Failed` are
  counted with no capability.
- `crates/engine/src/session.rs` — runs the two halves around the walk; an incomplete coverage ends the scan `Partial`.

## Produces

The honest coverage the UI shows (plan 04) and the "not seen" part of plan 003's result summary. The rule "a scan never
enters a guarded place that is not granted or opted into" holds here.

## Checks

- Negative: `a_guarded_place_that_is_not_opted_in_is_never_tried` (`FakeAccess` records no `try_place` call, the walk
  receives the place as skipped), `an_opted_in_place_that_is_denied_is_skipped_and_reported`,
  `guarded_folders_make_the_coverage_incomplete_and_the_scan_partial`, `coverage_holds_no_path` (serialized JSON has
  no `/`).
- Positive: `with_all_files_granted_no_place_is_tried_and_all_are_walked`,
  `an_opted_in_place_that_is_granted_is_walked`, `guarded_unreadable_folders_are_unlocked_by_all_files`,
  `coverage_is_sent_before_scan_finished_and_kept_in_the_last_scan`, `a_complete_scan_has_complete_coverage`.
- The core e2e (`crates/e2e`) gains a case on a fixture home with a guarded `Desktop`: the golden report shows it not
  entered.

## Conclusions

- `Coverage`, `NotEntered` and `UnreadableCount` are in `crates/core/src/coverage.rs` (camelCase fields, exported to
  TypeScript); `Event::ScanCoverage { scan, coverage }` is published right before `ScanFinished`, and
  `LastScan.coverage` keeps it. `crates/engine/src/coverage.rs` holds the two halves: `before_walk` decides every
  guarded place of the roots, `after_walk` builds the coverage. The session passes the skipped places as the walk
  request's `avoid` (plan 004's name for "folders not to enter for access"; skipped by the user does not exist yet),
  and an incomplete coverage ends the scan `Partial`.
- Not spelled out by the step, taken the simplest way:
  - **What is walked without a try**: every place when `AllFiles` is `Granted`, and a place whose own capability
    `allows()` (`Granted`, or `NotApplicable`: nothing guards it). **What is tried**: a place whose switch
    `core.access.<id>` is on, and a `Volume` place (a scan root on that volume is the user's own opt-in); a place
    guarded by `AllFiles` alone is never tried. A capability is tried at most once per scan, and its answer stands
    for its other places; it is named once in `not_entered`, with the access that kept it out (the try's answer, or
    the state as last probed).
  - The answer of a try is not written back into the access status (no `AccessChanged` from a scan); the re-check
    on focus (plan 03/04) learns the grant.
  - A `Guarded` count is always unlocked by `AllFiles`, as the step says; zero counts are left out; the walk's
    `not_scanned` (avoided places, other volumes) is not repeated in the coverage, which names the capabilities
    instead. A walk that gave no summary (cancelled, stopped) counts no unreadable folder.
  - The opt-ins are read when the scan is planned (`ScanPlan.opt_ins`), through a small `Settings::core_switch` in
    `crates/engine/src/settings.rs` (not in the step's file list).
  - The e2e case is `crates/e2e/tests/coverage.rs` on `fixtures/homes/guarded.toml` with the reference walk: the
    Desktop is never tried nor entered (the committed state lists it as avoided) and `fixtures/golden/coverage.json`
    shows it not entered; a second case opts in, is granted, and ends complete.
- The UI ignores `scan-coverage` until plan 04 (one more case in the reducer); the demo stream of the UI tests sends
  it. Regenerated: the bindings (`Coverage`, `NotEntered`, `UnreadableCount`, `LastScan.coverage`, the new event) and
  `fixtures/golden/demo.json` (the event and `lastScan.coverage`, both complete).
- Left for later plans: the app's `UnknownAccess` knows no guarded place, so until plan 03 the app's walk still avoids
  the stand-in list of plan 004 and the coverage names nothing.
