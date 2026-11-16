# 004/02/04 — The walk request and commit

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** core, engine, testing, app

## Files

- `crates/core/src/walk.rs` — `Unreadable::NeedsFullDiskAccess` is renamed `Unreadable::Guarded` (refused by the
  system's privacy protection, `EPERM`), and `NoPermission` is documented as POSIX permissions (`EACCES`): the names
  plan 006 uses, settled here because the core is frozen after this plan. The port's new shape:

  ```rust
  pub trait DiskWalk: Send + Sync {
      fn walk(&self, request: &WalkRequest<'_>, cancel: &CancelToken,
              on_match: &mut dyn FnMut(Match), on_progress: &mut dyn FnMut(WalkProgress))
          -> Result<WalkSummary, WalkError>;
      /// Records the scan's claims and keeps its state; called only after a walk that was not cancelled.
      fn commit(&self, claims: &[ClaimedPath]) -> Result<Arc<dyn DiskState>, WalkError>;
  }
  pub struct WalkRequest<'a> { pub roots: &'a [PathBuf], pub interests: &'a InterestSet,
                               pub avoid: &'a [PathBuf], pub mode: WalkMode }
  pub enum WalkMode { Full, Auto, Restore }
  pub struct ClaimedPath { pub path: PathBuf, pub category: CategoryId }
  ```

  `WalkSummary` gains `kind: ScanKind`, `fallback: Option<FullScanReason>` and `not_scanned: BTreeMap<NotScanned, u64>`
  (avoided places and other volumes next to the unreadable counts). `WalkError` gains `NothingToRestore` and
  `Commit { kind: io::ErrorKind }`.
- `crates/core/src/event.rs`, `crates/core/src/command.rs` — `ScanKind { Full, Rescan { changed_folders: u32 },
  Restored { taken: Timestamp } }` (IPC type); `ScanFinished` and `LastScan` gain `kind`. `FullScanReason` (no cache,
  broken cache, other fingerprint, other roots, other avoid list, other volume, journal gap, too many changes, check
  failed, asked by the user) is a core type, logged by the engine, not sent to the UI.
- `crates/testing/src/naive_walk.rs` — adapted: `Full` and `Auto` walk the disk (always `ScanKind::Full`), `Restore`
  returns `NothingToRestore`; `avoid` is honoured (an avoided folder is not entered, counted in `not_scanned`);
  `commit` returns a `FixedDiskState` built from what it walked (paths and sizes of the folders, no kept files).
- `crates/engine/src/session.rs`, `engine.rs` — adapted to the request: roots from the settings, `avoid` empty,
  `mode: Auto`; `commit` called with no claims after a walk that was not cancelled, its state kept in the results
  (`Engine::disk_state()`); a commit error is logged and the scan ends `Partial`. Claims, restore and modes come in
  05/05 and 07/04.
- `app/tauri/src/adapters.rs` — `NoWalk` adapted (its `commit` returns an empty state); the hack stays until 05/06.
- `app/ui/src/ipc/bindings.generated.ts` — regenerated: `ScanKind`, `LastScan.kind`, `ScanFinished.kind`.
- `docs/architecture/README.md` — the ports table lists `FolderReader`, `ChangeJournal`, `Volumes`, `ScanCache` and
  `DiskState` with their implementers and fakes.

## Produces

The port the scanner implements (05), with the modes 07 fills in; the engine's hold on the disk state (08).

## Checks

- Negative: `a_cancelled_scan_is_not_committed` (engine, over a fake walk that records `commit`),
  `the_reference_walk_has_nothing_to_restore`, `the_reference_walk_does_not_enter_an_avoided_folder`,
  `a_failed_commit_ends_the_scan_partial`.
- Positive: `a_completed_scan_is_committed_and_its_state_kept`, `scan_finished_carries_the_scan_kind`,
  `last_scan_serializes_its_kind`.
- `cargo nextest run --workspace` (the port change touches every user of `DiskWalk`); the core e2e
  (`crates/e2e/tests/demo.rs`) and its golden file pass, the golden file updated only by the new `kind` field;
  `npm run test` in `app/ui` passes with the regenerated bindings.

## Conclusions

- `DiskWalk` takes a `WalkRequest { roots, interests, avoid, mode }` and has `commit(claims)`; `WalkMode`,
  `ClaimedPath` and `FullScanReason` are in `walk.rs`. `Unreadable::NeedsFullDiskAccess` is now `Unreadable::Guarded`
  (`EPERM`), `NoPermission` is documented as `EACCES`. `WalkError` gained `NothingToRestore` and
  `Commit { kind }`. `WalkSummary` gained `kind`, `fallback` and `not_scanned`.
- Not spelled out by the step, taken the simplest way:
  - `WalkSummary::is_complete` still looks only at `unreadable`: an avoided place or another volume is not entered
    on purpose, so it does not make the scan `Partial` (until plan 006 every scan avoids places).
  - `FullScanReason::JournalGap` carries the `JournalGap`; the engine logs the reason with `tracing::info!`.
  - `ScanKind` lives in `event.rs`, serialized `{ kind: "full" | "rescan" | "restored" }` with `changedFolders` and
    `taken`; it defaults to `Full`, which the engine reports when the walk gave no summary.
  - The engine commits only after a walk that returned a summary and was not cancelled — also not after a walk that
    stopped with an error (a root that could not be read), so a broken walk never replaces the kept state. The commit
    runs after every module's `finish` (the claims of 05/05 come from the findings). `Engine::disk_state()` returns
    `Result<Option<Arc<dyn DiskState>>, CommandError>` like the other getters; the state is kept in the results until
    another scan commits one.
  - `NaiveWalk` is no longer `Clone` (it keeps the last walk's folders for `commit` behind a `Mutex`). An avoided
    folder is listed (counted as an entry), not matched and not opened; an avoided root is not entered at all. A
    folder on another device is counted in `not_scanned` as `OtherVolume`. Its committed state holds every folder
    with its size (an unreadable or avoided one `Partial`, which makes its ancestors `Partial`), the reasons of the
    folders not scanned, the owners of the claims, ids counting the commits, and the time 0 (it has no clock).
  - The app's `NoWalk` commits an `EmptyState` written in `adapters.rs` (the app does not depend on
    `fresher-testing`); the shell's and the e2e's test walks commit an empty state too.
- Regenerated: the bindings (`ScanKind`, `LastScan.kind`, `ScanFinished.kind`, `Unreadable` with `guarded`) and the
  golden file (`kind: full` on `ScanFinished` and `lastScan` only). The UI's test events gained `kind`.
- `docs/architecture/README.md` lists the five new ports with their implementers and fakes.
