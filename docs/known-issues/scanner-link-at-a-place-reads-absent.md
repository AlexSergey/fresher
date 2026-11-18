# A link at a module's place reads as no folder

**Status:** open · **Updated:** 2026-11-18

## Symptom

`~/Library/Caches` (or another place a module asks about with `cx.place_seen`) that is a link to a folder on another
disk answers `PlaceSeen::Absent`, which counts as complete, as if the folder did not exist. `app-leftovers` reads its
places this way, so the data behind such a link is neither offered nor seen.

## Cause

The scanner never follows a link out of a scanned root and folds a link out of the tree, so a link at a place cannot
be told from no entry at all without a change to the walk's hot path.

## What was tried

- Answering `NotScanned` for a link (plan 019/01/02, first ruling): not possible, the tree keeps no trace of the link.
- Keeping `Absent` (the ruling that stands): safe, because the proof of an app's removal is a present saved state or
  container, never an absent place, so the link can only cause missed offers.

## Options left

- Record the link in the tree as a flag and answer `NotScanned` for it; costs a byte per link on the hot path and a
  bench.
- Accept it: the one effect beyond a missed offer is that the age rule of `app-leftovers` (a leftover changed within
  the days setting keeps its app) does not see recent data behind such a link. The app is already proven not installed
  by every other check.
