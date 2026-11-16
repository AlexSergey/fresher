# 004/03/04 — The change journal

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-15 · **Modules:** macos

## Files

- `crates/macos/build.rs` or `#[link(name = "CoreServices", kind = "framework")]` in `ffi/fsevents.rs` — linking the
  framework.
- `crates/macos/src/ffi/fsevents.rs` — our own declarations of the FSEvents functions 01/02 used:
  `FSEventsGetCurrentEventId`, `FSEventsCopyUUIDForDevice`, `FSEventStreamCreate` (or `…RelativeToDevice`, as 01
  decided), `FSEventStreamSetDispatchQueue`, `FSEventStreamStart`, `FSEventStreamFlushSync`, `FSEventStreamStop`,
  `FSEventStreamInvalidate`, `FSEventStreamRelease`; the libdispatch queue functions; the few CoreFoundation calls
  (a `CFString` for a path, a `CFArray` of one, `CFRelease`, the UUID bytes). Each type that owns a CF object has a
  `Drop` that releases it, so no path leaks one.
- `crates/macos/src/journal/stream.rs` — a safe `HistoryStream`: created for one root path since an event id, started
  on its own serial queue; the callback (an `extern "C" fn`, its context a `Box` the stream owns) copies each event's
  path and flags into a `Mutex<Vec<…>>` and signals when `HistoryDone` arrives. `collect(cancel)` flushes, waits for
  `HistoryDone` or the cancel token (checked every 50 ms), stops, invalidates and releases.
- `crates/macos/src/journal/mod.rs` — `MacChangeJournal` implements `ChangeJournal`:
  - `position(volume)` → `JournalPosition { journal: the journal's UUID for the device, cursor: the current event id }`;
    `None` for a volume with no journal (01/02, question 8).
  - `changes_since` → a different journal UUID is `Reset`; the flags 01 listed map to the gaps
    (`UserDropped`/`KernelDropped` → `Dropped`, `EventIdsWrapped` → `Wrapped`, `RootChanged` → `RootChanged`, a
    history older than the journal → `TooOld` if 01 found a way to tell); `MustScanSubDirs` puts the path in
    `subtrees`; every other event's folder goes in `folders` (an event on a file names its folder). Paths are
    deduplicated; `end` is the position taken when the call started, so nothing between the call and the next scan is
    lost.
- `crates/macos/src/lib.rs` — `pub use journal::MacChangeJournal`.

## Produces

The journal the rescan (07) uses in the app and in its integration tests.

## Checks

- Negative: `another_journal_id_is_a_reset` (a position with a made-up journal id), `a_volume_without_a_journal_has_no_position`
  (a FAT disk image, `#[ignore]` without `hdiutil`), `a_cancelled_collect_returns_cancelled_and_releases_the_stream`,
  `changes_outside_the_root_are_not_reported`.
- Positive, each on a fresh temporary folder (position, change, `changes_since`): `a_new_file_reports_its_folder`,
  `a_file_growing_in_place_reports_its_folder` (the case 01/02 confirmed), `a_removed_folder_reports_its_parent`,
  `a_renamed_folder_reports_both_parents`, `nothing_changed_reports_nothing`.
- No stream outlives its call (a test counts the live streams through a debug counter in the stream type).
- `cargo nextest run -p fresher-macos`; if a journal test is flaky on CI despite `FlushSync`, it is recorded in
  `docs/known-issues/` with what was tried, not retried in a loop.

## Conclusions

- `ffi/fsevents.rs` declares the FSEvents, `CoreFoundation` and libdispatch calls itself and links both frameworks with
  `#[link(kind = "framework")]` (no build script). `FSEventStreamCreate` (not `…RelativeToDevice`), folder-level,
  `NoDefer | WatchRoot`, latency 0, on its own serial queue. Every owned object has a `Drop`: `CfOwned` releases a
  `CoreFoundation` object; `Stream` stops, invalidates, drains its queue (`dispatch_sync_f`, so no callback runs once
  its inbox is gone), then releases the stream and the queue. The callback copies paths, flags and ids into a
  `Mutex<Vec<…>>` and signals a `Condvar`.
- **Changed order in `collect`:** it waits for `HistoryDone` (checking the cancel token every 50 ms, and a time limit),
  then calls `FlushSync`, then releases the stream. `FlushSync` first, as written, blocks for as long as the replay
  takes, out of reach of the cancel token (measured: 70 s for a replay from the journal's start); flushing after
  `HistoryDone` still delivers the changes of the last ~100 ms that 01 saw arrive late.
- `MacChangeJournal::new()` replays each volume whole, from its mount point (found among the local mounts by its
  volume id); the events come back with the paths the user sees (on the data volume `/private/var/…`, `/Users/…`, not
  `/System/Volumes/Data/…`), checked here. `MacChangeJournal::under(paths)` replays only those paths, which is what the
  tests use and what keeps a replay small when the roots are known. `with_time_limit` (default 10 minutes) turns a
  replay that does not finish into `JournalGap::Failed`; 07 sets the real budget.
- `position` is `None` for a volume with a made-up id (step 03), one that is not mounted, or one with no journal
  (a FAT image mounted `-nobrowse`); `changes_since` answers `NoJournal` for them. The gap flags map as written;
  `TooOld` is never answered (01 found no way to tell). A watched root's parent may be reported (a root moved): it is
  passed on, for the scanner's root check.
- The tests take the position only once the setup's own events are in the journal (`settled`: a marker file is
  written and the replay polled until it shows), so no test sleeps. A test that replays a whole busy volume accepts
  `Dropped` (other tests writing in parallel can overflow it; a gap is the safe answer). Every journal test checks
  that no stream is alive after the call. No flaky test was seen in repeated runs, so nothing went to
  `docs/known-issues/`.
- Checks: 18 new tests; `cargo llvm-cov` on the crate: 96.1 % of lines.

