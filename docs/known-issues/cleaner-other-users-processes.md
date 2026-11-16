# The in-use check sees only the user's own processes

**Status:** open (narrowed by plan 009) · **Updated:** 2026-11-16

## Symptom

An item held open only by a process of another user (a system service, a daemon running as `root` or as its own
user) is not skipped as in use: the cleaner moves it to the Trash, or deletes it on a permanent cleanup. The same
holds for a file the user's own process opens after the last snapshot (taken at the start and again every two seconds;
mapped files every 30 seconds), and for the few descriptors of the user's own processes whose path macOS withholds.

## Cause

Check 10 of the cleaner's guard reads the `OpenFiles` snapshot (`fresher-macos`'s `MacOpenFiles`, `libproc`). Without
administrator rights the process table answers `EPERM` for another user's process, so it is counted and left out, not
failed (`crates/macos/src/open_files.rs`); a snapshot is a moment, not a lock.

## What was tried

Plan 008/01 measured it: other users' processes answer `EPERM` and nothing else; a snapshot takes 7–19 ms and raises no
prompt, so the two-second refresh holds. The cleanup is not refused for it, because nearly every Mac runs services of
other users and the cleaner would then never act.

## Options left

- **Done by plan 009** (2026-11-16): check 10 also skips an item a running app or service owns by bundle id (a name of
  its path is the id, or starts with it and a dot), whether or not the cleaner can see its open files; the running
  apps include other users' processes whose executable the system names. Left: an item of another user's process
  that is not named after its bundle id.
- A privileged helper could read every process; it would need administrator rights the app does not ask for.
- What limits the harm now: the Trash is the default, so a wrong move can be put back; a rule never claims a running
  service's data folder.
