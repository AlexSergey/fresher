# A FIFO is read as a regular file

**Status:** fixed ([plan 017/01/02](../../plans/017-temp-files/01-core-prerequisites/02-entry-kind.md)) · **Updated:** 2026-11-17

## Symptom

A FIFO (named pipe) in a scanned folder is listed as a regular file. If it carries a marker's name, for example
`CACHEDIR.TAG`, the matcher counts it as that marker; the reference walk (`NaiveWalk`) does not.

## Cause

The core's `RawEntry` and `EntryKind` (plan 004/02) know folders, files and links only, and the core is frozen until
plan 004 ends.

## What was tried

Nothing: noted by plan 004/04. For `CACHEDIR.TAG` no false marker results, because the walker reads the tag's content
before it trusts it, and a FIFO never yields the signature. Other markers are claimed by name alone.

## Options left

- A fourth kind, `Other`, in `EntryKind`, read from the bulk read's object type; the matcher ignores it as a marker.
  Taken up when the core is next opened (the user's call of 2026-11-16: live with it for now).

## Fix

Plan 017/01/02 added `EntryKind::Other` for a socket, a named pipe or a device: the bulk read, the portable reader and
the reference walk read it, the matcher never takes it as a marker, a rule never claims it, the cache keeps it (tree
format 3) and the cleaner's check 7 refuses an item that is one.
