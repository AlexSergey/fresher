# 021/02/02 — The decision

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** modules

What the module does with the projects of a scan: gather the folders it matched, then in `finish` leave out the
projects that are not projects here, group the rest by their outermost project, and decide every artifact and cache
by the rules of the design's section 3.4 — offered, left out with its reason, or not claimed. Pure decisions on plain
values, so every row is a unit test; the module only gathers and reports. The design:
[`../00-design.md`](../00-design.md), sections 3.1, 3.2 and 3.4 (agent's calls 7 to 11).

## Files

- `modules/stale-projects/src/gathered.rs` (new) — the folder paths the module matched in one scan, keyed by its
  cancel token, as `modules/downloads/src/gathered.rs`: a folder match of any interest adds its path; a file, a link or
  an `Other` match adds nothing.
- `modules/stale-projects/src/place.rs` (new) —
  - `pub(crate) struct Places { home: PathBuf, outside: Vec<PathBuf> }`, `Places::of(system: &SystemFacts) ->
    Option<Places>` (`None` when the home cannot be located; `outside` = the located `Library`, `Applications`,
    `UserApplications`, `SystemApplications`);
  - `Places::holds(&self, root: &Path) -> bool`: false for the rows of the design's section 3.2 (the home or an
    ancestor of it; inside one of `outside`; a component below the filesystem root ending `.app`, ASCII case ignored;
    a component directly under the home starting with `.`), compared by components, never by string;
  - `pub(crate) struct Group<'a> { pub(crate) outermost: &'a Project, pub(crate) members: Vec<&'a Project> }` and
    `pub(crate) fn groups<'a>(projects: &'a [Project], places: &Places) -> Vec<Group<'a>>`: the projects `places`
    holds, each in the group of the one holding project inside no other (the outermost first among its members), the
    groups by the outermost's root.
