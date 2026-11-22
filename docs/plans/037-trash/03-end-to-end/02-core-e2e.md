# 037/03/02 — The core e2e

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** e2e, testing

The Trash view end to end on files: the real scanner, the real `DiskCleaner` reaching only inside a sandbox folder,
a fake Trash at the flow home's `home/.Trash` (the home's known Trash folder), the real journal, a fake clock. What the
sandbox autotest cannot lay out lives here: an entry emptied from the Trash, a put back, a permanent cleanup, a second
Trash, a rescan, Finder's setting. The design: [`../00-design.md`](../00-design.md), section 7.

Read first: `docs/testing.md` "Kinds of tests" (the core e2e), `crates/e2e/tests/app_caches.rs` and its home folder
(a recent flow test on a flow home), `crates/e2e/tests/support/home_engine.rs` (`Parts`, `engine`: the fake Trash at
`home/.Trash`, `FakeSystemUi`, `FakeCommandRunner`), `crates/e2e/tests/support/home_cleanup.rs` (cleaning and put back
helpers), `crates/e2e/tests/cleanup.rs` (put back, permanent), `fixtures/homes/` (the flow homes' format).

## Files

- `crates/e2e/tests/support/home_engine.rs` — `pub(crate) struct Extras { pub(crate) system_ui: Arc<FakeSystemUi>,
  pub(crate) finder: AutoEmpty }` with `Default` (a fresh `FakeSystemUi`, `AutoEmpty::Unknown`) and `pub(crate) fn
  engine_with(sandbox, stated, open_files, given, extras) -> Engine`; `engine` calls it with `Extras::default()`, so
  no other home changes. The cleanup ports get `FakeFinderSettings::new(extras.finder)`.
- `fixtures/homes/trash.toml` (new) — a flow home on the `trial` module: two trial caches next to markers
  (`home/work-a/fresher-trial-cache`, 64 KiB of data; `home/work-b/fresher-trial-cache`, 32 KiB), a file the user put
  in the Trash (`home/.Trash/old-notes.txt`, 16 KiB of data), and a plain file (`home/plain/keep.txt`).
- `crates/e2e/tests/trash.rs` (new) with its home module (`crates/e2e/tests/trash_home/mod.rs`, as `app_caches_home`
  is) — the module doc says what the file proves and that nothing in it empties a Trash. `mod negative` first:
  - `no_line_before_any_scan_or_cleanup` — a fresh engine answers no line.
  - `the_users_own_file_counts_only_in_the_first_number` — after a scan: one line, `waiting` the file's blocks,
    `from_cleanups` zero.
  - `an_entry_emptied_from_the_trash_counts_nothing` — clean cache A to the Trash, remove its entry from
    `home/.Trash` with `std::fs` (the test standing in for the user's Finder), ask again: the share is zero.
  - `an_item_put_back_counts_nothing` — clean, put back through `Engine::put_back`, ask again: the share is zero.
  - `a_permanent_cleanup_adds_nothing_from_cleanups` — confirm with the permanent method (as `cleanup.rs` does): the
    share stays zero.
  - `open_trash_before_any_answer_is_refused` — `UnknownTrash`, the fake system UI recorded nothing.
  Then `mod positive`:
  - `a_cleanup_adds_its_items_to_both_numbers` — clean both caches: the share equals the cleanup detail's in-Trash
    items' allocated sizes; `waiting` is the user's file plus that share.
  - `a_rescan_does_not_count_the_moved_items_twice` — clean, advance the clock, rescan: `waiting` is the scan's
    measure (the user's file plus the moved items), the share unchanged.
  - `an_item_trashed_outside_the_home_trash_gets_a_line_of_its_own` — a journal entry whose landed path is in a
    second Trash folder outside the scan roots (a fake Trash rooted elsewhere in the temporary folder for one
    cleanup, or a `FakeTrash` built on another folder through `Extras`; pick the simpler and say which in the
    conclusions): a second line, no `waiting`, its share; the home line first.
  - `open_trash_opens_the_home_trash` — after `trash()`, `open_trash(0)` records `OpenFolder(<home>/.Trash)`.
  - `finders_setting_on_dates_the_share` — `Extras { finder: AutoEmpty::After30Days, .. }`: `gone_by` is the
    cleanup's end plus 30 days on the fake clock.
  Every test checks at its end that the outside of the fixture is untouched, as the other flow tests do.
- `crates/e2e/Cargo.toml` — a `[[test]]` entry only if the crate lists its tests (follow `app_caches`).

## Checks

- `cargo nextest run -p fresher-e2e --test trash`; the other flow tests stay green
  (`cargo nextest run -p fresher-e2e`).
- clippy (`--all-targets`), fmt.

## Conclusions

- Built as written: 11 flows (6 negative, 5 positive) on `fixtures/homes/trash.toml`, all green; the other flow tests
  stay green (171 run in `fresher-e2e`).
- The second Trash: a `FakeTrash` built on another folder through `Extras` (`other_trash`: the fake Trash moves there
  instead of `home/.Trash`, and the composition names the folder in `CleanupPorts::trashes`) — simpler than a second
  engine for one cleanup. The folder is a temporary folder outside the sandbox; the flow also opens it (`open_trash(1)`).
- The emptied entry leaves `home/.Trash` by `std::fs::rename` into a temporary folder outside the sandbox, not by a
  delete: the workspace's clippy forbids any delete outside the cleaner's deletion point, and to the engine an entry
  gone from its landed path is the same either way.
- `waiting` of the user's file is exact: 16 KiB logical, the file's blocks allocated. The rescan flow advances the
  fake clock 60 s before rescanning (whole-second timestamps). "The outside untouched" is checked as the plain file
  and the user's file in the Trash, byte for byte, at the end of every flow.
- By hand, each reverted: counting every cleanup as "since the scan" fails the rescan flow; leaving the second Trash
  out of `CleanupPorts::trashes` fails the second-Trash flow.
