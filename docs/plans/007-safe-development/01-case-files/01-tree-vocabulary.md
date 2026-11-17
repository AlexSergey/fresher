# 007/01/01 — The tree vocabulary, reusable

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** testing

## Files

- `crates/testing/src/fixture_tree.rs` — today's private `Description` becomes the public `TreeDescription`
  (`from_toml_str`, and `merge`, which refuses a path written twice) with
  `build_at(&self, folder: &Path, now: SystemTime)`. `FixtureTree::from_toml*` call it on their temporary folder with
  a fixed "now" (the constant `FIXTURE_NOW`), so every existing fixture builds as before.
- The same file, three new fields: `[[marker]]` (`path`: writes the sandbox marker into that folder, as
  `mark_sandbox` does); `age` (`"<n>d"` or `"<n>h"`, never together with `modified`, counted back from `now`);
  `fill = "data"` (real bytes of a fixed, seeded pattern instead of a sparse file; at most 64 MiB a file and 512 MiB a
  description, refused above).
- `docs/testing.md` — "Fixture homes": the three fields.

## Produces

`TreeDescription` and `build_at` for the case format (02) and the generator (03).

## Checks

- Negative: `age_and_modified_together_are_refused`, `a_fill_over_the_cap_is_refused`,
  `a_marker_outside_the_root_is_refused`, `merging_two_descriptions_that_write_one_path_is_refused`,
  `build_at_refuses_a_path_through_a_link` (a folder of the target replaced by a link before the build).
- Positive: `every_existing_fixture_builds_as_before` (the `TreeReport` of each `fixtures/homes/*.toml` unchanged),
  `an_age_is_counted_back_from_now`, `filled_files_take_their_blocks` (allocated size at least the logical size),
  `a_marker_holds_the_signature`.
- `cargo nextest run -p fresher-testing`; the golden tests of `crates/e2e` unchanged.

## Conclusions

`TreeDescription` is public in `fresher-testing` (`from_toml_str`, `merge`, `build_at`) with `FIXTURE_NOW`; `FixtureTree`
builds through it and the fixture homes' reports did not change. Added: `[[marker]]`, `age`, `fill = "data"` (capped), and
refusal of a path that leaves the folder or passes through a link. `FIXTURE_NOW` is seconds (`u64`), since `SystemTime`
arithmetic is not const. Review finding: an age is parsed with `strip_suffix`, never by slicing at a byte, and computed once.
