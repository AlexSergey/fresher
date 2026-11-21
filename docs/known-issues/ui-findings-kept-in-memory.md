# The UI keeps every finding of a scan in memory

**Status:** fixed ([plan 003/05](../plans/003-base-ui/05-data-flow/readme.md)) · **Updated:** 2026-11-16

## Fix

Plan 003/05/02: the store keeps no finding. `findings-added` only marks the overview stale, and the refresher
(`app/ui/src/state/refresh.ts`) asks `get_overview` for the totals per category at most 4 times a second, and once
more when the scan finishes. The lists ask the core for pages of findings (`app/ui/src/state/findings.ts`, step
003/05/03), at most 10 pages of 100 cached. Measured in plan 003/08/01: over a streamed scan of 50,000 findings the
page's JS heap after a full collection grows from 2.7 MB to 4.9 MB, so the page holds no scan.

## Symptom

The UI store (`app/ui/src/state/reduce.ts`) keeps every finding the scan's `findings-added` events carry, grouped by
category. A batch copies each group it adds to once, so a scan of N findings in one category costs about N² / 512 copied
references over its batches of 256: nothing for thousands of findings, noticeable from a few hundred thousand. The
memory grows with the scan as well.

## Cause

Plan 002 renders the findings straight from the events, as the first screen that proves the pipeline. The core already
answers pages (`get_findings` with `offset` and `limit`), but nothing in the UI uses them yet.

## What was tried

The plan 002 review found a per-finding copy (O(N²) within a batch and across the scan); it was replaced by one pass
per batch and one copy per touched group per batch (the commit "fix(ui): a batch of findings is grouped in one pass
and each group is copied once").

## Options left

- Plan 003 (a category's details, `FindingList`): the store keeps per-category totals from the events and fetches the
  findings a screen shows with `get_findings`, page by page; the events no longer carry a copy of every finding into
  the store.
- If the events must keep feeding a list: keep each group's findings in chunks (one array per batch), so a batch adds
  a chunk instead of copying the group.
