# 008/04/04 — The folder walk

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** cleaner

Check 11, run only with `Depth::Full` (right before an act, and in the dry run).

## Files

- `crates/cleaner/src/guard/walk.rs` (new) — `measure(handle, root_device, cancel) -> Result<Measured, SkipReason>`:
  a depth-first walk with `std::fs::read_dir` and `symlink_metadata` (no link followed; a link counts as itself), a
  hard link counted once (by device and file id when its link count is over one), allocated bytes from `st_blocks`;
  an entry on another device stops it with `OtherVolume`; the cancel token is read between folders (a cancelled walk
  is `NotReached` for the pipeline, not a skip); an unreadable folder inside stops it with `CheckFailed` (what cannot be
  measured cannot be confirmed).
- `crates/cleaner/src/guard/mod.rs` — check 11 in `verify`: the walk's allocated size against the plan item's
  `sizes.allocated`: `Grown` when it is more than 25 % **and** more than 64 MiB over (the constants named
  `GROWTH_SHARE`, `GROWTH_FLOOR`); smaller is fine.

## Produces

The check that stops a permanent delete from crossing into another volume, and a folder that changed in size from
being removed as if it were the one confirmed.

## Checks

- Negative (first): `a_folder_that_grew_past_both_margins_is_grown`, `a_volume_mounted_inside_is_another_volume`
  (a disk image mounted inside the fixture, skipped without `hdiutil`), `an_unreadable_folder_inside_fails_the_check`,
  `a_cancelled_walk_is_not_reached`.
- Positive: `a_folder_that_grew_within_the_margin_passes`, `a_folder_that_shrank_passes`,
  `a_hard_link_inside_counts_once`, `a_link_inside_is_counted_as_itself_not_its_target`,
  `quick_depth_does_not_walk` (a counting test double of the walk sees no call).

## Conclusions

- Built in `crates/cleaner/src/guard/walk.rs` (`SizeWalk`, the app's `StdWalk`) and check 11 in `guard/mod.rs`
  (`GROWTH_SHARE` = 25 %, `GROWTH_FLOOR` = 64 MiB, both margins needed, computed without overflow), with the step's
  checks and a few more: a walk that meets another volume skips the item, a cancelled walk of the real disk, the margins
  at their edges, a link item is not walked, a file is measured without a walk. The volume case mounts a disk image
  inside the folder and checks the image's file is untouched; the growth cases use a walk double that answers a size,
  so no test writes gigabytes.
- **`verify(item, cx, depth)` answers `Unverified`**: `Skipped(SkipReason)` or `Cancelled` (a walk stopped by the
  cancel token), so the pipeline reports a cancelled walk `NotReached` without guessing from the token.
- **The walk is a port of the guard** (`SizeWalk` in the `GuardContext`), so `quick_depth_does_not_walk` counts its
  calls; the cleaner always uses `StdWalk`.
- Taken the safest simple way where the step is silent:
  - **Check 11 measures files too** (their allocated size from the handle, no walk): a file that grew past both
    margins is `Grown` as a folder is. A link is never measured: it is removed as the link.
  - The measure counts the item itself and every entry (folders included) by `st_blocks`; a hard link is counted once
    whatever folder its names are in; an entry that vanishes or cannot be read during the walk fails it
    (`CheckFailed`).
