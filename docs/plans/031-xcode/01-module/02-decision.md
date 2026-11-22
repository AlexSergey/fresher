# 031/01/02 — The names and the decision

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** modules

What the module does with each match: read the folder's name by its row's shape, decide by the rows of the design's
section 4 — offered, left out with its reason, or not claimed —, mark the newest device support versions, and report.
Pure functions on plain values, so every row is a unit test; the module only gathers and reports. The design:
[`../00-design.md`](../00-design.md), sections 3 to 5. Read first, as the shape to follow (do not depend on it):
`modules/app-caches/src/{decide.rs,decide/tests.rs,gathered.rs,report.rs,tests.rs,lib.rs}`.

## Files

- `modules/xcode/src/name.rs` (new) and its tests — on plain values:
  - `pub(crate) struct Version(Vec<u32>)` with `Ord` by its numbers (`17.10` > `17.9`, `17.2.1` > `17.2`) and `Display`
    (`17.2.1`);
  - `pub(crate) struct DeviceSupportName { model: Option<String>, version: Version, build: String }` and
    `pub(crate) fn device_support(name: &OsStr) -> Option<DeviceSupportName>`: valid UTF-8; `[<model> ]<version>
    (<build>)` where `<version>` is two or three dot-separated runs of ASCII digits (no leading `+`/`-`, each fits a
    `u32`), `<build>` one or more ASCII letters or digits, `<model>` one word of ASCII letters, digits and commas
    followed by exactly one space; nothing before or after;
  - `pub(crate) struct ArchiveName { name: String, date: String }` and `pub(crate) fn archive(path: &Path) ->
    Option<ArchiveName>`: the parent's name is `YYYY-MM-DD` (month 01 to 12, day 01 to 31), the folder's name ends with
    `.xcarchive` (ASCII case ignored) and has at least one character before it; `name` is the folder's name without
    the suffix.
- `modules/xcode/src/decide.rs` (new) and `src/decide/tests.rs` — on plain values, never a port:
  - `pub(crate) enum Reason { Special, NotYours, NoSystem, AppOpen }` with `ALL` and `key()`
    (`xcode.left-out.<special|not-yours|no-system|app-open>`);
  - `pub(crate) const XCODE_APPS: [&str; 2] = ["com.apple.dt.Xcode", "com.apple.iphonesimulator"];`
  - `pub(crate) enum Named { DeviceSupport(DeviceSupportName), Fixed, Archive(ArchiveName) }`;
  - `pub(crate) enum Verdict { NotClaimed, LeftOut(Reason), Offered(Named) }` (the left-out ones keep their `Named` too
    when the newest-versions pass needs it: `LeftOut(Reason, Named)` is fine; choose one and test it);
  - `pub(crate) struct NoSystem;`
  - `pub(crate) fn decide<'r>(candidate: &Candidate, row: &Row, running: impl FnOnce() -> Result<&'r RunningSet,
    NoSystem>) -> Verdict`: the ten rows in order; `running` called only when a candidate reaches row 8; row 9 keeps
    when any running app's `bundle_ids` holds one of `XCODE_APPS` (ASCII case ignored, as `BundleId` compares);
  - `pub(crate) fn recent(offered: …) -> BTreeSet<…>`: per `platform`, the two highest distinct `Version`s among the
    device support candidates that were offered or left out; a candidate whose version is among them is "recent".
- `modules/xcode/src/gathered.rs` (new) — the candidates of one scan (path, kind, interest index, sizes, entry facts,
  `holds`, `inside`), keyed by its cancel token, as `modules/app-caches/src/gathered.rs`.
- `modules/xcode/src/report.rs` (new) — the text keys made once with the module: `Report::problems()` builds the
  declared left-out keys from `Reason::ALL`; `Report::tell(row, candidate, verdict, recent, out)` pushes a
  `LeftOutItem { category, reason, sizes }` or a `NewFinding` as the design's section 5 table says: identity, subject
  `Subject::Path` of the folder, removal `Removal::Trash` or `Removal::EmptyFolder` of the same path by the row's
  `removal`, the match's sizes, the title and the reason with their arguments (`platform`, `version`, `build`, `name`,
  `date` as `TextArg::Text`; `size`, the logical size, as `TextArg::Size`), `for_app("com.apple.dt.Xcode")`.
