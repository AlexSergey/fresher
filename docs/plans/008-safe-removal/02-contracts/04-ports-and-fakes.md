# 008/02/04 — Ports and fakes

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** core, testing, macos, engine, scanner,
app

## Files

- `crates/core/src/ports.rs` — the `Trash` port, changed:

  ```rust
  pub trait Trash: Send + Sync {
      /// Moves the item at `path` to the Trash of its volume; returns where it landed.
      fn move_to_trash(&self, path: &Path) -> Result<PathBuf, TrashError>;
      /// Moves `landed` back to `original`; never replaces anything at `original`.
      fn put_back(&self, landed: &Path, original: &Path) -> Result<(), TrashError>;
  }
  pub enum TrashError { NotFound, NoTrash, Occupied, NoParent, Refused { reason: String } }   // no path held
  ```

  and the new `OpenFiles` port: `fn snapshot(&self, within: &[PathBuf]) -> Result<OpenSet, OpenFilesError>`, where
  `OpenSet` holds `(path, process name)` pairs sorted by path and answers `holder_under(&Path) -> Option<&str>` by
  components.
- `crates/core/src/volume.rs` — `Volumes::available(&self, path: &Path) -> Result<ByteSize, ReadError>`.
- `crates/core/src/cleanup/port.rs` (new) — the `Cleaner` port (`rehearse`, `execute`, `locate`, `put_back`, as in
  the design, section 7), `CleanupContext<'a>` (roots, protection with the ignored paths, access status, guarded
  places, clock), `Depth` (`Quick`, `Full`), `Rehearsal` (`Ok(ItemIdentity)` or `Skip(SkipReason)` per item),
  `ItemDone` (one outcome, with the landed path for the journal), `TrashedItem`, `TrashState` (`in-trash`, `gone`,
  `replaced`), `PutBackOutcome`.
- `crates/core/src/cleanup/journal.rs` (new) — the records of the design, section 8 (`BeginRecord`, `ItemRecord`,
  `EndRecord`, `PutBackRecord`), `JournalState` (`Ready`, `ReadOnly { why }`), `JournalError`, and the
  `CleanupJournal` port (`state`, `begin`, `item`, `sync`, `end`, `put_back`, `cleanups`, `cleanup`).
- `crates/core/src/cleanup/sandbox.rs` (new) — `SANDBOX_MARKER` (`.fresher-sandbox`) and `SANDBOX_SIGNATURE` (its
  first line), shared by the cleaner and the fixtures.
- `crates/macos/src/volume.rs` — `MacVolumes::available` from the `statfs` it already reads (`f_bavail × f_bsize`).
- `crates/testing/src/fakes.rs` — `FakeTrash` changed: returns the landed path (`<trash>/<n>-<name>`), `put_back`
  with no replacement (`Occupied`), `without_trash()` (every move answers `NoTrash`), `refusing(path)` as now;
  `FakeOpenFiles` (a fixed list, or `failing()`); `FakeVolumes::available` (scripted values per call);
  `MemoryCleanupJournal` (records in memory, `read_only()`, `failing_after(n)`); `FakeCleaner` (records every plan it
  is given and answers scripted rehearsals and outcomes, for the engine's tests).
- `crates/testing/src/fixture_tree.rs` — every `FixtureTree` writes the sandbox marker at its root;
  `FixtureTree::without_sandbox()` for the tests that need its absence.
- `crates/engine/`, `crates/scanner/`, `app/tauri/src/adapters.rs` — adapt to the changed ports only (no behaviour).

## Produces

The ports plans 03, 04, 05 and 06 implement or call, and the fakes their tests run on.

## Checks

- Negative: `the_fake_trash_never_replaces_on_put_back`, `the_fake_trash_without_a_trash_moves_nothing`,
  `an_open_set_does_not_take_a_shared_prefix_for_inside`, `the_memory_journal_refuses_when_read_only`,
  `a_fixture_tree_without_sandbox_has_no_marker`.
- Positive: `the_fake_trash_returns_where_the_item_landed`, `the_fake_trash_puts_back`,
  `an_open_set_names_the_holder_of_an_item_inside`, `the_fake_cleaner_records_the_confirmed_plan`,
  `a_fixture_tree_holds_the_sandbox_marker`, `mac_volumes_answers_the_available_space` (a temporary folder).
- `cargo nextest run --workspace` and Clippy pass; no behaviour of the scan changes (the e2e goldens are unchanged).

## Conclusions

- Built: the `Trash` port changed, the `OpenFiles` port with `OpenSet`, `Volumes::available` (and
  `MacVolumes::available`: `f_bavail × f_bsize` from the `statfs` it already reads), `cleanup/port.rs`,
  `cleanup/journal.rs`, `cleanup/sandbox.rs`, and in `fresher-testing` the changed `FakeTrash`,
  `FakeVolumes::answering_available`, and `FakeOpenFiles`, `MemoryCleanupJournal`, `FakeCleaner` (in a file of their
  own, `cleanup_fakes.rs`). One more compile-fail case: `unconfirmed_plan_execute.rs` (an unconfirmed plan does not
  fit `Cleaner::execute`). No behaviour of the scan changes: every e2e golden is unchanged.
- **The `Trash` port carries the design's call 37** (from plan 01's measurements, merged before this step):
  `move_to_trash(path, held: &File) -> Result<Landed { at, reported }, TrashError>`. `at` is where the item of the
  handle the cleaner holds is after the call (`F_GETPATH` in `fresher-macos`); `reported` is the call's own landed
  path, which the system sometimes leaves out. `TrashError` holds no path: `NotFound` (Cocoa 4, −43), `NoTrash` (3328,
  −120), `NotAllowed` (513, −5000: a locked item, a read-only folder, a volume that refuses a Trash folder),
  `Occupied` and `NoParent` (put back), `ThroughLink` (put back with `RENAME_NOFOLLOW_ANY`, `ELOOP`), `Refused {
  reason }` for anything else. `FakeTrash` answers `Landed` with both paths, or without `reported`
  (`reporting_nothing()`), refuses with `NotAllowed` (`refusing`), moves nothing without a Trash (`without_trash()`),
  and puts back without replacing (`Occupied`, `NoParent`).
