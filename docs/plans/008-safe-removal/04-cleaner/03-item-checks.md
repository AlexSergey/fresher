# 008/04/03 — The item: identity, removability, capability, use

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** cleaner

Checks 7 to 10, and `verify`, which joins the checks of 01–03 into one call.

## Files

- `crates/cleaner/src/guard/mod.rs` (new) — `pub fn verify(item: &PlanItem, cx: &GuardContext<'_>, depth: Depth) ->
  Result<Verified, SkipReason>`, running checks 1 to 10 in the design's order (11 joins in 04); `Verified` holds the
  handle, the placed path and the identity, is not `Clone` and has no public constructor.
- `crates/cleaner/src/guard/identity.rs` (new) — check 7: the handle's device, file id and kind against the plan's
  `ItemIdentity` (`Replaced`, `KindChanged`); in the rehearsal of a new plan, where there is no identity yet, it reads
  and returns it.
- `crates/cleaner/src/guard/facts.rs` (new) — check 8: `EntryFacts` (plan 006) read again from the handle's metadata
  and its parent's (owner against the given user, write access from the mode bits as plan 006's reference walk does,
  `st_flags`), then `Removability::of`; anything but `Removable` → `NotRemovable { removability }`.
- `crates/cleaner/src/guard/access.rs` (new) — check 9: the guarded place the path lies in (from the context's
  `GuardedPlace`s, by components), its capability, `AccessStatus::check`; a tool's `Tool(id)`.
- `crates/cleaner/src/guard/in_use.rs` (new) — check 10: `OpenSet::holder_under(path)` → `InUse { app }`; the set is
  the one the pipeline keeps fresh (06).

## Produces

`verify`: the one function every removal and every rehearsal goes through.

## Checks

Negative (first), on fixtures and fakes:

- `an_item_swapped_for_another_is_replaced` (removed and made again between the rehearsal and the check: a new file id),
  `a_file_turned_folder_has_changed_kind`, `a_locked_item_is_not_removable`, `an_item_of_another_owner_is_not_removable`
  (the user given as another uid), `an_item_in_a_read_only_folder_is_not_removable`,
  `an_item_in_a_guarded_place_without_its_capability_is_refused`, `a_tool_whose_capability_is_denied_is_refused`,
  `a_held_file_inside_a_folder_makes_it_in_use` (with the holder's name), `a_held_folder_is_in_use`.
- Positive: `an_untouched_item_verifies`, `a_rehearsal_reads_the_identity`,
  `a_guarded_place_with_its_capability_passes`.
- `Verified` cannot be cloned (the compile-fail case is added in 08).

## Conclusions

- Built in `crates/cleaner/src/guard/` (`mod.rs`, `identity.rs`, `facts.rs`, `access.rs`, `in_use.rs`) with the
  step's checks; each check also has its own unit tests, and the verify-level cases run on a sandboxed fixture tree
  (`test_support.rs`'s `Setup`: the tree as the only root, the protections, the access, the guarded places, the open
  set). An item of another owner is tested on `facts.rs` itself: at the level of `verify`, a user given as another
  uid already fails check 1, since the sandbox marker must be that user's.
- **`Verified` is public** (re-exported at the crate root) so the compile-fail cases of 08 can name it; it has
  private fields, no constructor outside the guard, no `Clone`, and only crate-private accessors. `verify` itself stays
  crate-private.
- Taken the safest simple way where the step is silent:
  - **A tool's item has its own checks** (`verify_tool`): check 1 (never in the sandbox), its arguments (no `-f`,
    `--force`, `--force=…`: `ForcedTool`, again below the plan builder), and check 9, where the tool must be listed
    as `Granted` by plan 006's tool check: a tool the status does not list is refused. `verify` refuses a tool's item
    with `CheckFailed`.
  - **Check 1 before check 2**: in the sandbox, an item that cannot be placed in a root is `NotInSandbox`; on the
    real disk it gets check 2's answer.
  - **An act on an item without an identity** acts on nothing (`CheckFailed`); a rehearsal takes the identity found.
    A different kind is `KindChanged` before a different file id is `Replaced`. **An `Empty` of something that is not
    a folder** is `KindChanged`.
  - **Check 8 opens the parent through no link** (a link there is `ThroughLink`); the flags are read from the item's
    handle (`UF_IMMUTABLE`/`UF_APPEND` lock it; `SF_IMMUTABLE`/`SF_APPEND`/`SF_NOUNLINK`/`SF_RESTRICTED` make it the
    system's), the constants spelled in the crate since it depends on the core alone.
  - **Check 9 also asks for the capability of a guarded place inside the item** (it would go with it), and matches
    both forms of the path.
  - **Check 10 looks under both forms of the path**; a process holding a folder above the item does not count.
- `verify` takes `(item, cx)` here; the depth joins in 04 with the walk.
