# 044/02/02 — The size setting and the stale days

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** core, engine, modules

`core.largest.folder-min-size`, and the stale projects' days readable by the engine. The design:
[`../00-design.md`](../00-design.md), section 6; agent's call 11.

## Read first

`crates/core/src/settings/core_schema.rs` (`LARGEST_MIN_SIZE`, `MIN_SIZES`, `min_size_of`, `core_settings`, its
tests), `crates/core/src/settings/days.rs` (`DaySetting`), `crates/core/src/settings.rs` (the exports),
`crates/core/locales/en.json` (`core.largest.min-size.*`), `crates/engine/src/settings.rs` (`largest_min_size`,
`module`), `modules/stale-projects/src/lib.rs` (`UNUSED_FOR`, its uses at the descriptor and in `finish`); the
settings screen's test of the Scan group (`app/ui/src/screens/settings/`).

## Files

- `crates/core/src/settings/core_schema.rs` — `pub const LARGEST_FOLDER_MIN_SIZE: &str =
  "core.largest.folder-min-size";` (doc: "The smallest folder the Largest screen's Folders lists: a choice of
  `500-mb`, `1-gb`, `5-gb` or `10-gb`."), `FOLDER_MIN_SIZE_DEFAULT = "1-gb"`, `FOLDER_MIN_SIZES: [(&str, u64); 4]`
  (500 000 000, 1 000 000 000, 5 000 000 000, 10 000 000 000), `pub fn folder_min_size_of(choice: &str) -> ByteSize`
  (an unknown choice is the default's); the descriptor `folder_min_size` (Choice, group Scan, effect Now) right after
  `min_size` in `core_settings`.
- `crates/core/src/settings/days.rs` — `pub const STALE_PROJECTS_MODULE: &str = "stale-projects";` and `pub const
  STALE_PROJECTS_UNUSED_FOR: DaySetting = DaySetting::new("unused-for-days", 30, 3650, 365);` with a doc: declared by
  the stale projects module, read by the engine for Largest's Folders.
- `crates/core/src/settings.rs`, `crates/core/src/lib.rs` — export them and `folder_min_size_of`,
  `LARGEST_FOLDER_MIN_SIZE`.
- `crates/core/locales/en.json` — the six texts of the design's section 6.
- `crates/engine/src/settings.rs` — `largest_folder_min_size(&self) -> ByteSize` (as `largest_min_size`) and
  `stale_days(&self) -> u32` (`ModuleId::new(STALE_PROJECTS_MODULE)` then `STALE_PROJECTS_UNUSED_FOR.days(&self
  .module(id))`; the default's days when the id does not parse).
- `modules/stale-projects/src/lib.rs` — `UNUSED_FOR` is `fresher_core::STALE_PROJECTS_UNUSED_FOR`; the module id
  from `STALE_PROJECTS_MODULE`. Its behaviour and its tests do not change.

## Interfaces produced

- `Settings::{largest_folder_min_size, stale_days}` (engine, `pub(crate)`); the setting in `get_settings`.

## Tests (negatives first)

`crates/core/src/settings/core_schema.rs` (or its test module):
- `mod negative`: `an_unknown_folder_size_choice_is_the_default`, `a_stored_folder_size_outside_the_choices_is_a_problem`
  (resolved to the default, with its problem, as `core.largest.min-size`'s test does).
- `mod positive`: `folder_size_choices_are_decimal_bytes`, `the_folder_size_follows_the_smallest_size_in_the_scan_group`.

`crates/engine/src/settings.rs`:
- `mod negative`: `stale_days_out_of_range_take_the_nearest_limit`.
- `mod positive`: `stale_days_default_to_a_year`, `stale_days_follow_the_stale_projects_setting`,
  `the_folder_size_follows_its_setting`.

`npm run check:texts` holds the new keys.

## Checks

- `cargo nextest run -p fresher-core -p fresher-engine -p fresher-module-stale-projects`
- `npx vitest run src/screens/settings` (from `app/ui`): the Scan group lists the new setting with its texts
- `npm run check:texts`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`

## Docs

None yet (04/01 names the setting in "The largest files").

Commit: `feat(core): the smallest folder Largest lists, and the stale days for the engine`.