- Changed from the step, each the simpler or the safer way:
  - **The sandbox marker is opt-in**: `FixtureTree::with_sandbox()` writes it at the root and `mark_sandbox(folder)`
    in a folder of the tree; a tree is built without it. Written into every tree, it was counted by the scanner's,
    the reference walk's and the listing's tests and changed five golden files; a tree without a marker is the safe
    default (the cleaner removes nothing in it). The check `a_fixture_tree_without_sandbox_has_no_marker` holds.
  - **`CleanupContext` holds no clock**: `{ roots, protection, access, guarded }`, borrowed; the clock is one of the
    cleaner's own ports (04/01's `CleanerPorts`). `roots` are the scan roots in the settings now; `rehearse` takes the
    roots of the items' scan as an argument and `execute` reads them from the confirmed plan.
  - **`ItemDone` carries the finding and its sizes** besides the index, the outcome, `trashed` and a tool's
    `status`: the plan goes to the cleaner by value, so the engine has nothing else to build the report and the
    events from. `ItemDone::report()` and `record(cleanup)` give the event's and the journal's forms.
  - **`execute` takes `out: &mut dyn FnMut(ItemDone)`** and answers `Ended` (`Completed` or `Cancelled`); a failed
    journal append stops the cleanup through the cancel token (06/02), and the engine reports it `Stopped`.
  - **`Rehearsal::Passed(Option<ItemIdentity>)` / `Skipped(SkipReason)`** (no identity for a tool's item).
  - **`locate(&[TrashedItem]) -> Vec<TrashState>`**, as 04/07 has it; `TrashedItem { original, landed, identity }`
    is also each emptied folder's entry.
  - **The journal port**: `begin(&BeginRecord) -> Result<CleanupId, _>` (the store numbers cleanups, 05/01),
    `cleanups(offset, limit) -> JournalPage { cleanups, total, skipped }`, `cleanup(id) -> Option<RecordedCleanup>`.
    The records are plain data with public fields; the store writes its own serde forms (05/01).
    `BeginRecord::new(at, &ConfirmedPlan, space)` takes every item from the plan (path, identity, action, category,
    the finding's identity as `key`, the reason's text, danger, sizes). `RecordedCleanup::summary()` gives the
    history's `CleanupSummary` (an item with no outcome is `Unknown`, a cleanup with no end `Interrupted`; `inTrash`
    counts what was moved and not put back, which the engine checks with `locate`). For it, `OutcomeCounts::add`
    takes a `RecordedOutcome`, `OutcomeCounts` gained `unknown`, and `CleanupReport::count_recorded` was added.
  - `OpenFilesError { reason }` is one struct; `OpenSet::new` sorts by path components, so `holder_under` is one
    binary search (a property test against a plain scan).
- The fakes: `FakeOpenFiles` (`holding`, `hold` between snapshots, `failing()`, a count of snapshots),
  `MemoryCleanupJournal` (`read_only()` answers `NewerVersion`, `failing_after(n)`, `syncs()`, `recorded()`),
  `FakeCleaner` (`skipping`, `ending`, `locating`, `putting_back`; records each rehearsal's findings and each executed
  plan's method and items; by default a path item moves to `/fake-trash/<index>` or is deleted, a tool's is done).
- `docs/architecture/README.md` ("Ports") is left to plan 08, which owns it; the new ports are listed there then.

