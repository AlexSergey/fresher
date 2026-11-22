# The sandbox clock starts again at every launch

**Status:** open · **Updated:** 2026-11-22

Sandbox mode only (`npm run dev:sandbox`). The normal app runs on the system clock and is not affected.

## Symptom

After the app is closed and opened again on the same sandbox:

- the scan screen's Trash line can count the app's cleanups twice: "Waiting in the Trash" shows the scan's measure
  plus items the scan already measured;
- a scan cache of any age is taken as fresh: the rescan's age guard (a cached state older than 14 days is read again
  in full) never applies.

## Cause

Sandbox mode runs the core, the scanner and the journal on a `StartedClock` that starts at the moment written in the
sandbox's `world.toml` every time the app starts, so the case files' ages hold whenever the sandbox is opened. The
journal of cleanups and the scan cache in the sandbox's data folder outlive a launch, and their times come from earlier
launches of the same clock. After a relaunch the clock is back at the start, so:

- a cleanup made in an earlier launch can carry a time later than a new scan's: the Trash view takes it as moved after
  the scan (which could not see it) and adds it to the scan's measure, while the scan did measure it;
- the cache's age (now minus the time the cached state was taken) is zero or negative, which reads as fresh.

## What was tried

Nothing beyond one clock for the scanner and the journal (plan 037): before it, the scanner dated its states by the
system clock and the journal by the sandbox clock, which broke the comparison within a single launch. A clock that
keeps running across launches was considered and left: the case files' ages need the generation's moment at every
launch, and the dates a run by hand checks are those ages.

## Options left

- Generate the sandbox again before a run by hand (`npm run sandbox -- <feature>`): a fresh data folder has no journal
  and no cache from an earlier launch.
- Start the sandbox clock at the generation's moment plus the time since the data folder was first used, so it never
  goes back across launches.
