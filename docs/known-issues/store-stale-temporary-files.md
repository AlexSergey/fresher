# Stale temporary files in the data folder

**Status:** open · **Updated:** 2026-11-16

## Symptom

If the app is killed (or the Mac loses power) between creating the temporary file of an atomic write and renaming it,
the temporary file stays in the data folder (`~/Library/Application Support/io.github.alexsergey.fresher/`). Nothing reads it;
it only takes a little space. Each further crash at the same moment can leave one more.

## Cause

An atomic write is a temporary file in the same folder, `fsync`, then a rename (`crates/store/src/atomic.rs`). A failed
write removes its temporary file, but a crash gives it no chance to.

## What was tried

- Since the code review of plan 002 the temporary files are named `.<target>.<random>.tmp` (`.settings.json.<random>.tmp`),
  so one left behind is recognizable as the store's own.
- Removing them when `DataFolder::open` runs was considered and not done: only `fresher-cleaner` may delete
  (Clippy `disallowed-methods`, [`../restrictions/deletion-safety.md`](../restrictions/deletion-safety.md)), there is no
  documented exception for the app's own data folder, and plan 002 decided that the store deletes nothing (agent's
  call 8, "clearing the app's own data is decided in plan 005").

- Plan 005 left clearing to later ([deferred](../deferred/own-data-clearing.md)): the store and the app still delete
  nothing of their own.

## Options left

- Plan 005 left clearing the app's own files to later ([deferred](../deferred/own-data-clearing.md)); when it is
  done, the stale temporary files can be removed there: regular files only, never a link, matching exactly
  `.<known target>.<random>.tmp`, inside the data folder.
- An exception to the deletion lint for the data folder, with its reason in the restriction, if the user agrees.
