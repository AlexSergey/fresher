# Stale temporary files in the data folder

**Status:** fixed (`fix(cleaner)`: the data folder's stale temporary files are removed when it opens) · **Updated:** 2026-11-26

## Symptom

If the app is killed (or the Mac loses power) between creating the temporary file of an atomic write and renaming it,
the temporary file stays in the data folder (`~/Library/Application Support/io.github.alexsergey.fresher/`). Nothing reads it;
it only takes a little space. Each further crash at the same moment can leave one more.

## Cause

An atomic write is a temporary file in the same folder, `fsync`, then a rename (`crates/store/src/atomic.rs`). A failed
write removes its temporary file, but a crash gives it no chance to.

## Fix

`fresher-cleaner` has one narrow operation for the app's own data folder, `remove_stale_temporaries`
(`crates/cleaner/src/own_data.rs`), run by the shell when it opens the folder, before any write
(`app/tauri/src/own_data.rs`). It lists the folder held open through no link and removes only entries directly in it
(no recursion) that are regular files and are named exactly `.<target>.<random>.tmp`, with `<target>` one of the
store's atomically written files (`atomically_written_files` in `crates/store/src/own_files.rs`) and `<random>` one or
more ASCII letters or digits. It deletes for good at the cleaner's single deletion point (they are the app's scratch
files, not the user's). A failure is logged and never stops the start. The restriction is in
[`../../restrictions/deletion-safety.md`](../../restrictions/deletion-safety.md), "Scanning is read-only"; the rest of
[the deferred item](../../deferred/own-data-clearing.md) stays deferred.

## What was tried (before the fix)

- Since the code review of plan 002 the temporary files are named `.<target>.<random>.tmp`, so one left behind is
  recognizable as the store's own.
- Removing them in the store was refused: only `fresher-cleaner` may delete (Clippy `disallowed-methods`) and plan 002
  decided that the store deletes nothing.
