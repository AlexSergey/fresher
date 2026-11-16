# A setting changed during a scan does not ask for a rescan

**Status:** open · **Updated:** 2026-11-16

## Symptom

A setting that applies at the next scan (the scan folders, a module's switch, an opt-in) changed while a scan runs
does not show "Rescan to apply" after that scan ends, although the scan ran with the settings it started with.

## Cause

The mark lives on the last scan (`LastScan.settingsChanged`). A change during a scan marks the scan before it; the
running scan then finishes with a new `LastScan`, unmarked.

## What was tried

Nothing yet: plan 005 left it, since a scan takes seconds and the screens mostly show the change once the user scans
again.

## Options left

- The engine remembers "changed since the running scan started" in the session and carries it into the scan's
  `LastScan` when it ends.
