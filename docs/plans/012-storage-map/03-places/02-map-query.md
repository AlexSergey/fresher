# 012/03/02 — The map's query

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** engine

## Files

- `crates/engine/src/storage/query.rs` (new) — `impl Engine`:
  - `pub fn storage_map(&self) -> Result<StorageMap, CommandError>` — the three phases of the design's section 6:
    - **`NoScan`**: the configured scan roots (`~` resolved through the known folders, as the scan resolves them),
      grouped by volume (`Volumes::identify(root).id`), each volume's map with `NotScannedYet` as its used space;
    - **`Scanning`**: each finding of the running scan (its counted size, ignored findings left out) assigned to the
      root that holds its path, so to that root's volume; `Explained` is their sum, `NotScannedYet` the rest;
    - **`Complete`**: the last committed state's roots grouped by volume, `Shares` and `AppPlaces` of that state, the
      unreadable places on each volume.
    The home's volume comes first, then the others in the order of their first root.
  - `pub fn storage_meaning(&self, volume: usize) -> Result<Vec<MeaningGroup>, CommandError>` — the by-meaning groups
    of one volume of the complete map (01/04 with 02/04's parts); `NoScan` before a commit, `UnknownVolume` (a new
    `CommandError` variant with its text) for an index past the list.
  - `pub fn storage_place(&self, id: NodeId, offset: u32, limit: u32) -> Result<PlaceView, CommandError>` — 01's view;
    `StaleNode` for a node of another state, `limit` capped at 500 like every disk query.
  Every query reads the state's `Arc` outside the engine's lock, as `disk.rs` does: none blocks a scan.
- `crates/engine/src/results.rs` — beside the committed state: its `Shares` (built on the first map query after the
  commit, or after the ignore lists change, in a `OnceLock` reset by both) and the volume facts read at the scan's start,
  at its commit and at the restore (`Volumes::space` per volume; a failure leaves that volume without facts, logged).
  The `Volumes` is the one in `CleanupPorts` (design, call 11).
- `crates/engine/src/engine.rs` — the volume facts read at the scan's start and commit, through the session's existing
  hooks (no new thread).
- `crates/engine/src/lib.rs` — nothing beyond 01/03's `mod storage;` unless a type is re-exported.
- `crates/core/src/error.rs`, `crates/core/locales/en.json` — `CommandError::UnknownVolume` and its text.
- **Measured** (rule 4): `Shares::of` on the developer's home state (restored from the cache), its time and heap, in
  the step's conclusions; the budget is 50 ms and 5 MB (design, section 7). A hand-run test in release
  (`crates/e2e/tests/storage_budget.rs`) holds it within the budget over a state the real scanner commits on
  `LargeTree::LARGE`.

## Produces

The engine API the commands (04/01) call.

## Checks

- Negative first: `the_map_before_any_scan_is_no_scan_with_the_volume`, `a_place_of_an_older_state_is_stale`,
  `an_unknown_volume_is_refused`, `a_failed_volume_read_gives_the_scanned_parts_only`, `ignored_findings_are_not_explained_while_scanning`,
  `a_cancelled_scan_keeps_the_last_map`, `the_volume_is_not_read_per_query` (`FakeVolumes` counting `space` calls: four
  map queries during a scan read it once).
- Positive: `explained_grows_while_the_scan_streams` (a `ScriptedWalk` held by its `Gate`: two batches of findings,
  `Explained` grows, `Explained + NotScannedYet` stays the used space), `the_commit_gives_the_complete_map`,
  `roots_on_two_volumes_give_two_maps_home_first`, `ignoring_a_category_moves_its_bytes_to_your_folders_without_a_rescan`,
  `the_two_views_have_one_total` (the sum of `storage_meaning`'s leaves equals the sum of the map's parts),
  `storage_place_descends_from_the_home`.
- `cargo nextest run -p fresher-engine -p fresher-core`.

**Changed by 04/05:** the place view takes `descent: Descent` (`through` | `stay`): `through` opens down the folders that are almost all one child, `stay` opens exactly the folder asked (a step of the breadcrumb).

## Conclusions

Built in `290473b`, fixed in `2eeb34e` and `1d8e09e` after review, final wave `f38e356`.

- Three queries in three phases, one snapshot deciding the phase. Meaning is scoped to one volume's roots, so each
  view adds up to its volume. The volumes of the committed state are kept beside it (never a new scan's findings with
  the old state). Places are refused while a scan runs. Volumes are read through `CleanupPorts`, leaving 007's
  `compose.rs` alone; they are read before `begin()` and stored with it under one lock.
- Review changed: the shared volumes slot (a cancelled scan of other roots changed the map) and the budget, first
  measured on a stand-in tree. A place page holds at most 500 rows. Largest folders are only folders.
- **Measured, generated LARGE tree** (`TreeSpec::LARGE`, 3 million entries, committed by the real scanner with every
  dependency tree and cache folder claimed: 1 025 owned nodes), release build on the developer's Mac:
  `Shares::of` in 3.9 ms, 625 226 heap bytes (the hash tables' bucket count, not their capacity), against 50 ms and
  5 MB. The measurement on the developer's home, restored from the app's cache, is left for the finish run by hand
  (06/02).
