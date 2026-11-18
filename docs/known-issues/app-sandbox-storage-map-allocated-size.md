# The sandbox autotests failed at times on an allocated size

**Status:** fixed (`fix(testing)`: a data-filled fixture file is sized before it is written) · **Updated:** 2026-11-18

## Symptom

Under load (the whole workspace's tests running at once, as `npm run pre-push` does) a sandbox autotest of
`app/tauri/tests/sandboxes.rs` failed on an allocated size: `storage_map` (the 48 MiB file of
`fixtures/sandboxes/storage-map.toml` off by 48 to 112 blocks) and `trial` (the storage map's `explained` part of
`fixtures/sandboxes/trial.toml` larger by 208 or 240 blocks, the remainder smaller by as much). Run alone, they passed.
Seen during plan 017 (2026-11-17 and 2026-11-18), on the plan's base commit too.

## Cause

The fixture writer (`fresher-testing`, `TreeDescription::build_at`) grew a `fill = "data"` file by appending 64 KiB
writes, then synced it. A file grown by appends can keep blocks the file system reserved past its end; under load they
were still there after the sync and the close, so the file's size on disk was larger than its length rounded up to whole
4 KiB blocks. Only the largest files (40 MiB and more) were seen with them, by 128 KiB to 960 KiB.

The scan read them right: a probe printed each data file's `st_blocks` right after the generation and after the scan,
and the scan's node held the same larger number. The writer's own probe saw it straight after `sync_all`
(`trial`'s 50 MiB file: 52559872 bytes on disk for 52428800 bytes). In four whole-workspace runs before the fix, every
run failed one sandbox autotest this way.

## What was tried

- `set_len` to the same length after the sync: no effect (the length does not change, nothing is trimmed).
- The fix: `set_len` to the full length before writing, so every write lands inside the file and none extends it. In
  three whole-workspace runs after it, none of the 1140 data files written took a block past its end, and no sandbox
  autotest failed. The fixture writer's test `filled_files_take_exactly_their_blocks` holds the exact size on disk of a
  1 MB and a 50 MiB file.

## Options left

None needed. The test cannot force the reservation without load, so it guards the contract, not the race.
