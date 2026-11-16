# 008/06/02 — The clean operation

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** engine

## Files

- `crates/engine/src/state.rs` — `State::Cleaning(Running)` beside `Scanning`, with `Operation::Clean` in `Busy`;
  `EngineStatus::Cleaning { cleanup }`; `cancel` and `shutdown` work the same for both.
- `crates/engine/src/engine.rs` — `Engine::confirm_cleanup(&Confirmation) -> Result<CleanupId, CommandError>` and
  `Engine::cancel_cleanup()`; `Engine::request_quit() -> bool` (publishes `QuitRequested` and answers `true` while
  cleaning) and `Engine::quit_after_cleanup()` (cancels and waits for the thread, like `shutdown`).
- `crates/engine/src/cleanup/operation.rs` (new) — confirm: take the pending plan (`PlanGone` if none), check the
  fail-closed preconditions again, `plan.confirm(echo, now)` (`ConfirmError` → `PlanMismatch` or `PlanGone`), read the
  available space of each affected volume, `journal.begin` (an error refuses, nothing acted on), switch to `Cleaning`,
  publish `CleanupStarted`, and run on its own thread:
  - `cleaner.execute(confirmed, cx, cancel, out)`, where `out` appends each `ItemDone` to the journal, syncs every 50
    items and every second on the `Clock`, and batches the reports into `CleanupItems` (256 or 50 ms, like findings);
  - a failed journal append cancels the operation: the cleaner finishes the item in flight and reaches no other;
  - at the end: the available space again, `journal.end`, the `CleanupReport` (`measured` the sum over the volumes of
    the space gained, at least 0; `None` when a volume could not be read), the removed findings out of the results
    (`FindingsRemoved`), `LastScan.changedSince`, `CleanupFinished`, then `Idle`.
- `crates/engine/src/results.rs` — `remove(findings)`: the totals and the overview recomputed from what is left (the
  claims of the removed findings dropped).
- `crates/engine/src/session.rs` — a scan start kills the pending plan (the hook of 01).

## Produces

`confirm_cleanup`, `cancel_cleanup` and the quit handshake for the commands (04); the events plan 07 renders.

## Checks

With `FakeCleaner`, `MemoryCleanupJournal`, `FakeVolumes`, `FakeClock` (negative first):

- `confirm_without_a_pending_plan_is_gone`, `a_mismatched_echo_is_refused_and_the_plan_is_gone` (a mismatch means
  the UI showed something else: the user prepares again), `a_confirmed_plan_cannot_be_confirmed_again`,
  `a_journal_that_cannot_begin_refuses_and_acts_on_nothing`, `a_failed_append_stops_after_the_current_item`,
  `scanning_while_cleaning_is_busy`, `a_cancel_ends_cancelled_with_the_rest_not_reached`,
  `a_volume_that_cannot_be_read_gives_no_measured_space`.
- Positive: `the_begin_record_is_written_before_the_first_item`, `events_come_started_items_finished_in_order`,
  `outcomes_are_batched`, `the_report_counts_and_measures`, `removed_findings_leave_the_overview`,
  `the_last_scan_is_marked_changed`, `quit_while_cleaning_asks_and_quit_after_cleanup_waits_for_the_item_in_flight`.

## Conclusions

- Built as written: `crates/engine/src/cleanup/operation.rs` (`confirm` and the cleanup's thread), the state machine
  `Idle → Scanning → Idle` and `Idle → Cleaning → Idle` (`State::Scanning(ScanId, Running)`,
  `State::Cleaning(CleanupId, Running)`, the thread renamed `OperationThread`), `Results::remove` and
  `mark_changed`, and the batcher made generic (findings and outcomes, 256 or 50 ms on the `Clock`). Every check of
  the step passes, with a cleaner that panics (the cleanup still ends `Stopped`, its journal closed, the engine
  `Idle`) and a permanent cleanup.
- Changed from the step, each the simpler or the safer way:
  - **The order inside `confirm`**: `Busy`, the pending plan taken (`PlanGone` if none; it is pending no more,
    whatever follows), the fail-closed preconditions, `plan.confirm` (`Mismatch` → `PlanMismatch`; `WrongPlan` and
    `Expired` → `PlanGone`), the context, the space, then `journal.begin` (a full disk → `CleaningBlocked { reason:
    disk-full }`, any other error → `journal`), all under the state's lock, so no scan can start in between.
  - **`CleanupStarted` is published by the cleanup's thread** as its first act: it always comes before the items.
    A thread that cannot be started ends the journalled cleanup `Stopped` and answers `Internal`.
  - **The space** is read once per volume of the plan's roots (`Volumes::identify` tells the volumes apart); the
    measured gain sums each volume's gain, each at least 0, and is `None` when any read failed.
  - **The removed findings** are the items that ended moved, emptied, deleted or done by a tool; `FindingsRemoved` is
    sent only when there are some. The last scan is marked changed after every cleanup. A finding counted less for one
    nested in it keeps its counted size (what was inside is gone too), so the totals stay a lower bound. **Changed by
    06's fixes**: the findings at or below an item that went whole leave too, and a plan of no item is refused.
  - **`cancel_scan` and `cancel_cleanup`** each cancel only their own operation (`NotRunning` otherwise); `shutdown`
    stops either. `quit_after_cleanup` is `shutdown`: it waits for the item in flight at most 5 s, like a scan; a
    longer item is left to the app's exit and the journal shows it as unknown.
  - **The blocked reason `open-files` is not raised by the engine**: it has no `OpenFiles` port (the cleaner does),
    and the `Cleaner` port cannot refuse a whole plan; a snapshot that cannot be taken is the cleaner's to answer by
    skipping the items (`CheckFailed`), never by removing them. The engine checks the settings file and the journal.
