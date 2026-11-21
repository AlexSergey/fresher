# 030/01/02 — The owner and the decision

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules

What the module does with each match: find the installed app that owns the folder, decide by the rows of the
design's section 4 — offered, left out with its reason, or not claimed — and report. Pure decisions on plain values,
so every row is a unit test; the module only gathers and reports. The design: [`../00-design.md`](../00-design.md),
sections 2 to 5.

## Files

- `modules/app-caches/src/owner.rs` (new) — on plain values:
  - `pub(crate) struct Owner<'a> { app: &'a InstalledApp, own: bool }`;
  - `pub(crate) fn id_of(name: &OsStr) -> Option<BundleId>`: valid UTF-8, `BundleId::new`, at least one dot;
  - `pub(crate) fn owner<'a>(name: &str, apps: &'a [InstalledApp]) -> Option<Owner<'a>>`: the apps whose bundle id or
    a helper's names `name` (`BundleId::names`), the longest naming id first, ties by the app's path; `own` when `name`
    is the app's bundle id ignoring case. Never a prefix the other way round.
- `modules/app-caches/src/decide.rs` (new) — on plain values, never a port:
  - `pub(crate) enum Reason { Special, NotYours, NoSystem, AppOpen }` with `ALL` and `key()`
    (`app-caches.left-out.<name>`);
  - `pub(crate) enum Verdict<'a> { NotClaimed, LeftOut(Reason), Offered(Owner<'a>) }`;
  - `pub(crate) struct NoSystem;`
  - `pub(crate) fn decide<'a, 'r>(candidate: &Candidate, exclude: &Exclude, apps: Result<&'a [InstalledApp], NoSystem>,
    running: impl FnOnce() -> Result<&'r RunningSet, NoSystem>) -> Verdict<'a>` (two named lifetimes: a borrowed return
    in an `Fn*` bound does not elide): the eleven rows in order; `running` called only when a candidate reaches row 9;
    row 10 keeps when a running app has the owner's bundle id or a
    helper's, or `RunningSet::owner_of(path)` names one.
- `modules/app-caches/src/report.rs` (new) — the text keys made once with the module (as `modules/local-ai-models`):
  `Report::problems()` builds the module's declared left-out keys from `Reason::ALL`, and
  `Report::tell(row, candidate, verdict, out)` pushes a `LeftOutItem { category, reason, sizes }` or a `NewFinding`:
  identity `<row id>/<folder name>`, `Subject::Path` and `Removal::EmptyFolder` of the folder, the match's sizes,
  title `app-caches.title.own` with `app` or `app-caches.title.part` with `app` and `folder`, reason `app-caches.reason`
  with `app` (`TextArg::Text`) and `size` (the logical size, `TextArg::Size`), `for_app(owner's bundle id)`.
- `modules/app-caches/src/gathered.rs` (new) — the candidates of one scan (path, kind, row, sizes, entry facts,
  inside), keyed by its cancel token, as `modules/local-ai-models/src/gathered.rs`.
- `modules/app-caches/src/lib.rs` — `on_match` gathers; `finish` asks `cx.system().installed_apps()` once (only when
  something was gathered), then decides and reports every candidate in path order, the running apps asked at most once
  and only when a candidate reaches row 9 (memoised in a `OnceCell`, since the closure is per candidate), checking
  `cx.cancel()` between candidates (`ModuleError::Cancelled`).
- `modules/app-caches/src/tests.rs` (new) — the module run on matches built by hand (as
  `modules/local-ai-models/src/tests.rs`) with `FakeInstalledApps` and `FakeRunningApps`; the properties of order and of
  a restored context. `modules/app-caches/src/decide/tests.rs` — the tests of `decide.rs`.
- `modules/app-caches/Cargo.toml` — `proptest` as a dev-dependency.

## Produces

The module's answers, for 01/03 (fixtures and golden) and 02.

## Checks

- `owner.rs`, negatives first: `mod negative` `a_name_without_a_dot_has_no_id`, `a_name_not_in_utf8_has_no_id`,
  `a_folder_no_app_names_has_no_owner`, `a_sibling_id_is_not_named` (`com.example.Editorx`),
  `a_vendor_prefix_names_no_app` (`com.example` with `com.example.Editor` installed); `mod positive`
  `the_app_s_own_folder_is_own`, `a_helper_s_folder_is_a_part_of_its_app`, `an_id_under_the_app_s_is_a_part`,
  `the_longest_naming_id_wins`, `two_copies_of_one_app_are_one_owner`, `case_is_ignored`.
- `decide.rs`, negatives first, one test per row, keep before offer: `a_file_or_a_link_is_not_claimed`,
  `an_excluded_id_is_not_claimed`, `nothing_is_claimed_when_the_installed_apps_cannot_be_listed`,
  `a_removed_app_s_cache_is_not_claimed`, `a_folder_read_in_part_is_not_claimed`, `an_empty_folder_is_not_claimed`,
  `a_socket_inside_keeps_it_special`, `an_entry_not_yours_keeps_it`, `it_is_kept_when_the_running_apps_cannot_be_listed`,
  `it_is_kept_while_its_app_runs`, `it_is_kept_while_a_helper_runs`; then `an_installed_app_s_cache_is_offered`,
  `a_helper_s_cache_is_offered_in_its_app_s_name`. Property (proptest):
  `a_candidate_stopped_before_row_9_never_asks_the_running_apps`.
- `tests.rs`: properties (proptest) `a_restored_context_decides_the_same` and
  `the_verdict_does_not_depend_on_the_order_of_matches`; `mod negative` `nothing_left_out_is_also_offered`,
  `a_cancelled_scan_stops_between_candidates`; `mod
  positive` `a_finding_names_the_app_and_the_size`, `a_finding_empties_the_folder`, `a_finding_is_counted_in_its_app_s_line`
  (`for_app`).
- `cargo nextest run -p fresher-module-app-caches`; clippy; fmt; `npm run check:machete`.

## Conclusions

Built as written. `owner` picks the longest id that names the folder (the app's own id, a helper's, or an id under them; never a vendor prefix or a sibling); `decide` is the design's rows in order on plain values, with two lifetimes (the apps, the running set, preflight F4); the running apps are asked once, only when a candidate reaches row 9. The report builds the problems from the reasons, so the declared list and the given list cannot differ. The two module-level properties (order of matches, restored context) live in `src/tests.rs`, since `decide` takes one candidate. Calls taken: a tie between two owners goes to the smaller app path, which only decides the title (safety stays with `owner_of` at row 10); a running helper missing from the installed helper list is caught only when its id names the folder or by the cleaner's open-file check (the accepted risk F7, narrower than first stated). Review: approved; coverage 99.79 %.
