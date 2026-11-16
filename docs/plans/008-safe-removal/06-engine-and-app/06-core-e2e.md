# 008/06/06 — The core end to end

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** e2e, testing

## Files

- `crates/e2e/Cargo.toml` — `fresher-cleaner` and `fresher-module-trial` (the e2e crate may depend on anything).
- `fixtures/homes/cleanup.toml` (new) — the home: folders the `trial` module claims (05) — caches to trash, one nested
  in another, a folder to empty and keep — a link from one of them to outside the root, a folder standing in for a
  protected place (the test's `Protection` names it), a hard link shared with an unclaimed folder; the fake runner
  lists one tool object.
- `crates/e2e/tests/cleanup.rs` (new) — the engine with the reference walk, the `trial` module, the real `DiskCleaner`
  (`Reach::Sandbox`, `FakeTrash` in the fixture, `FakeOpenFiles`, `FakeCommandRunner`), `FileCleanupJournal` on a
  temporary data folder, `FakeVolumes`:
  1. **the whole path**: scan, select a category and a finding, prepare (the nested folder left out as covered),
     confirm, execute, the journal read back, the overview without the removed findings, put back one item, the
     history;
  2. **cancelled**: a cancel after the first item — the rest not reached, the journal and the tree as the golden says;
  3. **refused**: an echo with a wrong count — nothing moved, nothing journalled;
  4. **changed under it**: an item replaced between prepare and confirm — skipped as `Replaced`, the rest done;
  5. **permanent**: the same selection deleted permanently — nothing in the fake Trash, the folders gone, the link's
     target outside untouched.
- `fixtures/golden/cleanup.json` (new) — each case's events, journal records (paths relative to the fixture root) and
  the tree after, reviewed like every golden file.

## Produces

The proof that selection → plan → confirmation → cleaner → journal → history → put back works as one, on files.

## Checks

- `cargo nextest run -p fresher-e2e`; `UPDATE_GOLDEN=1` writes the golden file, whose diff is reviewed.
- Outside the fixture root nothing changes in any case (the test lists the outside target before and after).

## Conclusions

- Built: `crates/e2e/tests/cleanup.rs` (nine tests), its harness `crates/e2e/tests/cleaning/mod.rs`,
  `fixtures/homes/cleanup.toml` and four golden files. The engine scans with the reference walk and cleans through the
  real `DiskCleaner` (`Reach::Sandbox`, `FakeTrash` in the fixture, `FakeOpenFiles`, `FakeCommandRunner` listing one
  tool object), `FileCleanupJournal` in a temporary data folder, `FakeVolumes` and a `FakeClock`. Every run is the same
  (checked five times in a row); the goldens carry no wall clock, no machine path (the fixture is written `~`) and no
  device or file numbers (the journal's identities are left out).
- **The cases**, negative first: a wrong count (`PlanMismatch`, the plan gone after it), a plan older than its lifetime
  (the clock moved past `PLAN_LIFETIME`, `PlanGone`), a home with no real marker (every item left out `NotInSandbox`,
  the confirmation refused `NothingToClean`), a data folder that cannot be written (`CleaningBlocked { journal }`) and a
  full disk (`CleaningBlocked { disk-full }`): each with the tree as before, no Trash and no cleanup in the history.
  Then the whole path (`cleanup-whole-path.json`: the plan, the events, the findings and the overview after, the
  history, the detail, put back one item and again (`PutBack`, journalled once), the journal, the tree after the cleanup
  and after the put back), a cancel after the first item (`cleanup-cancelled.json`: the rest `NotReached`, the journal
  closed `cancelled`), an item replaced between prepare and confirm (`cleanup-replaced.json`: skipped `Replaced`, the
  rest done) and the same selection deleted permanently (`cleanup-permanent.json`: nothing in the Trash). In every case
  the folder outside the scan root, which a link of the emptied folder points into, is listed before and after, and the
  hard link's other name survives.
- Changed from the step, each the simpler way:
  - **One golden file per case** (`fixtures/golden/cleanup-<case>.json`), not one `cleanup.json`: the cases are tests
    of their own, run in parallel, and `UPDATE_GOLDEN=1` would race on a shared file.
  - **The nested folder left out as covered** cannot come from `trial` (its rules claim a folder whole, so the walk
    never enters one, 05). The test registers a small module of its own, `build`, that claims `fresher-e2e-build`
    folders as an item (`Claim::Item`, the walk goes on inside); a trial cache sits in it next to a marker of its own.
    The selection is the three trial categories and the build folder by its id: the cache inside is left out
    `Covered`. An item claim counts only the entry, so the build folder's size is 0 in the plan; no real module claims
    a removal that way.
  - **The protected place**: a marked `home/protected` named `never` by the cleaner's protection. Its trial cache is
    left out `NotInSandbox`, not `Protected`: a marker counts only where the system's protection refuses nothing, and
    the folders around a protected place are refused as its ancestors. The scan root is `home/` of the fixture, so the
    link's target (`outside/`) and the Trash (`trash/`) sit outside the root.
  - **The cancel** is made at a known point: the test's `PausingCleaner` wraps the real cleaner and, after the first
    item, waits until `Engine::cancel_cleanup` sets the token; the events are batched on the fake clock, so they come
    as one batch at the end.
  - **The full disk** is the real journal whose `begin` answers `NoSpace` (a wrapper in the harness): a disk image too
    small for one record is not worth its cost here; the unwritable journal is the real one in a folder made `0500`.
- **Found by the e2e, fixed after it** in the engine (`75c1ddd`, each with a failing test first; the goldens
  regenerated: `findings-removed` gains the covered finding, the whole path's overview counts one cache less and its
  detail and second put back read `put-back`, one put-back record fewer):
  - A finding left out `Covered` stayed in the results after the item that held it went. Now a cleanup drops every
    finding at or below the path of an item that went whole (moved, deleted, or emptied with no entry left), in the
    totals, `found`, the claims and `FindingsRemoved` (the removed ones first, then those below, in id order). An
    item skipped, failed or emptied with entries left takes nothing below it: what may still be there stays listed.
  - A plan whose every item is left out (count 0) could be confirmed and journalled an empty cleanup. **Chosen**: the
    plan is still prepared, so its view shows why each item was left out (the "no sandbox" case is where that is
    needed), and its confirmation is refused with `NothingToClean`: the plan is consumed, nothing is acted on, no
    journal record, no event. The UI should not offer to confirm a plan of count 0.
  - A second put back of an item answered `Gone` and journalled it, so the detail of an item back in its place read
    `Gone`. **Chosen**: an item put back already answers `PutBack` again, with no attempt and no record, so the
    journal's last answer for it stays the one that put it back. The core has no "already back" outcome and is frozen.
