# 002/02/03 — The scan session

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** engine

## Files

- `crates/engine/src/session.rs` — one scan: builds the `InterestSet` of the enabled modules, runs the `DiskWalk` with
  the roots from the settings, sends each `Match` to the module that owns the interest, then calls every module's
  `finish`; assigns `FindingId`s in order; publishes `ScanStarted`, `ScanProgress`, `FindingsAdded`, `ModuleFinished`,
  `ScanFinished`.
- `crates/engine/src/batch.rs` — the batcher: flush at 256 findings, at 50 ms on the `Clock` port, at a module's end.
- `crates/engine/src/results.rs` — the last scan's findings by id; `Engine::overview()` (totals per category: count,
  summed sizes with the weakest completeness, highest danger), `Engine::findings(query)`, `Engine::finding(id)`.
- A module that returns an error gets `ModuleFinished { outcome: Failed }` and receives nothing more; the others go on.
  A module that panics is caught (`catch_unwind`) and ends `Failed` with the core's `core.module.panicked`; the release
  profile unwinds (`panic = "unwind"`). Fixed after review, see the notes.

## Produces

The scan path end to end inside the engine; `overview`, `findings`, `finding` for the commands (05/02) and for plan 008.

## Checks

- Negative: `a_failing_module_does_not_fail_the_scan`,
  `a_disabled_module_gets_no_matches`, `refuses_an_unknown_finding_id`, `an_id_from_the_previous_scan_is_unknown`,
  `a_cancelled_scan_still_publishes_scan_finished_partial`.
- Positive: `sends_each_match_to_the_module_that_asked_for_it`, `assigns_finding_ids_in_order`,
  `batches_findings_instead_of_one_event_each` (1 000 findings, `FakeClock` still → 4 batches),
  `flushes_a_batch_when_the_clock_moves_50_ms`, `overview_sums_sizes_and_takes_the_highest_danger`,
  `a_partial_size_makes_its_category_total_partial`.

## Notes from the implementation

- **What a scan runs is fixed when it starts** (`ScanPlan`: the roots, the enabled modules with their settings); a setting changed during a scan counts from the next one.
- **Settings are loaded here already** (`settings.rs`: load through the port, resolve against the schema), because
  `a_disabled_module_gets_no_matches` needs `<module>.enabled`; 02/04 adds the rest. `Engine::new` now returns
  `Result<Self, SchemaError>`: it builds the schema from the registry, which fails only if the core's own settings
  repeat a key (no panicking path).
- **The roots are passed to the walk as written** (`~` stays `~`): the engine does no IO, so the app's walk adapter
  fills in the home folder (decision 22 of the design).
- **Finding ids keep increasing across scans** (from 1), so an id of an earlier scan is unknown by construction. Ids
  are taken when a module pushes a finding; if the `u32` ids ever run out, further findings are dropped and the scan
  ends `Partial`.
- **Results are live**: the current scan's findings are kept as they are sent, so `findings` and `overview` agree with
  the events already published. `overview.last_scan` is the last *finished* scan; `totals` lists only categories with
  findings, in registration order. Sizes are summed saturating at `u64::MAX`.
- **A scan ends `Partial`** when it was cancelled, the walk could not read everything (`WalkSummary` not complete, or
  `WalkError::RootUnreadable`, which is logged), a module failed or was cancelled, or ids ran out. A failing module
  does not fail the scan: its `ModuleFinished { Failed }` is published at once (after the open batch) and it gets no
  more matches or `finish`.
- **Batches** (`batch.rs`): the 50 ms are measured from the batch's first finding on the `Clock` port; a clock that
  went back sends the batch at once. Every `ModuleFinished` sends the open batch first.
- `FindingsQuery` names its module by string: an unknown module is `UnknownModule`; a category nobody declared matches
  nothing (the core has no `UnknownCategory` error).
- `ScanFinished` and the results' `LastScan` are written under the state's lock, just before the engine goes `Idle`.
- **Fixed after review: a panicking module.** A panic in a module's `on_match` or `finish` used to kill the scan
  thread: the engine stayed `Scanning` for good and a release build aborted. Each module call is now wrapped in
  `catch_unwind`; a panic becomes `ModuleFinished { Failed { core.module.panicked } }`, the other modules finish and
  the scan ends `Partial`. Any other panic on the scan thread (the walk) still ends the scan: the `Output` that
  ends it does so in its `Drop` (`Partial`), so the engine goes `Idle` and `ScanFinished` is published.
- **Fixed after review: walk progress is throttled.** Every `WalkProgress` used to become a `ScanProgress`.
  `throttle.rs` lets one through at most every 50 ms on the `Clock` port (the batches' interval; a clock that went back
  lets it through), and the newest one held back is sent when the walk ends, before the modules' `finish` and
  `ScanFinished`.
- **Fixed after review: a scan that cannot start keeps the previous results.** `plan()` used to empty the results
  before the thread was spawned, so a failed spawn wiped the last scan's findings. The scan's thread now empties them
  (`Results::begin`, which also gives the first finding id) as it starts.
