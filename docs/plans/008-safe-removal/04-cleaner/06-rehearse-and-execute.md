# 008/04/06 — Rehearse and execute

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** cleaner

## Files

- `crates/cleaner/src/pipeline.rs` (new):
  - `rehearse(items, cx, depth) -> Vec<Rehearsal>`: the open-files snapshot once, then `verify` for each item in
    order; the `Verified` values are dropped at once; nothing else is called. It takes the items of an unconfirmed plan
    and has no `Act`, so it cannot call a removal function (the types hold it).
  - `execute(confirmed, cx, cancel, out)`: `let act = confirmed.act();` then, item by item in plan order: the cancel
    token (after it is set, every remaining item is `NotReached`), the snapshot refreshed when two seconds have passed
    on the `Clock`, `verify` with `Depth::Full`, the removal of its action (`Remove` → `to_trash` or `delete` with the
    consent of the method, `Empty` → `empty`, `Tool` → `run_tool`), then `out(ItemDone)` before the next item.
  - A snapshot that cannot be taken fails the whole cleanup closed: every item `Skipped { CheckFailed }`, nothing
    acted on (the engine refused it already in 06/02; this is the second line).
- `crates/cleaner/src/lib.rs` — `DiskCleaner` implements `Cleaner::{rehearse, execute}` with the pipeline.

## Produces

The cleaner's two entry points for the engine (06/01, 06/02) and the dry run (`Depth::Full` rehearsal) for plan 007.

## Checks

- Negative (first): `a_rehearsal_changes_nothing` (the fixture tree lists exactly the same before and after, for
  every kind of action and method), `a_cancel_leaves_every_later_item_not_reached`,
  `an_item_that_fails_its_check_is_skipped_and_the_next_goes_on`, `a_failed_snapshot_acts_on_nothing`,
  `a_file_opened_after_the_first_snapshot_is_seen_after_two_seconds` (the fake clock moved, the fake open set changed).
- Positive: `items_go_in_plan_order_one_at_a_time` (the outcomes arrive in order, each before the next act — the fake
  Trash records the order), `the_trash_method_moves_and_the_permanent_method_deletes`,
  `a_dry_run_reports_what_would_happen_with_the_folder_walk`.

## Conclusions

- Built in `crates/cleaner/src/pipeline.rs` and wired into `DiskCleaner`, with the step's checks and more: a snapshot
  not refreshed before two seconds, a tool's item never run in the sandbox, an item without an identity, a root no
  longer scanned, a root written `~`, a root that cannot be made canonical, the system's protection with an empty
  context. The tests drive the real `DiskCleaner` through the `Cleaner` port on a sandboxed fixture tree with
  `FakeTrash`, `FakeOpenFiles`, `FakeClock` and `FakeCommandRunner` (`test_support.rs`'s `Rig`).
- **What the engine (06/01–06/03) leaves to the cleaner**, each with a negative test:
  - **The system's protection always applies**: `DiskCleaner::new`'s list is checked with the context's (which is
    `Protection::default()` until the ignore list exists), never instead of it
    (`the_systems_protection_applies_when_the_context_has_none`, and in `paths.rs`).
  - **Roots arrive as the settings write them**: `~` and `~/…` are read under the home folder before any check, so
    **`DiskCleaner::new` takes the home**: `new(reach, user, home, protection, ports)` (the step had no home; plan
    06/04's composition passes it). A root that is still not absolute and plain (a relative path would be read
    against the working folder) or cannot be made canonical is no root: every item under it is skipped
    (`a_root_that_cannot_be_made_canonical_acts_on_nothing`, `a_relative_or_dotted_root_is_never_used`).
  - **No snapshot of open files, nothing acted on**: the engine has no `OpenFiles` port, so the cleaner is the only
    line: a snapshot that fails at the start or at a refresh skips that item and every later one (`CheckFailed`); a
    rehearsal without one skips every item (`a_failed_snapshot_acts_on_nothing`).
- Taken the safest simple way where the step is silent:
  - A clock that went back counts as two seconds passed (the snapshot is taken again).
  - The answer is `Cancelled` when the token is set at the end (an item not reached, or an emptying cut short), else
    `Completed`.
  - The removal follows the action and the plan's method: `Remove` goes to the Trash or, with the plan's
    `PermanentConsent`, to the deletion point; `Empty` empties either way; `Tool` runs its tool after its own checks.
  - Plan 03's `MacOpenFiles` leaves Fresher's own process out of every snapshot; the pipeline also holds no handle
    while a snapshot is taken (each `Verified` is consumed by its removal before the next item).
- The disk-image helper of the tests tries `hdiutil` again a few times: images made by tests running at once
  sometimes found it busy.
