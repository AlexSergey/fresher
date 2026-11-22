# 032/04/02 — Other apps' data, written bytes, insight expectations

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, testing, app

Read first: [`../00-design.md`](../00-design.md) section 8, `crates/sandbox/src/case.rs` (`RawAccess`,
`guarded_folder`, the test that refuses `other-apps-data` today), `crates/sandbox/src/world.rs` (`access`,
`stated_places`, `declares_access`), `crates/sandbox/src/expected.rs` (the checklist), `crates/testing/src/
tree_description.rs` (`Fill::Sparse`, `MAX_FILLED_FILE`), `app/tauri/src/sandbox.rs` (`compose`: the standing avoid
list and `stated_places`), `app/tauri/tests/sandboxes/harness.rs` (the switches turned on, `report_of`, the
left-out checks), `app/tauri/tests/sandboxes/left_out.rs` (the shape of an expectation's checks).

## Files

- **Other apps' data in the world's access.**
  - `crates/sandbox/src/case.rs` — `access` accepts `other-apps-data` beside `desktop`, `documents`, `downloads`
    (`granted`, `denied`, `not-asked`); the case's access becomes `Vec<(StatedCapability, Access)>` with `enum
    StatedCapability { Folder(GuardedFolder), OtherAppsData }` (or the core's `Capability` limited to those two kinds,
    whichever keeps the diff smaller). The test that refused `other-apps-data` now expects it read; `all-files` and
    unknown keys stay refused.
  - `crates/sandbox/src/world.rs` — `access(home)` answers `Capability::OtherAppsData` as stated (else `NotAsked`) and
    guards `home/Library/Containers` and `home/Library/Group Containers` with it; trying either answers as a stated
    folder does; `stated_places(home)` lists both when other apps' data is stated; `world.toml` writes
    `other-apps-data = "…"` under `[access]` (the most restrictive statement wins across cases, as for folders).
  - `app/tauri/tests/sandboxes/harness.rs` — the run also turns on `core.access.other-apps-data` when the world
    states it.
- **Written bytes at the head of a sparse file.**
  - `crates/testing/src/tree_description.rs` — a `[[file]]` may state `written = <bytes>` with the sparse fill (the
    default): its first `written` bytes are the data pattern, the rest stays a hole, so its logical size is `size` and
    its allocated size about `written`. Refused: `written` with another fill, `written > size`, `written` over
    `MAX_FILLED_FILE`, and counted in the tree's filled total (`MAX_FILLED_TREE`). Case files (`[[case.file]]`) take it
    through the same vocabulary.
- **Insight expectations.**
  - `crates/sandbox/src/case.rs` — a case may state `[[case.insight]]`: `identity` (kebab case), `figures`, `parts`,
    `commands` (whole numbers: how many figures and parts the insight has, and how many parts carry a command). Refused:
    an identity twice in one file, `commands > parts`, an unknown field. Allowed on a negative case (nothing of it is
    offered) and on a positive one.
  - `app/tauri/tests/sandboxes/harness.rs` (and a new `app/tauri/tests/sandboxes/insight.rs`, in the shape of
    `left_out.rs`) — the scan's insights are read from the `ModuleFinished` events of the case file's module. An
    insight no case expects fails: "insight `<identity>` of `<module>` is described but no case expects it"; an
    expected one not given fails: "case `<name>` expects insight `<identity>`, which `<module>` did not describe"; a
    count that differs fails with both numbers ("figures: expected 2, described 3"). The golden report gains
    `"insights"` (the insights as serialized, in order, paths relative) when there is any; a case file without insights
    keeps its golden byte for byte.
  - `crates/sandbox/src/expected.rs` — `EXPECTED.md` gains, per expected insight, a line in the case's section: "Under
    «Described, not cleaned», the card «<identity>» shows <figures> numbers; its Details lists <parts> parts, <commands>
    with a command to copy. Copy puts the command on the clipboard. Nothing of it can be selected." (and for 0 parts:
    "its Details lists no part").

## Produces

`other-apps-data` in the world's access, `written`, `[[case.insight]]` and its checks, the golden's `insights`, for
04/03.

## Tests (negative first)

- `case.rs` — negative: `all_files_stays_refused_in_access`, `an_insight_identity_twice_is_refused`,
  `more_commands_than_parts_is_refused`; positive: `other_apps_data_is_read_in_access`, `an_insight_expectation_reads`.
- `world.rs` — negative: `other_apps_data_not_stated_is_not_asked`; positive:
  `other_apps_data_guards_containers_and_group_containers`, `other_apps_data_round_trips_through_world_toml`,
  `the_most_restrictive_other_apps_data_statement_wins`.
- `tree_description.rs` — negative: `written_with_the_data_fill_is_refused`, `written_over_the_size_is_refused`,
  `written_over_the_fill_cap_is_refused`; positive: `a_written_head_takes_about_its_bytes_on_disk`.
- `app/tauri/tests/sandboxes/insight.rs`: no registered module describes before 04/03, so its check functions are
  tested on hand-built insights and expectations: negative `an_insight_no_case_expects_fails_naming_it`,
  `an_expected_insight_not_described_fails`, `an_insight_count_that_differs_fails_with_both_numbers`; positive
  `an_expected_insight_passes_and_is_reported`. The end-to-end proof is 04/03's case files and mutation checks.
- `expected.rs` — positive: `an_expected_insight_writes_its_line` (with parts and with none).

## Done when

- `cargo nextest run -p fresher-sandbox -p fresher-testing -p fresher-app`; `npm run sandbox:test` (every existing case
  file passes, goldens unchanged); clippy; fmt.
- One commit: `feat(sandbox): other apps' data, written bytes and insight expectations`.

## Conclusions

Not written yet.