- `modules/xcode/src/lib.rs` — `on_match` gathers; `finish` decides every candidate in path order, the running apps
  asked at most once and only when a candidate reaches row 8 (memoised in a `OnceCell`), then computes the recent
  versions and reports; it checks `cx.cancel()` between candidates (`ModuleError::Cancelled`). No port but the running
  apps is asked; no command is run.
- `modules/xcode/src/tests.rs` (new) — the module run on matches built by hand (as `modules/app-caches/src/tests.rs`)
  with `FakeRunningApps`.
- `modules/xcode/Cargo.toml` — `proptest` as a dev-dependency (and its row in `docs/dependencies.md`'s used-by column).

## Produces

The module's answers, for 01/03 (fixtures and golden) and 02.

## Checks

- `name.rs`, negatives first: `mod negative` `a_plain_word_is_no_device_support_name` (`Notes`),
  `a_version_of_one_number_is_refused` (`17 (21A1)`), `a_build_without_its_space_or_parentheses_is_refused`
  (`17.2(21C62)`, `17.2 21C62`), `an_empty_build_is_refused` (`17.2 ()`), `four_version_numbers_are_refused`
  (`1.2.3.4 (A1)`), `a_name_not_in_utf8_is_refused`, `an_archive_outside_a_dated_folder_is_refused` (`Archives/x/A.xcarchive`,
  `2026-13-01`, `2026-1-01`), `a_bare_suffix_is_no_archive` (`.xcarchive`); `mod positive`
  `a_version_and_build_is_read` (`17.2 (21C62)`), `a_model_prefix_is_read` (`iPhone15,2 17.2 (21C62)`, `Watch6,1 10.0
  (21R355)`), `three_numbers_are_read` (`16.4.1 (20E252)`), `versions_compare_by_number`, `an_archive_is_read`
  (`2026-03-01/Shop 01-03-2026, 10.15.xcarchive` → name `Shop 01-03-2026, 10.15`, date `2026-03-01`),
  `the_suffix_ignores_case`.
- `decide/tests.rs`, negatives first, one test per row, keep before offer: `a_file_or_a_link_is_not_claimed`,
  `a_name_of_the_wrong_shape_is_not_claimed`, `a_folder_without_its_sign_is_not_claimed`,
  `a_folder_read_in_part_is_not_claimed`, `an_empty_folder_is_not_claimed`, `a_socket_inside_keeps_it_special`,
  `an_entry_not_yours_keeps_it`, `it_is_kept_when_the_running_apps_cannot_be_listed`, `it_is_kept_while_xcode_runs`,
  `it_is_kept_while_the_simulator_runs`; then `a_device_support_folder_is_offered`, `the_previews_are_offered`,
  `the_simulator_caches_are_offered`, `an_archive_is_offered`, `the_two_newest_versions_of_a_platform_are_recent`
  (three iOS versions and one watchOS: two iOS and the watchOS one recent), `a_left_out_version_still_counts_as_newest`.
  Property (proptest): `a_candidate_stopped_before_row_8_never_asks_the_running_apps`.
- `tests.rs`: properties (proptest) `a_restored_context_decides_the_same` and
  `the_verdict_does_not_depend_on_the_order_of_matches`; `mod negative` `nothing_left_out_is_also_offered`,
  `a_cancelled_scan_stops_between_candidates`, `no_command_is_run` (a `FakeCommandRunner` in the context records no
  call); `mod positive` `a_device_support_finding_names_its_platform_version_and_build`,
  `a_recent_version_has_its_own_reason_and_the_same_danger`, `the_caches_are_emptied_and_the_rest_moved_to_the_trash`,
  `every_finding_is_counted_in_xcode_s_line`, `an_archive_finding_names_its_date`.
- `cargo nextest run -p fresher-module-xcode`; clippy; fmt; `npm run check:machete`.

## Conclusions

Built as the step says: the names (`[<model> ]<version> (<build>)`, the archive's dated folder), the decision rows in the
design's order and the report. 75 tests, 99.5 % of lines. Changes: `Verdict::LeftOut` carries `(Reason, Named)` so a
left-out version still counts towards the two newest; `Version` compares by its numbers (`17.02` equals `17.2`);
`XcodeError::AppId` was added; the fixed rows choose their texts by category.
