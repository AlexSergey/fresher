# 002/02/02 — The state machine

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** engine

## Files

- `crates/engine/src/engine.rs` — `Engine::new(registry, Ports { walk, settings, clock, commands, access })` (each an
  `Arc<dyn …>`), `status()`, `start_scan() -> Result<ScanId, CommandError>`, `cancel_scan()`, `shutdown()` (cancel and
  join the running thread). The state lives in one `Mutex<State>`; a poisoned lock is `CommandError::Internal`, never a
  panic.
- `crates/engine/src/state.rs` — `State { Idle, Scanning { scan, cancel, thread } }` and its transitions as plain
  functions (testable without threads). `Cleaning` stays in `EngineStatus` for plan 008; no command reaches it yet.
- `crates/engine/src/testing.rs` (`#[cfg(test)]`) — `ScriptedWalk` (emits a fixed list of matches, can block until
  cancelled), `MemorySettings`.

## Produces

The lifecycle 02/03 runs a scan inside, and `shutdown` for the shell's quit (05/03).

## Checks

- Negative: `refuses_a_scan_while_one_is_running` (`Busy { running: Scan }`), `refuses_to_cancel_when_idle`
  (`NotRunning`), `a_cancelled_scan_ends_partial`, `a_poisoned_state_is_an_internal_error`.
- Positive: `a_scan_goes_idle_scanning_idle`, `shutdown_cancels_and_joins_the_running_scan`,
  `scan_ids_increase_per_scan`.
- No test sleeps on the wall clock: waits use the bus's receiver with a timeout only as a failure guard.

## Notes from the implementation

- `error.rs` arrives here: `EngineError { Poisoned, Spawn, ScanIdsExhausted }`, each `CommandError::Internal` for the
  UI (logged with `tracing`). `lock` maps a poisoned lock to `Poisoned`; the paths that must finish (the end of a scan,
  `shutdown`) take the data as it was left (`lock_anyway`).
- `state.rs` holds a `Machine` (the `State` plus the id of the last scan): `next_scan` (`Busy`, or the next id),
  `start`, `cancel` (`NotRunning`), `finish(scan)` and `stop` (cancel and hand over the thread). A running scan's
  `thread` is an `Option`, so the transitions are tested without threads. Scan ids start at 1.
- `Engine::new(registry, ports)`; `Engine` also has `subscribe()`, `modules()`, `dictionary()` and `access_status()`.
  Dropping the engine calls `shutdown`.
- The scan thread publishes `ScanFinished` and goes `Idle` under the state's lock, so a subscriber that sees
  `ScanFinished` always finds the engine idle, and the next scan's events never come before it.
- Until 02/03 the scan thread only runs the walk with no roots and no interests.
- `testing.rs` adds `ScriptedWalk` (`held()` waits on a `Gate` or the cancel token, with `yield_now`, no sleep),
  `MemorySettings`, `FixedAccess`, the `engine(walk)` fixture, `wait_for_finish` (a 10 s guard, never a timing
  assumption) and `poison`. The engine's tests use `FakeClock` and `FakeCommandRunner` of `fresher-testing` as a
  dev-dependency; `crates/testing/` is not changed.
- **Fixed after review: `shutdown` has a deadline.** It used to join the scan thread with no limit, so a port that
  ignored the cancel could hang the app's quit. The thread now holds the sender of a `done` channel, closed when it
  ends (a return or a panic); `shutdown` cancels, waits on that channel at most 5 s (`SHUTDOWN_DEADLINE`), then
  joins. Past the deadline it logs an error and drops the handle: the thread is detached and the app's exit ends it.
  Test: `shutdown_gives_up_on_a_scan_that_does_not_stop` (a walk that ignores the cancel, a 10 ms deadline).
