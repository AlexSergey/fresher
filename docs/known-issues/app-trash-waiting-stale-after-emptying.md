# The Trash line's first number is stale after the user empties the Trash

**Status:** open · **Updated:** 2026-11-22

## Symptom

The user empties the Trash in Finder and comes back to Fresher. The line "Waiting in the Trash: 23 GB, 20 GB of it from
cleanups here" loses the share at once (the journal's items are no longer found in the Trash), but the first number
keeps the last scan's measure of the Trash until the next scan.

## Cause

The Trash view has no walk of the Trash of its own: its first number is the last scan's measure of the folder plus
what cleanups moved there since (plan 037, agent's call 4). A scan is the only thing that measures.

## What was tried

Nothing: the stale number was accepted in the design, for the cost of a walk of a folder that can hold millions of
entries on every look.

## Options left

- A shallow measure of the Trash's top-level entries on each look.
- "As of the last scan" in the line's sentence.
