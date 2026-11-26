# A setting changed during a scan does not ask for a rescan

**Status:** fixed (the engine carries a change made during a scan into that scan's `LastScan`) · **Updated:** 2026-11-26

## Symptom

A setting that applies at the next scan (the scan folders, a module's switch, an opt-in) changed while a scan runs
does not show "Rescan to apply" after that scan ends, although the scan ran with the settings it started with.

## Cause

The mark lives on the last scan (`LastScan.settingsChanged`). A change during a scan marks the scan before it; the
running scan then finishes with a new `LastScan`, unmarked.

## Fix

The results keep "a setting that applies at the next scan changed since the running scan began", cleared when a scan
begins. A change while a scan runs sets it; a change with no scan running still marks the last scan. The scan's
`LastScan` takes the flag when the scan ends, whatever its outcome, so the UI shows "Rescan to apply". Proved by
`crates/engine/src/scan_settings_tests.rs`.
