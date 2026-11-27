# 044/01/01 — `newest` from the fold and `freeze`

**Status:** ✔️ done (the LARGE timing is pending: taken once after steps 01 and 02, in plan 01's conclusions) · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** scanner

Each folder's newest change: its listing's newest file during the walk, its subtree's at `freeze`; a change is the
later of a file's `modified` and `changed` (ctime), settled here so the tree format changes once (01/02). The design:
[`../00-design.md`](../00-design.md), section 1; agent's calls 1, 2 and 18.

## Read first

`crates/core/src/folder.rs` (`RawEntry.modified`, `RawEntry.changed`: no field is added), `crates/scanner/src/tree/node.rs` (`Node`, `time_bits`, `time_of`, the size test), `tree/fold.rs` (`fold`,
`FolderBatch`), `tree/apply.rs` (`Tree::apply`), `tree/arena.rs` (`freeze`, `sum_bottom_up`, `heap_bytes`) and
`tree/{fold_tests,arena_tests}.rs`; [`../../../testing.md`](../../../testing.md), "Timing bars".

## Files

- `crates/scanner/src/tree/node.rs` — `Node.newest: i64` after `modified`, doc: "The newest change of a file, link
  or special entry inside: its own listing's until the tree is frozen, its subtree's after; `i64::MIN` for none.";
  `Node::new` sets `NO_TIME`; `Node::newest() -> Option<Timestamp>` (`time_of`); `Node::join_newest(&mut self, bits:
  i64)` (the larger). The doc of `Node` says 80 bytes; `a_node_takes_72_bytes` becomes `a_node_takes_80_bytes`.
- `crates/scanner/src/tree/fold.rs` — `FolderBatch.newest: i64` (`NO_TIME` in `new`), doc: the newest change (the
  later of `modified` and `changed`) of the listing's files, links and special entries that were read, `.DS_Store`
  left out; `fn change_of(entry: &RawEntry) -> i64` (`time_bits` of the later, `NO_TIME` when neither is known). `fold` joins every entry with
  no error whose kind is not `Folder` and whose name is not `b".DS_Store"`, kept or folded, before deciding to keep it.
  `const DS_STORE: &[u8] = b".DS_Store";` with a comment saying why (the Finder writes it when the user only looks).
- `crates/scanner/src/tree/apply.rs` — `self.join_newest(folder, batch.newest)?` after `add_own` (a new
  `Tree::join_newest(index, bits) -> Result<(), TreeError>` beside `set_modified` in `arena.rs`).
- `crates/scanner/src/tree/arena.rs` — `sum_bottom_up` joins `node.newest` into the parent after the sums;
  `Tree::newest(index) -> Option<Timestamp>`. The doc of `freeze` names the newest.

## Interfaces produced

- `Node.newest` (`pub(crate)`), `Node::{newest, join_newest}`, `Tree::{newest, join_newest}`, `FolderBatch.newest`:
  01/02 stores and patches them, 02/01 reads them.

## Tests (negatives first)

`crates/scanner/src/tree/fold_tests.rs`:
- `mod negative`: `an_entry_that_could_not_be_read_adds_no_time`, `ds_store_is_not_a_change`,
  `a_subfolders_own_time_is_not_a_change`, `a_listing_of_folders_only_has_no_newest`,
  `a_file_copied_today_with_an_old_modified_is_recent` (the seam is `RawEntry` itself: `entry(..)` with `modified` 400
  days before and `changed` one day before; the batch's newest is the `changed`).
- `mod positive`: `the_newest_small_file_is_the_batchs_newest`, `a_kept_file_and_a_link_count_as_changes`,
  `an_entry_with_only_one_time_counts_that_one`.

`crates/scanner/src/tree/arena_tests.rs`:
- `mod negative`: `an_unfrozen_folder_holds_only_its_own_newest`.
- `mod positive`: `freeze_takes_the_newest_of_every_subtree`, `freeze_twice_keeps_the_newest` (beside
  `freeze_twice_changes_nothing`), and the `freeze_is_idempotent` property with random times.

`crates/scanner/src/tree/node.rs`: `a_node_takes_80_bytes`.

## Checks

- `cargo nextest run -p fresher-scanner`
- `heap_bytes_per_node_stays_under_96` passes unchanged. If it fails, stop and report: the bar is not raised without
  the user (the design's risk; the fallback is a side table of folders).
- The scan time of `LARGE` before and after, cold and warm on `SMALL`, as [`../../../testing.md`](../../../testing.md)
  "Timing bars" runs them; the numbers go in plan 01's conclusions.
- `cargo clippy -p fresher-scanner --all-targets -- -D warnings`, `cargo fmt --all --check`

## Docs

None yet: 04/01 describes `newest` with the format of 01/02.

Commit: `feat(scanner): each folder knows its newest change`.
