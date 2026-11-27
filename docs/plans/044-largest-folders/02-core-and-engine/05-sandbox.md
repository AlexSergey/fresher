# 044/02/05 — The sandbox's cases

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** sandbox, app/tauri (sandbox mode and
its tests), fixtures

A case file can say a folder is listed on Folders (as a project or not) or not listed; `withheld` keeps describing a
Browse row. The sandbox states each file's ctime, which the disk cannot be told. The design:
[`../00-design.md`](../00-design.md), section 10; agent's calls 15 and 18.

## Read first

02/04's commit; `crates/sandbox/src/{largest.rs,case.rs,error.rs,expected.rs,generate.rs,stated_space.rs,lib.rs}`
(`Shown`, `RawLargest`, `LargestExpectation::from_raw`, the `EXPECTED.md` lines, how ages are set, how a port is
wrapped); `crates/core/src/folder.rs` (`FolderReader`, `RawEntry.changed`); `app/tauri/src/sandbox.rs` (where
`StatedSpace` is plugged in); `app/tauri/tests/sandboxes/{harness.rs,largest.rs}`;
`fixtures/sandboxes/{large-files,stale-projects}.toml` and their `EXPECTED.md`; [`../../../testing.md`](../../../testing.md),
"Sandboxes".

## Files

- `crates/sandbox/src/largest.rs` — `Shown::{Offered, Absent, Withheld, NotSeen, Listed, Unlisted}`, `Shown::ALL`
  with `"listed"` and `"unlisted"`; `Withheld`'s doc becomes "the Browse row at `at` carries the case's reason";
  `RawLargest` gains `project: Option<bool>` (only with `listed`, else `CaseError::LargestProject`); `listed` and
  `unlisted` need an `at` that is a folder of the case.
- `crates/sandbox/src/error.rs` — `LargestProject` added.
- `crates/sandbox/src/case.rs` — `[[case.file]]` and `[[case.dir]]` gain `changed: Option<String>` (an age, as
  `age`; only with `age`, and not older than it).
- `crates/sandbox/src/stated_times.rs` (new, exported) — `StatedTimes`: a `FolderReader` around the real one, built
  from the world's aged paths; for an entry of an aged path `changed` is the case's `changed` or else its `age`'s time,
  at the world's clock; every other entry, and every error, passes through. Its doc says why: macOS moves a ctime on
  every write and cannot set it back.
- `crates/sandbox/src/expected.rs` — the `EXPECTED.md` lines: "Largest, Folders: `~/<at>` is listed (a project)." /
  "... is listed." / "Largest, Folders: `~/<at>` is not listed."; the `withheld` line names Browse.
- `app/tauri/src/sandbox.rs` — sandbox mode reads through `StatedTimes` (beside `StatedSpace`); the harness
  (`app/tauri/tests/sandboxes/harness.rs`) does the same.
- `app/tauri/tests/sandboxes/largest.rs` — `Withheld` checked through `largest_place` as today; `Listed` and
  `Unlisted` through `engine.largest_folders(&FoldersQuery { offset, limit: 100 })`, every page read: `Listed` finds
  the row at `at` (and its `project`), `Unlisted` finds none.
- `fixtures/sandboxes/largest-folders.toml` (new) — the header says what it holds, that every folder and file of a
  case is aged (a folder's own time does not count, a file's does; a file's ctime is stated by the sandbox) and that
  it writes about 2.6 GB of filled data; `feature = "largest-folders"`, `module = "trial"`, `[settings]
  "core.largest.folder-min-size" = "500-mb"` and `"core.ignore.paths" = ["~/Old-tools"]`; the nine cases of the
  design's section 10, negatives first (with "an ignored folder is never listed, even holding explained items" and "a
  folder of old files copied today is not listed"); its `EXPECTED.md` generated.
- `fixtures/sandboxes/large-files.toml` — unchanged (its Public case is a Browse row withheld as a standard folder).

## Interfaces produced

- `npm run dev:sandbox -- largest-folders` for 04/02's cases and 04/03's walk; `fresher_sandbox::StatedTimes`.

## Tests (negatives first)

`crates/sandbox/src/largest.rs`:
- `mod negative`: `an_unknown_shown_is_refused` (kept), `a_project_without_listed_is_refused`,
  `listed_at_a_file_is_refused`.
- `mod positive`: `listed_and_unlisted_are_read`, `a_listed_project_is_read`, `withheld_still_names_a_reason`.

`crates/sandbox/src/case.rs`: negative `changed_without_an_age_is_refused`, `changed_older_than_the_age_is_refused`;
positive `changed_is_read_as_an_age`.

`crates/sandbox/src/stated_times.rs`:
- `mod negative`: `an_entry_the_world_did_not_age_passes_through`, `an_error_passes_through`.
- `mod positive`: `an_aged_entry_is_changed_at_its_age`, `a_stated_changed_wins_over_the_age`.

The harness's autotest runs both case files (`npm run sandbox:test`); every other case file's `EXPECTED.md` and
autotest stay as they are (the stated ctime is what their ages meant).

## Checks

- `cargo nextest run -p fresher-sandbox`, `npm run sandbox:test`
- `npm run sandbox -- largest-folders` and `npm run sandbox -- all` (the generated `EXPECTED.md` match, none of the
  others changed)
- `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`

## Docs

`docs/testing.md`, "Sandboxes": the `[case.largest]` values (`listed`, `unlisted`, `project`; `withheld` now a Browse
row), the `changed` key, and that sandbox mode states ctimes.

Commit: `test(sandbox): cases for the folders left alone`.
