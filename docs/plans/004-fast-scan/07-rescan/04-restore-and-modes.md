# 004/07/04 — Restore at launch and scan modes

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner, engine, app, e2e

## Files

- `crates/scanner/src/scanner.rs` — `WalkMode::Restore`: load and validate the cache (the same reasons as 07/01,
  except the journal: nothing is asked of it), `emit` from the tree, `ScanKind::Restored { taken }`; no folder is
  read (a test with a reader that fails every call proves it). Nothing to restore → `NothingToRestore`.
- `crates/engine/src/engine.rs` — `Engine::start_scan(mode: ScanMode)` with `ScanMode { Auto, Full }` (the command
  passes `Auto`; a full-scan choice in the UI is plan 003's); `Engine::restore()`: the same operation as a scan
  (refused while one runs), in `WalkMode::Restore`, running only the enabled modules that declare interests; the
  modules with none (they ask an external tool) do not run, so `ScanStarted.modules` lists only the ones that do.
  `NothingToRestore` ends quietly: no `ScanFinished`, the engine stays without a last scan.
- `crates/engine/src/session.rs` — the plan carries the mode. In `Restore` mode the session first collects the
  walk's matches (thousands at most, from memory) and publishes `ScanStarted` only once the walk has returned them,
  then runs the modules over them; so `NothingToRestore` publishes nothing at all. The outcome of a restore is
  `Completed`, labelled through `LastScan.kind = Restored { taken }`. The engine commits as usual to get the
  `DiskState`; the scanner records the claims but does not save a restored state (it is already the cache's current
  generation, and saving it would push the real previous one out).
- `crates/core/src/command.rs` — no change: `start_scan` keeps its input; the engine API takes the mode.
- `app/tauri/src/compose.rs` — the scanner gets `FileScanCache` (on the data folder) and `MacChangeJournal`.
- `app/tauri/src/lib.rs` (the setup hook) — `engine.restore()` once the window is up; the UI learns of it through the
  usual events.
- `crates/e2e/tests/rescan.rs` (new) — the engine with the real scanner (`StdReader`), `FileScanCache` on a temporary
  data folder and `FakeChangeJournal`: a scan, a scripted change (a cache folder grows), a rescan → only the changed
  folder read, the category total grown, `ScanKind::Rescan`; a new engine on the same data folder → `restore()`
  gives the same findings without reading the disk; a journal gap → a full scan with its fallback logged. Against
  `fixtures/golden/rescan.json`.

## Produces

The behaviour the user sees: the last result at launch, labelled with its date; a quick rescan on "Scan".

## Checks

- Negative: `a_restore_reads_no_folder`, `a_restore_runs_no_module_without_interests` (a test module that counts its
  `finish` calls through a fake command runner: zero), `a_restore_during_a_scan_is_refused`,
  `nothing_to_restore_leaves_the_engine_idle_without_a_last_scan`, `a_cancelled_rescan_keeps_the_previous_cache`.
- Positive: `a_restore_shows_the_last_findings_with_their_date`, `a_second_scan_is_a_rescan`,
  `start_scan_full_ignores_the_cache` (e2e and engine tests).
- `npm run pre-push` passes; by hand in `npm run tauri dev`: a restart shows the last result at once with its date; a
  second "Scan" is quick; the times are written in the notes, with no path.

## Conclusions

- `WalkMode::Restore` in `Scanner`: the roots are located without the reader (`Roots::locate`: canonical paths and
  volumes only), the cache is loaded and checked with the reasons of 07/01 except the journal's, the matches are
  emitted from the tree and the summary rebuilt from it with `ScanKind::Restored { taken }`. Any reason is
  `NothingToRestore` (logged). The restored tree is committed with the date it was taken and not saved again.
- `Engine::start_scan(ScanMode)` (`Auto` or `Full`, exported by the engine; the command passes `Auto`) and
  `Engine::restore()`, the same operation as a scan: refused while one runs, its plan only the enabled modules that
  declare interests. The session collects a restore's matches before anything is published: `NothingToRestore` (or
  any error of the walk) sends the engine back to idle with no event and no last scan. A restore then runs like a
  scan; its outcome follows the usual rule (`Completed` when the restored state was complete), labelled
  `LastScan.kind = Restored { taken }`.
- The app restores in the setup hook, right after composing the engine; the UI learns of it through the events and
  the overview. The scanner gets `FileScanCache` on the data folder and `MacChangeJournal` (07/01).
- e2e (`crates/e2e/tests/rescan.rs`, the shared harness moved to `tests/app/mod.rs`): a scan, `out.o` grown in place
  in a folder claimed whole, a rescan (`Rescan { changed_folders: 2 }`: the folder and its parent) with the finding
  grown by the difference, then a new engine on the same data folder whose reader refuses every call restores the
  same probe findings without one call (the demo module has no interests: it does not run). A journal gap is a full
  scan; `Full` never asks the journal; a rescan cancelled while it reads leaves the cache as it was. Golden:
  `fixtures/golden/rescan.json` (progress events left out: their count depends on the threads).
- **Changed from the step**: the e2e cannot hold "only the changed folder read": the check reads up to 32 unchanged
  folders, and the fixture home has fewer, so it reads them all; the scanner's bars on `SMALL` (07/03) hold the
  counts. `a_restore_runs_no_module_without_interests` counts the module's `finish` calls directly, not through a
  fake command runner. The restore bench (`scan/restore/large`, bar 0.2 s) is covered by its parts: decode 10 ms
  (07/01) and the emit, which reads no disk; no group of its own was added.
- **Left for the user, by hand** in `npm run tauri dev`: a restart shows the last result at once with its date; a
  second "Scan" is quick; note the times (no path).
