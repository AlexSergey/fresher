# 002/03/03 — The app's footprint

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** store

## Files

- `crates/store/src/footprint.rs` — `DataFolder::footprint() -> Footprint { total: ByteSize, budget: ByteSize, files:
  u32 }` (allocated sizes; links are counted as links, never followed); `FOOTPRINT_BUDGET` = 50 MB; `Footprint::over()`.
  Nothing is deleted: enforcing the budget belongs to the files that fill it (plan 004's cache), clearing to plan 005.

## Produces

The number plan 005 shows as "Mac Fresher's own data: N MB".

## Checks

- Negative: `does_not_follow_a_symlink_out_of_the_folder`, `an_empty_folder_is_zero`.
- Positive: `sums_every_file_in_the_folder`, `reports_over_budget`.
- `cargo llvm-cov nextest -p fresher-store` ≥ 90 % of lines; plan 03's conclusions written.

## Notes from the implementation

- `DataFolder::footprint()` returns `Result<Footprint, DataFolderError>`: reading the folder can fail
  (`DataFolderError::Io`). Subfolders are walked; every entry that is not a folder (a file or a link) is counted, by
  its allocated size (`st_blocks × 512`, from `lstat`), so a link costs only itself.
- `FOOTPRINT_BUDGET` is 50 MB in decimal units (50 000 000 bytes), as macOS shows sizes. `Footprint::over()` is
  `total > budget`: exactly the budget is not over. The fields are public; `reports_over_budget` builds a `Footprint`
  instead of writing 50 MB.
- A test that needs an unreadable folder takes its permissions away and gives them back: the store's tests delete
  nothing (the deletion lint holds for test code too).
