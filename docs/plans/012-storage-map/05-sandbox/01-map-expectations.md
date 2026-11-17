# 012/05/01 — Map expectations

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
sandbox, app

## Files

- `crates/sandbox/src/case.rs` — `Case` gains `map: Option<MapExpectation>` (`[case.map]`), with
  `MapExpectation { part: MapPartName, at: Option<String> }` and `MapPartName::{Explained, YourFolders}` (serde
  kebab-case, `deny_unknown_fields` as the rest of the format). `at` is the folder the place view must show the case's
  files under (relative to the sandbox; default: the folder of the case's first file). Allowed on a negative case
  (`your-folders`) and on a positive one (`explained`, with its `expect`); the generator ignores it.
- `crates/sandbox/src/error.rs` — `CaseError` gains, each naming its case: `MapOnTheWrongKind` (`explained` on a
  negative, or `your-folders` on a positive), `MapAtOutsideTheCase` (`at` is not one of the case's paths or their
  folders).
- `crates/sandbox/src/expected.rs` — `EXPECTED.md` adds one plain sentence per map expectation, under the case's own
  line: "on the overview, by place: `home/Movies` is in *Your folders*, nothing of it explained"; "…: `home/work/
  fresher-trial-cache` is explained".
- `app/tauri/tests/sandboxes.rs` — after the scan and before the cleanup, for each case with `[case.map]`:
  `storage_place` of `at` (found by `DiskState::find` through the engine's state) — `your-folders`: the row's
  `explained` is zero and no finding is at or below it; `explained`: the row's `explained` equals its size. Then, once
  per case file: the map's parts add up to the fake volume's capacity and both views have one total. The golden
  report gains `map`: the parts of each volume and the place rows of every `at`, sizes as numbers, paths relative to
  the sandbox. The harness's `FakeVolumes` gets a `DiskSpace` for the sandbox (`FakeVolumes::disk("Sandbox", 1 GiB,
  the tree's size + 64 MiB, 256 MiB)`), so the remainder and Free are deterministic.

## Produces

The expectation the storage map's case file (02) uses, open to every later feature that changes the map.

## Checks

- Negative first: `explained_on_a_negative_is_refused`, `your_folders_on_a_positive_is_refused`,
  `a_map_at_outside_the_case_is_refused`, `an_unknown_map_field_is_refused`; harness:
  `a_your_folders_expectation_that_is_explained_fails_with_its_case_name` (an inline case file whose negative sits
  inside a trial cache's claim).
- Positive: `a_case_with_a_map_parses`, `the_checklist_says_where_the_map_shows_it`; the existing case files (`trial`,
  `cleanup-guards`) pass unchanged, their golden reports gaining only `map`.
- `cargo nextest run -p fresher-sandbox`; `npm run sandbox:test`.

**Descent:** the map expectations call `storage_place` with `Descent::Through`, as the overview opens a folder row.

**Plan 05 is not built yet:** it waits for plan [007](../../007-safe-development/readme.md) to be merged into `main`, so the storage map's case file (02) does not exist yet.

## Conclusions

Built in `7cc99cf`, fixed in `79d08a6`: an optional `[case.map]` (`part` = `explained` or `your-folders`, `at`) in the
case format, one sentence per expectation in `EXPECTED.md`, and a map check in the sandbox harness before the cleanup
(the parts add up to the fake disk's capacity, then every expectation).

- **Deviations**: `at` is a `PathBuf` resolved at parse time (default: the folder of the case's first file;
  `MapWithoutFolder` when that is the sandbox root). A claimed folder is a leaf of the scan, so its row is checked in the
  parent's view with `Descent::Stay`. `your-folders` also fails on a finding above `at`.
- The goldens keep allocated sizes; a guard fails with a message unless the volume is APFS-like (4 KiB blocks, sparse
  files cost 0).
- Tests: `npm run sandbox:test` 20/20, stable over repeated runs; `fresher-sandbox` 54/54.
