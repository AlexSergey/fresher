# 008/04/05 — The removals and the deletion point

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** cleaner

## Files

- `crates/cleaner/src/remove/mod.rs` (new) — the removal functions, private to the crate; each takes a `Verified`
  (consumed) and an `&Act<'_>`:
- `crates/cleaner/src/remove/trash.rs` (new) — `to_trash(verified, act, trash) -> ItemDone`: the `Trash` port with the
  verified path and its held handle (design, call 37); the returned `reported` path, `lstat`-ed (no link followed),
  is the landed path when it has the verified device and file id; else `at` (where the held handle's item is now) when
  it lies in a Trash folder; else the outcome is `Failed` with "Another item was moved to the Trash in its place" and
  `reported`, when there is one, is still journalled so it can be put back. `NoTrash` → `Skipped { NoTrash }`; never a
  permanent delete instead.
- `crates/cleaner/src/remove/empty.rs` (new) — `empty(verified, act, method, …)`: the folder's entries are read from
  the verified folder, each one opened beneath it with the no-link open, its identity taken from the listing and
  checked again on the open handle, then moved to the Trash (or deleted) on its own; the cancel token is read between
  entries; the outcome is `Emptied { moved, skipped }` with each entry's landed path for the journal.
- `crates/cleaner/src/remove/permanent.rs` (new) — **the single deletion point**: `delete(verified, act, consent:
  &PermanentConsent)`, a file or a link with `std::fs::remove_file`, a folder with `std::fs::remove_dir_all`, under the
  workspace's only `#[expect(clippy::disallowed_methods, reason = "…")]`; afterwards the path must be gone (`lstat`
  answers not found).
- `crates/cleaner/src/remove/tool.rs` (new) — `run_tool(command, act, runner) -> ItemDone`: `CommandRunner::run` with
  the program and the fixed arguments; exit 0 → `ToolDone`; otherwise `Failed` with the core's text and the first
  line of the standard error (at most 200 characters); the journal gets the exit status only.
- `crates/cleaner/tests/deletion_point.rs` (new) — reads every `.rs` file of the workspace and fails when
  `disallowed_methods` is expected or allowed anywhere but `remove/permanent.rs`, or more than once there.
- `clippy.toml` — the lint's reason names the deletion point (`crates/cleaner/src/remove/permanent.rs`).

## Produces

Every way an item can leave its place; reachable only with a `Verified` and an `Act`.

## Checks

Negative (first), with `FakeTrash` and `FakeCommandRunner`:

- `a_volume_without_a_trash_skips_and_deletes_nothing`,
  `a_landed_item_that_is_not_the_verified_one_fails_and_is_journalled`,
  `an_emptied_folder_skips_an_entry_swapped_for_a_link`, `an_emptied_folder_stops_between_entries_when_cancelled`,
  `a_tool_that_fails_reports_its_first_error_line`, `a_tool_that_is_not_installed_fails`,
  `the_lint_is_expected_only_at_the_deletion_point` (the source test, against a planted offender in a temporary copy).
- Positive: `an_item_goes_to_the_trash_and_its_landed_path_is_returned`,
  `a_move_without_a_reported_path_lands_where_the_held_handle_is`, `an_emptied_folder_stays_and_its_entries_go`,
  `a_permanent_delete_removes_a_folder_without_following_a_link_inside` (a link inside points outside the root: its
  target is untouched), `a_link_item_is_deleted_as_the_link`, `a_tool_that_succeeds_is_done`.

## Conclusions

- Built in `crates/cleaner/src/remove/` (`mod.rs`, `trash.rs`, `empty.rs`, `permanent.rs`, `tool.rs`), with
  `tests/deletion_point.rs` and the lint's reason in `clippy.toml` naming the deletion point; the step's checks and
  more: a refused move, an item gone before the move, a delete the system refuses (a locked file: `Failed`, the item
  in place), an emptied folder whose entries have no Trash, a folder gone before it is emptied, a permanent emptying,
  a tool error cut to 200 characters. The Trash doubles of the tests (a decoy mover, an unlocated mover, one that
  swaps an entry for a link or cancels on its first move) sit in the tests; nothing reaches the real Trash.
- **Each removal returns a `Removed { outcome, trashed, status }`**; the pipeline (06) adds the index, the finding and
  the sizes to make the `ItemDone`. **`Act` is taken by value**: it is `Copy` and one pointer wide, so Clippy's
  pedantic `trivially_copy_pass_by_ref` refuses `&Act<'_>`; it holds the same licence.
- **The landed check, with plan 03's adapter as it was built** (its `at` is an empty path when `F_GETPATH` fails
  after a move): `Moved` to `reported` when it holds the verified identity; else `Moved` to `at` when `at` is a path
  in a Trash folder (a component named `.Trash` or `.Trashes`) that holds it; else, when `at` names a place outside
  the Trash, `Failed` ("Another item was moved…") with `reported` journalled under its own identity; and when `at` is
  empty, the original place decides: still holding the verified item, `Failed` as above; not, the item is `Moved`
  with nothing journalled, since where it went is unknown (never "failed" for an item that left its place).
- Taken the safest simple way where the step is silent:
  - Trash errors: `NoTrash` → `Skipped { NoTrash }`, `NotFound` → `Skipped { Vanished }`, `ThroughLink` →
    `Skipped { ThroughLink }`, anything else → `Failed` ("macOS refused…"); never a delete instead.
  - **An emptied folder's entries** are listed first (sorted by name), then each is opened beneath the folder and
    checked against its listed identity (`guard::verify_entry`: checks 5 to 7, which also refuse another volume),
    and goes through `to_trash` or `delete` with its own `Verified`, so the deletion point stays single. An entry that
    cannot be read, fails its checks or its removal counts as skipped; `moved` counts the entries that went, deleted
    ones included. What went to the Trash, and what a failed move journalled, is returned for the journal.
  - **The deletion point** deletes a file or a link with `remove_file`, a folder with `remove_dir_all`, keeps the
    handle open during the delete, and answers `Deleted` only when the delete returned and `lstat` says not found.
  - **A tool** runs as `<tool id> <arguments…>`; its first non-empty line of standard error is shown (at most 200
    characters); a tool that cannot be started fails as missing (`Failure::ToolMissing`).
  - **The source test is stricter**: besides the lint's name anywhere but once at the deletion point, it refuses any
    `.rs` file that names `clippy::style` or `clippy::all`, the groups that hold the lint.
