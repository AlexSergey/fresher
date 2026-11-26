# The disk chart keeps its numbers after a cleanup

**Status:** open · **Updated:** 2026-11-26

## Symptom

Found in the sandbox run by hand before 0.1.0 (`npm run dev:sandbox -- trial --real-trash`): after a cleanup to the
Trash and then a permanent delete of a 100 MB folder, the disk chart on the scan screen still showed the numbers it
had before the cleanups.

## Cause

Not investigated yet. To check first:

- which numbers stay: what Fresher can free (the findings removed by the cleanup), or the free space of the volume;
- whether the chart is refreshed after `cleanup-finished` and `findings-removed`, or only after a scan;
- in sandbox mode the volume's space is the real Mac's (`docs/testing.md`, "Sandbox mode"), so 100 MB freed on a large
  disk may be lost in the chart's rounding; the sizes of the findings are the sandbox's own.

## What was tried

Nothing yet.

## Options left

- Refresh the chart from the core after every cleanup, as the findings list already is.
