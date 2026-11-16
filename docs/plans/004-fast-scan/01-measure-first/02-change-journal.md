# 004/01/02 — The change journal

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-15 · **Modules:** macos (probe)

## Files

- `NO_COMMIT/probes/004/journal/` — a Cargo project outside the workspace with hand-written declarations of the
  FSEvents and libdispatch functions it calls:
  - `mark` — prints the current event id (`FSEventsGetCurrentEventId`) and the journal's id for the device of a folder
    (`FSEventsCopyUUIDForDevice`);
  - `since` — creates a stream for a folder with `sinceWhen = <id>`, on a dispatch queue, starts it, calls
    `FSEventStreamFlushSync`, collects the events until `kFSEventStreamEventFlagHistoryDone`, prints each path's
    folder relative to the probe folder and its flags, then stops, invalidates and releases the stream;
  - `script` — makes scripted changes in a temporary folder between `mark` and `since`.

## Questions to answer

1. **A file growing in place** (append to an existing file, no rename): is an event recorded for its folder? This is
   the case the product exists to find; if the journal misses it, the design falls back to a full scan and the plan
   says so before going on.
2. Create, remove, rename within a folder, rename across folders, a new subtree, a removed subtree, a change of
   permissions, a hard link added: which folders are reported, with which flags.
3. `FlushSync` and timing: are changes made one second before `since` always reported? After a sleep of the Mac?
4. Retention: the oldest id still answered on the developer's Mac (`FSEventsGetLastEventIdForDeviceBeforeTime` at
   1, 7, 30 days ago); what a `since` older than the journal returns — a flag, nothing, or a silent gap.
5. Gaps: how `MustScanSubDirs`, `UserDropped`, `KernelDropped`, `EventIdsWrapped`, `RootChanged`, `Mount` and
   `Unmount` show up, and which can be provoked in a probe (thousands of changes in a burst).
6. A journal reset: what changes after `.fseventsd` is reset (on a disk image only — never on the system volume): the
   journal's id, the event ids.
7. Without Full Disk Access (the probe binary run from a terminal that has it and one that has not): are events under
   protected folders reported, hidden, or reported without detail?
8. Volumes: a disk image formatted APFS, HFS+ and FAT (`hdiutil`): which have a journal at all, which ids are per
   device.
9. The cost: the time of `since` for 1 day, 1 week of history on the developer's home; the number of folders
   reported for an ordinary day.

## Produces

Answers in the probe's `RESULTS.md`: the list of flags that mean a full scan; whether `FlushSync` makes the history
current; the retention; how to detect a journal older than the position (or that it cannot be detected, which makes
the sampled check of the design load-bearing); the size of an ordinary day's change set. Plan 03/04 implements the
adapter from these answers; 07 sets its full-scan threshold from the day's size.

## Checks

- Question 1 has a clear yes or no, with the probe's output as evidence in `RESULTS.md`.
- Every flag the design names (section 8) is listed with how the probe saw it, or "not provoked".
- The probe leaves no stream running (it exits cleanly) and changes nothing outside its temporary folder and the disk
  images it created.
