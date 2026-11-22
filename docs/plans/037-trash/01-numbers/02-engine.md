# 037/01/02 — The engine's Trash view

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** engine, testing, e2e, app

`Engine::trash()` answers what waits in each Trash and how much of it the app's cleanups put there;
`Engine::open_trash(index)` opens one in Finder. `CleanupPorts` gains Finder's settings, wired at every site. The
design: [`../00-design.md`](../00-design.md), section 2 — read it whole before starting.

Read first: `crates/engine/src/cleanup/history.rs` (`page`, `Located` and its `state`, `waiting`, `in_trash_entries`),
`crates/engine/src/reveal.rs` (the shape of a command that opens Finder and its tests), `crates/engine/src/engine.rs`
(`CleanupPorts`, `Shared`, `Engine::cleanups`, `Engine::disk_state`), `crates/core/src/disk_state.rs` (`DiskState::find`,
`node`, `taken`; `NodeInfo::not_scanned`), `crates/engine/src/testing.rs` (`Fixture`, `ScriptedWalk::committing`),
`crates/testing/src/cleanup_fakes.rs` (`MemoryCleanupJournal`, `FakeCleaner::locating`),
`crates/testing/src/fixed_disk_state.rs`, `crates/testing/src/fakes.rs` (`FakeVolumes::space`).

## Files

