# 034/01/02 — The owner, the decision and the report

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** modules

What the module does with a match of a table row: find the installed app with the row's bundle id, decide by plan
030's rows with rows 2 and 4 by the table, and report a finding that empties the cache folder or a left-out in the
row's category. Pure decisions on plain values, so every row is a unit test. The design:
[`../00-design.md`](../00-design.md), sections 2 to 4.

## Context for the implementer

- Read `modules/app-caches/src/{lib.rs,gathered.rs,owner.rs,decide.rs,decide/tests.rs,report.rs,tests.rs}` and
  01/01's `src/apps.rs` first. Plan 030's behaviour for the `library-caches` row must not change: its tests stay
  green with only the mechanical renames below.
- No core change, no new dependency. Negative tests first. Build with
  `CARGO_TARGET_DIR=/Users/serhiialeksandrov/Desktop/Work/Projects/my/github/fresher/target`.
- The new text `app-caches.title.profile` is added to `locales/en.json` in 01/03 with the fixture run that uses it;
  here the report only builds its `TextKey`.

## Files

- `modules/app-caches/src/gathered.rs` — `Candidate.row: usize` becomes `Candidate.source: Source` (01/01's enum).
- `modules/app-caches/src/owner.rs` — `pub(crate) fn installed<'a>(id: &BundleId, apps: &'a [InstalledApp]) ->
  Option<Owner<'a>>`: the apps whose `bundle_id == *id` (ids compare ignoring case), the first by path; `own: true`.
  Never a helper's id, never a prefix.
- `modules/app-caches/src/decide.rs`:
  - `#[derive(Clone, Copy, Debug)] pub(crate) enum Naming<'t> { FolderName, Table(&'t BundleId) }`.
  - `decide(candidate, naming, exclude, apps, running)`: row 1 unchanged; row 2 for `FolderName` unchanged (the
    name's id, then `exclude.keeps`), for `Table(id)` `exclude.keeps(id)`; row 3 unchanged; row 4 for `FolderName`
    `owner(name, apps)`, for `Table(id)` `installed(id, apps)`; rows 5 to 11 unchanged. Its doc comment lists both
    namings.
- `modules/app-caches/src/report.rs`:
  - a `title_profile: TextKey` (`app-caches.title.profile`);
  - `tell_place(row: &places::Row, candidate, verdict, out)` (the former `tell`, unchanged in what it reports);
  - `tell_app(row: &AppRow, candidate, verdict, out)`: a left-out in the row's category with the candidate's sizes;
    or a finding in the row's category: identity `<row id>/<profile>/<cache>` for a row with a profile, else
    `<row id>/<cache>` (the cache is the candidate's file name, the profile the path's name at the `*`'s place,
    counted back from the cache folder: for `Google/Chrome/*` and `.../Google/Chrome/Profile 1/Cache` it is
    `Profile 1`); `Subject::Path` and `Removal::EmptyFolder` of the cache folder; the match's sizes; title
    `app-caches.title.profile` with `app` (the owner's shown name), `profile` and `folder` (the cache name) for a row
    with a profile, else `app-caches.title.part` with `app` and `folder`; reason `app-caches.reason` with `app` and
    `size` (`TextArg::Size(logical)`); `for_app(row.bundle_id)`. A name not in UTF-8 reports nothing (the walk's `*`
    matches only UTF-8 names).
  - the shared parts (the left-out, the finding's sizes and reason) in private helpers, not copied.
- `modules/app-caches/src/lib.rs`:
  - `on_match` gathers every match whose interest has a source (`self.sources.get(matched.interest)`), with that
    source.
  - `finish`: for `Source::Place { row }` as today with `Naming::FolderName` and `tell_place`; for
    `Source::App { row, .. }` with `Naming::Table(&row.bundle_id)` and `tell_app`. The installed apps asked once, the
    running apps at most once and only at row 9, `cx.cancel()` checked between candidates: unchanged.
- `modules/app-caches/src/decide/tests.rs`, `src/tests.rs`, `owner.rs`' tests — the renames (`source`, the `Naming`
  argument) and the new tests below.

## Produces

The module's answers for table rows, for 01/03 and 02.

## Checks

- `owner.rs`, `mod negative`: `no_installed_app_has_the_row_s_id` (none installed; only `com.google.Chrome.helper`
  installed as a helper of another app; `com.google.Chromex`; `com.google`); `mod positive`:
  `the_app_with_the_row_s_id_owns_it`, `the_row_s_id_is_found_ignoring_case`, `two_copies_are_one_owner_first_by_path`.
- `decide/tests.rs`, `mod negative` (table candidates, keep before offer): `a_table_link_or_file_is_not_claimed`,
  `a_table_row_whose_id_is_excluded_is_not_claimed`, `a_table_row_is_not_claimed_when_the_installed_apps_cannot_be_listed`,
  `a_table_row_whose_app_is_not_installed_is_not_claimed`, `a_table_cache_read_in_part_or_empty_is_not_claimed`,
  `a_table_cache_holding_a_socket_is_kept_special`, `a_table_cache_not_yours_is_kept`,
  `a_table_cache_is_kept_when_the_running_apps_cannot_be_listed`, `a_table_cache_is_kept_while_its_browser_runs`,
  `a_table_cache_is_kept_while_only_a_helper_runs`; `mod positive`: `a_table_cache_of_a_closed_installed_app_is_offered`.
  Property (proptest, as the existing one): `a_table_candidate_stopped_before_row_9_never_asks_the_running_apps`.
- `src/tests.rs`, the module on hand-built matches (as the existing tests build them): `mod negative`
  `a_match_of_no_interest_is_not_claimed` (an index past the last source), `a_profile_s_finding_never_names_another_path`;
  `mod positive` `a_browser_cache_is_titled_by_its_browser_profile_and_folder` (`Google Chrome, Profile 1: Code Cache`
  args: `app`, `profile`, `folder`), `a_web_engine_cache_is_titled_by_its_app_and_folder` (`title.part`),
  `a_table_finding_empties_its_folder_and_is_counted_in_the_app_s_line` (`EmptyFolder`, `for_app`),
  `table_identities_name_the_row_the_profile_and_the_cache`, `the_library_caches_row_reports_as_before`.
  The existing properties (`the_verdict_does_not_depend_on_the_order_of_matches`, `a_restored_context_decides_the_same`)
  draw table candidates too.
- `cargo nextest run -p fresher-module-app-caches` (the fixture and contract tests unchanged and green); clippy; fmt.

## Conclusions

Built as the step says: the owner of a table row is the installed app with the row's bundle id (case ignored, first by
path); `decide` takes a `Naming` (the folder's name or the table's id) and rows 2 and 4 follow it, the rest is plan
030's; `report.rs` has `tell_app` beside `tell_place` and the shared helpers. `cargo nextest run -p
fresher-module-app-caches`: 127 tests; every file of the crate 100 % of lines.

Changes from the design: two negative tests beyond the step's (an open browser's caches are left out in
`app-caches.browsers`; a name that is not UTF-8 gets no finding); a non-UTF-8 name suppresses only the finding, a
left-out is still reported with its sizes, as for `library-caches`; the profile is found with `checked_sub`, so no path
panics. A property test shows a candidate stopped before row 9 never asks which apps run.
