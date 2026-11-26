# A setting changed during a scan does not ask for a rescan

**Status:** fixed (the engine carries a change made during a scan into that scan's `LastScan`) · **Updated:** 2026-11-26

## Symptom

A setting that applies at the next scan (the scan folders, a module's switch, an opt-in) changed while a scan runs
does not show "Rescan to apply" after that scan ends, although the scan ran with the settings it started with.

## Cause

The mark lives on the last scan (`LastScan.settingsChanged`). A change during a scan marks the scan before it; the
running scan then finishes with a new `LastScan`, unmarked.

## Fix

The results count the changes of the settings that apply at the next scan (a revision). A scan's plan reads that
revision under the settings' lock, at the moment it reads the settings, in `start_scan`; the scan's `LastScan`, when
the scan ends, whatever its outcome, is marked when the revision moved since, so the UI shows "Rescan to apply". A
change made after `start_scan` returned but before the scan's thread began is caught too. A change with no scan
running still marks the last scan. Proved by `crates/engine/src/scan_settings_tests.rs`.