- `crates/engine/src/engine.rs` —
  - `CleanupPorts` gains `pub finder: Arc<dyn FinderSettings>` (its doc: "Finder's 30-day setting, for the Trash
    line's date."), and the struct's doc names it.
  - `Shared` gains `pub(crate) trash_folders: Mutex<Vec<PathBuf>>`, doc "The Trash folders of the last `trash`
    answer, by index: what `open_trash` opens." (empty at start).
  - `pub fn trash(&self) -> TrashView` (doc: "What waits in each Trash and how much of it this app's cleanups put
    there: the last scan's measure, and the journal's items found in the Trash again now. Reads only; a journal that
    cannot be read counts nothing from cleanups.") and `pub fn open_trash(&self, index: u32) -> Result<(),
    CommandError>` (doc: "Opens Trash `index` of the last `trash` answer in Finder; `UnknownTrash` for an index it did
    not have. Opens, nothing more."), both delegating to `cleanup::trash`.
- `crates/engine/src/cleanup/history.rs` — `Located` and the methods `trash` needs become `pub(crate)`; add
  `pub(crate) fn waiting_items(&self) -> impl Iterator<Item = (u32, u64, &TrashedItem)>`: each item still in the
  Trash, once, with its allocated size and its first entry `InTrash`. Nothing else changes; the history's tests stay
  green.
- `crates/engine/src/cleanup/trash.rs` (new) — `pub(crate) fn view(shared: &Shared) -> TrashView` and
  `pub(crate) fn open(shared: &Shared, index: u32) -> Result<(), CommandError>`, per the design's section 2:
  1. The home Trash: `shared.ports.system.known_folders.locate(KnownFolder::Trash)`.
  2. Every cleanup of the journal: page through `journal.cleanups(offset, CleanupsQuery::MAX_LIMIT)` until `total`,
     read each with `journal.cleanup(id)`, build `Located::of`, and for each `waiting_items` entry add its size to the
     folder `landed.parent()` (an entry with no parent is skipped). Remember, per folder, the latest moment an item
     went: the cleanup's `end.at`, else `begin.at`; and whether its cleanup began at or after the state's `taken()`.
     A journal error stops the reading with a `tracing::warn!` (what was read so far counts). No cleanup ports: no
     share, only the scan's measure.
  3. Waiting: `shared` 's last disk state (as `Engine::disk_state` gets it); `find(folder)` → `node(id)` with
     `not_scanned == None` → its `sizes` plus the sizes counted from cleanups that began at or after `taken()`,
     raised to at least `from_cleanups` (`Sizes::checked_add`, saturating as `CleanupReport` does); otherwise `None`.
  4. A folder whose `waiting` is `None` or zero allocated and whose share is zero is left out. Order: the home Trash
     first, then the others by path; `index` is the place in the answer.
  5. `volume`: `cleanup.volumes.space(folder).ok().map(|space| space.name)`.
  6. `gone_by`: `finder.trash_auto_empty() == AutoEmpty::After30Days` and a share above zero → the latest moment plus
     `FINDER_KEEPS_FOR: u64 = 30 * 24 * 60 * 60` seconds (a named constant with its doc), kept only when after
     `shared.ports.clock.now()`. Finder is asked once per `view`, only when some line has a share.
  7. The folders of the answer replace `shared.trash_folders`.
  The state's `taken()` and the journal's times are both the core's clock (`Ports::clock`, which the scanner is
  given): compare them as they are. If a test shows they are not one clock, stop and report it.
  `open`: the folder at `index` of `trash_folders`, else `Err(CommandError::UnknownTrash { index })`; then
  `shared.ports.system_ui.open_folder(&folder)`, its error mapped to `EngineError::SystemUi(error).into()` as in
  `reveal.rs`.
  Keep the file small: the folder bookkeeping in one private struct (`Tally` or the like), no new public item.
- `crates/engine/src/cleanup/mod.rs` — `mod trash;` and its `pub(crate) use`; `#[cfg(test)] mod trash_tests;` if the
  tests go in their own file (`crates/engine/src/cleanup/trash_tests.rs`, as `storage/*_tests.rs` do).
- `crates/engine/src/testing.rs` — `Fixture` gets `.finder(FakeFinderSettings)` (default
  `FakeFinderSettings::new(AutoEmpty::Unknown)`) and passes it into `CleanupPorts`.
- Every other `CleanupPorts { .. }` site gets `finder`:
  - `app/tauri/src/compose.rs` (`cleanup(..)`): `Arc::new(MacFinderSettings::new(Arc::clone(&commands)))` from the
    composition's command runner (the same runner the system UI gets).
  - `app/tauri/src/sandbox.rs`: `MacFinderSettings::new(Arc::clone(&ports.commands))` — the Mac's runner in sandbox
    mode (only read), the fake runner in the sandbox autotest.
  - `app/tauri/src/testing.rs` (`..cleanup` keeps it), `crates/e2e/tests/places.rs`,
    `crates/e2e/tests/support/home_engine.rs`, `crates/e2e/tests/cleaning/mod.rs`,
    `crates/e2e/tests/jetbrains_home/mod.rs`: `Arc::new(FakeFinderSettings::new(AutoEmpty::Unknown))`.
  `grep -rn "CleanupPorts {" crates app/tauri/src` lists them; the build fails on any missed.

## Tests (`trash_tests.rs`, fakes only; `mod negative` first, then `mod positive`)

Use `Fixture` with `MemoryCleanupJournal` (records begun, items with `trashed` entries, ends, put backs),
`FakeCleaner::locating(landed, state)`, `FakeVolumes` with spaces named `Macintosh HD` and `Backup`, a
`FixedDiskState` committed by `ScriptedWalk::committing`, the known folders' Trash at `/home/.Trash`, a fixed clock.

Negative:
- `no_scan_and_an_empty_journal_give_no_line`
- `an_empty_trash_with_nothing_from_cleanups_gives_no_line` (the scan measured `/home/.Trash` at 0)
- `an_item_gone_from_the_trash_counts_nothing` (located `Gone`)
- `a_replaced_item_counts_nothing`
- `an_item_put_back_counts_nothing`
- `a_permanent_cleanup_a_tools_item_and_a_skipped_item_count_nothing`
- `a_trash_the_scan_did_not_measure_has_no_waiting` (node with `not_scanned`, and a folder no state holds)
- `a_journal_that_cannot_be_read_counts_nothing_from_cleanups` (the scan's measure still shows)
- `open_trash_of_an_index_never_answered_is_refused_and_opens_nothing` (`UnknownTrash`, `FakeSystemUi` has no call)
- `a_failed_open_is_an_internal_error`
- `finders_setting_off_or_unknown_gives_no_date`
- `a_date_already_past_is_not_given`

Positive:
- `the_share_counts_an_emptied_folder_once` (three entries of one item, one size)
- `items_moved_after_the_scan_are_added_to_its_measure`
- `items_moved_before_the_scan_are_not_counted_twice`
- `waiting_is_never_below_the_share`
- `a_second_volumes_trash_has_its_own_line_named_after_its_volume` (`/Volumes/Backup/.Trashes/501/x`, no `waiting`,
  `volume: Some("Backup")`, after the home line)
- `the_journal_is_read_past_one_page` (101 cleanups, each with one item in the Trash)
- `open_trash_opens_the_folder_of_its_line` (`SystemUiCall::OpenFolder("/home/.Trash")`)
- `finders_setting_on_gives_the_latest_end_plus_thirty_days`

## Checks

- `cargo nextest run -p fresher-engine`; `cargo nextest run -p fresher-app`; `cargo nextest run -p fresher-e2e`
  (compiles with the new field; nothing else changes).
- `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt --check`.
- `git diff 97d9b314 -- crates/cleaner crates/scanner crates/store crates/rules modules` empty.
- Coverage of `crates/engine/src/cleanup/trash.rs` at or above 90 % of lines (`cargo llvm-cov nextest -p
  fresher-engine`, read the file's line).

## Conclusions

Built: `crates/engine/src/cleanup/trash.rs` (`view`, `open`) and its tests in `trash_tests.rs` (12 negative, 9
positive after the fix round), `Engine::trash` and `Engine::open_trash`, `CleanupPorts.finder` and
`CleanupPorts.trashes`. `fresher-engine`: 452 tests; `fresher-app` and `fresher-e2e` 848 passed, 2 skipped.
`cleanup/trash.rs` covers 91.97 % of its lines (`history.rs` 98.6 %, the engine 98.3 %).

What changed from the design, and why:

- The scanner shares the composition's clock (`Scanner::new(..).with_clock(..)` in `compose.rs`): the scan's date
  and the journal's times must be one clock for "moved since the scan". The normal app is unchanged (both were the
  system clock); the sandbox autotest's 64 goldens did not move.
- Only entries whose landed parent is a known Trash count: the home's Trash, a volume's `<mount point>/.Trashes/<uid>`
  and `CleanupPorts.trashes` (the Mac's home Trash with `--real-trash`). A `LandedElsewhere` item outside them is no
  line (review finding; negative test).
- `waiting_items` keeps the journal's `Sizes`; `waiting` raised to `fromCleanups` compares allocated sizes.
- `MemoryCleanupJournal::unreadable()` was added to the fakes.
- Known issue written: [the sandbox clock restarts at each launch](../../known-issues/app-sandbox-clock-restarts-each-launch.md).

Left for later: whole-second timestamps can double count within one second; `trash()` reads the whole journal on
each call (the time on a long history is a fact to verify).
