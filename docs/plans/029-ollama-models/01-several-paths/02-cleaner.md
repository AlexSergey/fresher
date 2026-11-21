# 029/01/02 — The cleaner

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** cleaner, testing, docs

The cleaner acts on a together item: every part through the whole guard before the first one moves, then the parts in
order, the subject first. The design: [`../00-design.md`](../00-design.md), section 2;
[`../../../architecture/removal.md`](../../../architecture/removal.md), "Adding a kind of removal", whose five rules
this step follows.

## Files

- `crates/cleaner/src/pipeline.rs` — `rehearse`: `Action::RemoveTogether` verifies each part in order against the
  guard (the same `verify` a single path takes, given the part and, at the act, its confirmed identity); the first
  refusal is the item's `Skipped(reason)`; else `PassedTogether(identities)`. `act_on`: verifies every part with
  `Depth::Full`, in order, keeping each `Verified`; the first refusal ends the item `Skipped` with nothing moved (every
  handle dropped); a dry run ends it `DryRun` after every check; else calls `remove::together`.
- `crates/cleaner/src/remove/together.rs` (new) — `pub(super) fn together(verified: Vec<Verified>, act: Act<'_>, way:
  Way<'_>, cancel: &CancelToken) -> Removed`: each part in order through `to_trash` or `delete` (the single deletion
  point) with its own `Verified`; the cancel token read between parts; every part gone: `Moved` or `Deleted` with every
  `TrashedItem`; a part skipped, failed or cancelled after an earlier one went: `Failed { reason: Failure::Partly {
  done, of }.text() }` with the parts that went in `trashed`, and nothing more is tried; the first part failing is
  that part's own outcome. It takes `Verified` by value and an `Act`, never deletes itself, never falls back from the
  Trash to a delete. `remove/mod.rs` — the module, private to the crate.
- `crates/cleaner/tests/never.rs` — the generator's actions gain `RemoveTogether` of two to five real or made-up paths
  (some inside protected, outside or linked folders); the model: a together item is removed whole when every part
  passes, else not at all, unless the Trash fake refuses a part at its act; the existing property (only plan parts or
  what is inside them leave; protected and outside trees list as before) holds for every part.
- `crates/cleaner/src/test_support.rs` — a rig helper for together items; the Trash fake refuses the n-th call on
  demand.
- `docs/architecture/removal.md` — "Kinds of removal" gains **Together** (`remove/together.rs`); the flow's rehearsal
  and act name it; the threat model gains the row "an item of several paths half done" (design section 2).

## Produces

The cleaner's together items, for 03 (what `trashed` holds) and the module's e2e.

## Checks

`remove/together.rs` and `pipeline.rs`, `mod negative` first, each asserting that the tree lists exactly as before and
that `trashed` is empty:

- `a_part_outside_the_roots_moves_no_part`; `a_protected_part_moves_no_part`; `a_part_reached_through_a_link_moves_no_part`
  (the layers' folder swapped for a link after the rehearsal); `a_replaced_part_moves_no_part` (same path, another
  file id); `a_vanished_part_moves_no_part`; `a_grown_part_moves_no_part`; `a_part_held_open_moves_no_part`;
  `a_mapped_part_moves_no_part` (the open-files fake names it as a mapped file); `a_part_a_running_app_owns_moves_no_part`;
  `a_dry_run_moves_no_part_after_every_check`;
- `the_trash_refusing_the_second_part_ends_partly_with_the_subject_journalled` (the subject in the Trash and in
  `trashed`, the rest in place); `a_cancel_between_parts_ends_partly_and_tries_no_more`;
  `a_failing_first_part_is_that_parts_own_failure`;
- positive: `every_part_goes_to_the_trash_the_subject_first`; `the_permanent_way_deletes_each_part_through_the_deletion_point`;
  `a_rehearsal_answers_every_parts_identity`; `a_together_item_beside_single_items_keeps_the_plan_order`.
- `tests/deletion_point.rs` still finds one deletion point; `tests/types.rs` compile-fail cases unchanged.
- `cargo nextest run -p fresher-cleaner` (and `PROPTEST_CASES=10000` for `never` once, by hand, written in the
  conclusions); clippy; fmt.

## Conclusions

Built as written. The guard's `verify` is split into a per-part check (checks 1 to 10, every part held open) and a
growth check (11) run once on the sum of the parts' measured sizes against the item's confirmed size, before the first
part moves (preflight F3: a part that grew while another shrank by the same amount passes; every other check is per
part). `verify_together` refuses a list of identities that is not one per part, a rehearsal answers
`PassedTogether`, a dry run ends `DryRun` after every check, and `remove/together.rs` moves the subject first, then the
others, each through the single Trash or deletion point, never falling back from the Trash to a delete. What changed
from the design: the cancel token is read before every part, the first included (call 2 of the design), so a cancel
after the subject moved ends "only part of it went" and is journalled; a part that failed with its entry landed
elsewhere counts as having gone (final wave: `Partly.done` equals the journal's `went`). The permanent way's partial
result has its own text: `Failure::Partly { done, of, way: PartlyWay::Trash | Deleted }`, two keys, the deleted one
saying the files that went cannot be put back (the final review found the Trash text on a permanent delete). The
together tests live in `pipeline/tests/together.rs` (a pure move out of a 1,010-line file). `tests/never.rs` now
generates together items of one to four more parts, protected, outside and linked ones among them, and holds. Review:
approved.
