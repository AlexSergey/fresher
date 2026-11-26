# The demo recording's scan flashes by in a second

**Status:** fixed (a demo's scan is paced: at least 3 s, its progress rising evenly toward the demo volume's used
space) · **Updated:** 2026-11-26

## Symptom

Seen by the user in a demo recording (`npm run demo:record`, `assets/demo/*.gif`): the scan's progress card shows for
about a second and is gone. At 12 frames a second the recording catches only a few frames of it, so it looks like a
frame jerks instead of a scan that runs.

## Cause

A demo world (`fixtures/demos/`) is small: its scan ends in about a second, as it should. The recording shows exactly
what the app does; nothing paced the scan for a viewer.

## Fix

In demo mode only (a debug build started with `FRESHER_SANDBOX` and `FRESHER_DEMO`), the shell wraps the disk walk in
`PacedWalk` (`crates/sandbox/src/paced_walk.rs`). The walk runs as it is: its matches go on at once and its summary is
its own, so the findings are real. Only a demo's progress numbers are made up for the recording, as a function of the
time since the scan began: every 50 ms they rise evenly, the bytes toward the demo volume's used space (`[demo.volume]
used`, the size the bar measures against, so the bar fills; the walk's own bytes when the world states no volume) and
the entries toward the walk's, and they never go down. The scan ends once the walk is done and `DEMO_SCAN_TIME` (3 s)
has passed; a cancel stops it at the next tick, a failed walk ends at once. The wrapper is chosen in the sandbox's
composition (`SandboxMode::paced_scan`, `ScanPace` in `app/tauri/src/sandbox.rs`), compiled into debug builds only; the
app, the demo worlds' scan test and every autotest walk unpaced. Tests on a fake clock:
`crates/sandbox/src/paced_walk_tests.rs`. The 3 s keep the `cleanup` scenario well inside its 20 s budget.

## What was tried

- Making the demo world larger was not taken: more files to write for every recording, and the time still depends on
  the machine.