- `modules/stale-projects/src/decide.rs` (new) — on plain values, never a port:
  - `pub(crate) struct Scan { pub(crate) now: Timestamp, pub(crate) days: u32, pub(crate) restored: bool }`;
  - `pub(crate) enum Staleness { Restored, Active, Unread, Stale { since: Timestamp } }` and
    `pub(crate) fn staleness(activity: LastChange, scan: &Scan) -> Staleness` (the design's group table; "within the
    days" a time after `now − days × 86 400 s`, or after `now`);
  - `pub(crate) fn artifact(artifact: &Artifact, matched: bool, staleness: Staleness) -> Verdict` (the artifact
    table);
  - `pub(crate) fn cache(cache: &Artifact, matched: bool, scan: &Scan) -> Verdict` (the cache table, with
    `CACHE_UNUSED_DAYS`);
  - `pub(crate) enum Verdict { NotClaimed, LeftOut(Reason), Offered }`, `pub(crate) enum Reason { ScanAgain, Unread,
    Recent, CacheRecent }` with `ALL` and `key()` (`stale-projects.left-out.<name>`);
  - `pub(crate) fn less(sizes: Sizes, offered: &[Sizes]) -> Sizes`: field by field, saturating, the completeness kept.
- `modules/stale-projects/src/report.rs` (new) — the text keys made once with the module (as
  `modules/downloads/src/report.rs`): `Report::tell(…)` pushes a `LeftOutItem { category, reason, sizes }` or a
  `NewFinding`: identity `<category name>/<path relative to the home>` (the whole path when outside it), `Subject::Path`
  and `Removal::MoveToTrash` of the path, the item's sizes, title `stale-projects.title` with `project` (the outermost
  root's last component, lossy) and `folder` (the path relative to the outermost root, lossy) as `TextArg::Text`,
  reason `stale-projects.reason.<category name>` with `date` (`TextArg::Date`, the group's `since`) and `command`
  (`TextArg::Text`, the row's), or for a cache `days` (`TextArg::Count`, 7) and `command` (the `target` row of the
  project's Rust kind).
- `modules/stale-projects/src/lib.rs` — `on_match` gathers; `finish`: `Places::of(cx.system())` (none: return
  `Ok(())`, nothing claimed), `cx.system().projects()` (none: `Ok(())`), `groups`, then per group, in root order,
  checking `cx.cancel()` between groups (`ModuleError::Cancelled`):
  1. `staleness(group.outermost.last_activity, &scan)` with `scan` from `cx.clock()`, `UNUSED_FOR.days(cx.settings())`
     and `cx.is_restored()`;
  2. every artifact of every member, by path, each path once: its row (`artifacts.row(kind, name)` for the member's
     kinds that list its folder name, the first in `ProjectKind::ALL` order) gives the category and the command;
     `decide::artifact`; reported;
  3. for every member of `ProjectKind::Rust` whose `target` artifact was not offered: every cache, by path,
     `decide::cache`, reported; that `target`, if it was left out, is reported with `less(target.sizes, offered
     caches' sizes)` — so 2 holds a left-out `target` back until its caches are decided.
- `modules/stale-projects/src/tests.rs` (new) — the module run on a `SystemFacts` whose projects are set by hand
  (`set_projects`) and matches built by hand, as `modules/downloads/src/tests.rs` builds its matches.
- `modules/stale-projects/Cargo.toml` — `proptest` as a dev-dependency.

## Produces

The module's answers, for 02/03 (fixtures and golden) and 03.

## Checks

- `place.rs`, negatives first:
  - `mod negative`: `the_home_and_its_ancestors_are_not_projects`, `nothing_is_held_when_the_home_is_unknown`,
    `a_project_in_library_or_an_apps_folder_is_not_held`, `a_project_inside_an_app_bundle_is_not_held` (`Tool.APP` in
    any case), `a_project_in_a_hidden_folder_of_the_home_is_not_held` (`~/.config/x`), `a_hidden_folder_deeper_down_is_held`
    (`~/Code/.archive/x` is held: only the home's own hidden folders are tools'), `a_look_alike_prefix_is_not_inside`
    (`~/Library2/x` is held);
  - `mod positive`: `a_monorepo_is_one_group_under_its_outermost_project`, `two_sibling_projects_are_two_groups`,
    `a_project_inside_a_left_out_project_is_its_own_outermost` (`~/package.json` beside `~/Code/app`).
- `decide.rs`, negatives first, one test per row of each table, keep before offer:
  `a_restored_scan_keeps_everything_to_scan_again`, `a_project_changed_within_the_days_is_active`,
  `a_project_changed_in_the_future_is_active`, `an_unread_project_older_than_the_days_keeps_its_artifacts`,
  `an_unknown_activity_keeps_its_artifacts`, `an_artifact_read_in_part_is_kept_unread`,
  `an_artifact_the_module_did_not_match_is_not_claimed`, `a_cache_built_within_seven_days_is_kept`,
  `a_cache_read_in_part_is_kept_unread`, `an_at_least_cache_time_within_seven_days_is_kept`; then the offers
  `an_artifact_of_a_stale_project_is_offered`, `a_cache_unused_for_seven_days_is_offered_whatever_the_project_s_age`;
  then `less_never_goes_below_zero_and_keeps_the_completeness`. Properties (proptest):
  `a_later_activity_never_turns_a_keep_into_an_offer`, `a_later_cache_change_never_turns_a_keep_into_an_offer`,
  `a_restored_scan_never_offers`.
- `tests.rs`: `mod negative` `nothing_is_claimed_before_the_projects_are_known` (`projects()` `None`),
  `framework_output_is_never_claimed` (a stale Node project's `dist` and `.next` matched by no interest and named by no
  project: no finding, no left-out), `the_offered_target_s_caches_are_not_offered_again`,
  `a_cancelled_scan_stops_between_groups`; `mod positive` `a_stale_monorepo_offers_every_package_under_its_name`
  (titles `shop · node_modules`, `shop · packages/web/node_modules`), `an_active_rust_project_offers_its_old_cache_and_keeps_the_rest`
  (the `target` left out `recent` with its sizes less the cache's), `the_matches_in_two_orders_give_the_same_answers`,
  `the_setting_moves_the_days`.
- `cargo nextest run -p fresher-module-stale-projects`; clippy; fmt; `npm run check:machete`.

## Conclusions

`gathered.rs`, `place.rs`, `decide.rs` (with `decide/tests.rs`), `report.rs` and `tests.rs` are in place; `on_match`
gathers the matched folders and `finish` decides group by group. Beyond the brief, by the lead's rulings and the
preflight: a group's last activity is the newest of its members' own (`decide::group_activity`, design agent's call
15; module test `a_commit_in_a_nested_repository_keeps_the_whole_group`); an artifact with no row is not claimed and
`framework_output_is_never_claimed` hands `dist` and `.next` in as artifacts and as matches (call 16); a restored scan
offers nothing (`a_restored_scan_offers_nothing`, call 17); the cache table has one test per row
(`a_cache_the_module_did_not_match_is_not_claimed`, `an_unknown_or_old_at_least_cache_is_kept_unread`, and the
restored test calls `decide::cache`); sizes `Unknown` count as read in part (call 18). The table's commands refuse a
blank or space-padded text, and the step's three `expect(dead_code)` are gone. The key tests were checked to fail
when the rule they guard is removed. `serde_json` joins `proptest` as a dev-dependency (the setting test stores the
days).
