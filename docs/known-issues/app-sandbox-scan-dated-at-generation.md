# A sandbox scan says it was made days ago

**Status:** open · **Updated:** 2026-11-17

## Symptom

In sandbox mode (`npm run dev:sandbox`), a scan made a moment ago reads as made days ago ("scanned 3 days ago") when
the sandbox was generated days before.

## Cause

Sandbox mode runs the core on a `StartedClock` that starts at the moment the generator wrote into `world.toml`, so the
case files' ages hold whenever the sandbox is opened. The scan's time is that clock's, while the interface counts the
time since the scan with the computer's own clock.

## What was tried

Nothing: the ages of the case files need the generation's clock, and the dates a run by hand checks are the ages.

## Options left

- The interface takes "now" from the core (a command, or with the overview), so both sides use one clock.
- Generate the sandbox again before a run by hand (`npm run sandbox -- <feature>`), which resets its clock.
