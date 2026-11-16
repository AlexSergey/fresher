# 008/03/02 — Open files and the user

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** macos

## Files

- `crates/macos/src/ffi/libproc.rs` (new) — safe functions over `proc_listallpids`, `proc_pidinfo`
  (`PROC_PIDTBSDINFO`, `PROC_PIDLISTFDS`, `PROC_PIDVNODEPATHINFO`, `PROC_PIDREGIONPATHINFO`), `proc_pidfdinfo`
  (`PROC_PIDFDVNODEPATHINFO`), `proc_pidpath` and `proc_name`; the constants and structures `libc` 0.2.191 lacks
  (`PROC_PIDFDVNODEPATHINFO` = 2, `PROC_PIDREGIONPATHINFO` = 8, `vnode_fdinfowithpath`, `proc_regionwithpathinfo`)
  defined there from the SDK's `sys/proc_info.h`; buffers grown and retried when a list grows between two calls;
  `EPERM` and a vanished process are answers, not errors.
- `crates/macos/src/open_files.rs` (new) — `MacOpenFiles`, which implements the core's `OpenFiles`: one pass over the
  processes of the current user (another user's answer `EPERM` already to `PROC_PIDTBSDINFO`, as 01/02 measured, and
  are counted, not failed), every vnode descriptor and the working folder, and every mapped file (the region walk,
  0.4–0.9 s, reused for 30 s: design, call 38), kept when it lies under one of `within`, with the process's name —
  the outermost app bundle's name in its executable path, else its short name (a helper's short name is not the
  app's: call 35). A descriptor whose path answers `EPERM` is counted and skipped. A failure to list the
  processes at all is `OpenFilesError` (the guard then fails closed); one process that vanishes is skipped.
- `crates/macos/src/process.rs` — `current_user() -> u32` (`getuid`, through `ffi`).
- `crates/macos/src/lib.rs` — the exports.

## Produces

Check 10 of the guard for the app (06/04); the current user for the guard's removability facts (plan 04).

## Checks

- Negative (first): `a_file_nobody_holds_is_not_in_the_set`, `a_file_outside_within_is_not_kept`,
  `a_shared_prefix_is_not_inside` (`/tmp/a/cache` held, `/tmp/a/cache2` asked).
- Positive: `a_file_held_by_a_child_process_is_found_with_its_name` (the test starts a child that opens a file in a
  temporary folder and waits), `a_working_folder_counts_as_held`, `a_file_mapped_without_a_descriptor_is_held` (the
  test maps a file and closes its descriptor), `the_current_user_owns_a_new_temporary_file`.
- Timing (recorded, not asserted): one snapshot on the developer's machine against the 01/02 numbers (descriptors
  7–19 ms; the region walk 0.4–0.9 s).
- `cargo nextest run -p fresher-macos`; Clippy; the "no unsafe outside `ffi`" test.

## Conclusions

- Built as written: `ffi/libproc.rs` (the process list, a process's owner, short name, executable path, vnode
  descriptors and their paths, working folder and mapped files; `proc_fileinfo`, `vnode_fdinfowithpath`,
  `proc_regioninfo` and `proc_regionwithpathinfo` defined from the SDK, their sizes checked at compile time; the two
  lists grown and read again, at most four times, when they fill the buffer), `MacOpenFiles` and `current_user()`.
- **This process is left out of the snapshot** (not in the step's text): the cleaner holds the verified item's handle
  open from check 5 to the act, so a snapshot refreshed meanwhile would name the app itself as the holder and skip
  every item `InUse`. The negative test `this_process_is_never_a_holder` holds it. The app maps nothing under a scan
  root, so nothing of its own is missed.
- **The mapped-files walk** is reused for 30 s for the same `within` (a monotonic `Instant`, not the core's `Clock`:
  it is a cache of the adapter, never a decision of the guard). A process that ended inside those 30 s still counts
  as a holder until the next walk: the safe side.
- Answers, not errors: another user's process (`EPERM` to `PROC_PIDTBSDINFO`) is counted and left out; a process
  that ended (`ESRCH`) is left out; a descriptor whose path is withheld (`EPERM`) is counted and skipped; the counts
  go to the debug log. Only a failure to list the processes is `OpenFilesError` (the guard fails closed).
- The name: the outermost `.app` folder in the executable's path (case ignored), else the short name, else
  "process N", looked up once per process and only for a process that holds something under `within`.
- Tests: children made by the test in its own temporary folder (`/bin/sh` that opens a file and changes folder, then
  becomes `sleep`, killed when the test ends). The mapped file is held by a child that is the test binary itself, run
  with an ignored helper test (`map_a_file_and_wait`) that maps the file, closes its descriptor, says so with a file
  beside it and waits for its input to close. Two more checks than listed: this process is never a holder, and the
  app name of a helper inside an app's frameworks.
- **Timing** (recorded, not asserted; this Mac, 3 050 held paths under the home folder, nothing read from it): the
  first snapshot, with the region walk, 556 ms (release) and 609 ms (debug); each later one, descriptors only,
  10.7–12.2 ms (release, median 11.0) and 17.5–17.8 ms (debug). Within the 01/02 numbers (7–19 ms; 0.4–0.9 s).
