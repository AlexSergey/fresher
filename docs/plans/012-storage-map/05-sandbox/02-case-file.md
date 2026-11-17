# 012/05/02 — The storage map's case file

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
fixtures, docs

## Files

- `fixtures/sandboxes/storage-map.toml` (new) — `feature = "storage-map"`, `module = "trial"`. Negative cases first:
  - "a folder no module claims is your folder, never explained": `home/Movies/holiday.mov`, 40 MiB, `[case.map]
    part = "your-folders"`;
  - "a look-alike of a trial cache is your folder": `home/work/fresher-trial-cache2/look-alike.bin`, `your-folders`
    at `home/work/fresher-trial-cache2`;
  - "a hard-linked file is counted once": a 16 MiB file under `home/Documents/a/` and its hard link under
    `home/Documents/b/`; `your-folders` at `home/Documents` (the golden report holds `home/Documents` at 16 MiB, not
    32);
  - "a folder that is almost one child opens through it": `home/Library/Containers/com.example.big/Data/blob.bin`,
    200 MiB, beside a 1 MiB file in `home/Library`; `your-folders` at `home/Library/Containers/com.example.big/Data`
    (the golden report holds the descent's path from `home/Library`);
  - "the Trash is your folder, never explained": a 4 MiB file in `home/.Trash`, `your-folders` at `home/.Trash`.
  Then the positive cases:
  - "a trial cache next to a marker is explained": a marker in `home/work`, `home/work/fresher-trial-cache/blob.bin`,
    8 MiB, `fill = "data"`; `expect` `trial.caches`, Safe, `in-trash`; `[case.map] part = "explained"`;
  - "a trial cache inside a trial cache counts once": a marker in `home/nest`, `home/nest/fresher-trial-cache/` with a
    nested `fresher-trial-cache/` inside it (each 4 MiB); `expect` the outer, `in-trash`; `explained` at
    `home/nest` (the golden report holds 8 MiB explained, not 12).
  What the code does today for a nested trial folder and for the Trash's contents (offered or not) is written down
  when the file is made, as 007 did for its guard cases, and the file follows it.
- `fixtures/golden/sandbox-storage-map.json` (new) — written with `UPDATE_GOLDEN=1`, reviewed: the findings, the map
  (parts adding up to the fake capacity), the place rows, the cleanup's outcome, the tree after.
- `app/tauri/tests/sandboxes.rs` — `storage-map` in the macro's list (the guard test `every_case_file_has_a_test`
  fails without it).
- `docs/testing.md` — "Sandboxes": `[case.map]` and the storage map's case file.

**Built:** the paths follow the sandbox, not this list. The scan keeps out of the places that prompt, so the unclaimed
file is `home/footage/holiday.mov` (not `Movies`), the hard link is in `home/linked-twice/{a,b}` (not `Documents`) and
the descent is `home/Library/Developer/com.example.big/Data` (not `Containers`), 48 MiB beside 1 MiB (a filled file is
capped at 64 MiB). `home/work` belongs to `trial.toml`, so the look-alike and the explained cache share
`home/project`. Every file is filled with data so it takes blocks. The descent case's `at` is `home/Library`. The
nested case is checked at its outer cache, not `home/nest`: the marker in `home/nest` takes a block of the user's own.
The explained cache is 12 MiB, not 8. Two items of one size are cleaned in the order of their finding ids, which
differ from run to run, so the harness sorts the report's lists by path and drops the Trash entries' order prefix. What the code does today: the inner trial cache is not a finding of its own (the scan
does not go into a claimed folder), so 8 MiB is explained for the nest, not 12; a file in the Trash with no marker
next to it is not offered and is the user's own.

## Produces

The storage map's sandbox: its autotest and its run by hand (06/02).

## Checks

- `npm run sandbox:test -- storage_map` passes; it fails when one `[case.map]` part is flipped on purpose (done once by
  hand, written in the conclusions).
- `npm run sandbox -- storage-map` writes `sandbox/storage-map/`; its `EXPECTED.md` reads well.
- `npm run lint` (Prettier on the TOML's neighbours and the doc).

## Conclusions

Built in `ac2d005`, fixed in `f331ae8`: `fixtures/sandboxes/storage-map.toml` (five negative cases, then two positive
ones, each with a `[case.map]`) and its golden `fixtures/golden/sandbox-storage-map.json`, wired into `sandboxes.rs`.

- **Deviations**: the cases moved out of the places that prompt (`Movies`, `Documents`, `Containers` became `footage`,
  `linked-twice`, `Library/Developer`), because the scan never enters them. The descent file is 48 MiB (under the fill
  cap). The nest is checked at the outer cache, since the marker in `home/nest` is the user's; the inner trial cache is
  not a finding of its own.
- The harness sorts the report's lists by path and drops the fake Trash's `<n>-` prefix, so the golden no longer depends
  on the findings' ids.
- Tests: `npm run sandbox:test` 21/21; stable 20/20 runs.
