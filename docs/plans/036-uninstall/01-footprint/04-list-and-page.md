# 036/01/04 — The list, the page and the commands

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** engine, app

The engine's view of the installed apps over the last committed scan: each app's footprint (its bundle and the data
places named after its ids), the rows that decide whether it is listed and offered, the last-used date, the Apps list
with its filter, an app's page, and Show in Finder; the three commands. Read-only: nothing here prepares or removes
anything. The design: [`../00-design.md`](../00-design.md), sections 2, 3 (cost), 5, 11 and 12.

## Before you start

Read `docs/architecture/README.md` ("The storage map", "Problems and reasons"), `crates/engine/src/storage/attribution.rs`
(`AppPlaces`, `named_after`, `ids`) and its tests, `crates/engine/src/session.rs` (`commit`: where `AppPlaces::of`
runs with the scan's `SystemFacts`), `crates/engine/src/results.rs` (`keep_state`, `places`, the ignore lists),
`crates/engine/src/engine.rs` (`Shared`, the public methods, `open_access_settings` for the shape of a command that
calls `SystemUi`), `crates/engine/src/reveal.rs`, `crates/engine/src/disk.rs` (how a query reads the state's `Arc`
outside the lock), `app/tauri/src/storage_commands.rs` and `app/tauri/src/lib.rs` (how commands are added). 01/01's
types and 01/03's `InstalledApps::bundle` exist.

## Files

- `crates/engine/src/storage/attribution.rs` — `AppPlaces` keeps the installed list it was made from (`installed:
  Option<Vec<InstalledApp>>`, `None` when it could not be listed) with `pub(crate) fn installed(&self) ->
  Option<&[InstalledApp]>`; `ids` and `named_after` become `pub(crate)` for the footprint. Nothing else changes; its
  tests pass unchanged.
- `crates/engine/src/apps/mod.rs` (new) — the module's entry points; `crates/engine/src/lib.rs`: `mod apps;`.
- `crates/engine/src/apps/places.rs` (new) — the design's section 5 table as `const PLACES: &[Place]`, each `Place {
  kind: FootprintKind, folder: KnownFolder, under: &[&str], name: NameShape }` with `NameShape::{Id, IdWithSuffix(&str),
  IdDotted(&str), TeamPrefixed}`: `caches` (`AppCaches`, `Id`), `logs` (`Logs`, `Id`), `saved-state` (`SavedState`,
  `IdWithSuffix(".savedState")`), `launch-agents` (`Library`, `["LaunchAgents"]`, `IdDotted(".plist")`: `<id>.plist`
  or `<id>.<more>.plist`), `web-data` (`Library`, `["HTTPStorages"]`, `Id` and `IdWithSuffix(".binarycookies")`;
  `["WebKit"]`, `Id`; `["Cookies"]`, `IdWithSuffix(".binarycookies")`), `settings` (`Preferences`,
  `IdWithSuffix(".plist")`), `support` (`AppSupport`, `Id`), `container` (`Containers`, `Id`), `group-container`
  (`GroupContainers`, `TeamPrefixed`). A test pins the table against `FootprintKind::ALL` (every kind but `Bundle` has a
  row).
- `crates/engine/src/apps/footprint.rs` (new) — pure: `fn footprint(state: &dyn DiskState, system: &dyn KnownFolders,
  installed: &[InstalledApp], app: usize, cx: &FootprintCx) -> Vec<Part>` with `Part { kind, node: NodeId, path,
  sizes, modified, kept: Option<KeptReason> }`, the children of each place's folder (`state.find`, `children` by name,
  every page) whose name has the shape and names the app by `named_after` (the longest naming id, so a helper's folder
  is its own app's and `com.example.Editorx` is nobody's); `TeamPrefixed`: `<team id>.` then anything, the app's
  `team_id`. `FootprintCx { other_apps_data: Access, ignored: &[PathBuf] }`. The kept reasons in the design's order:
  `shared` (another installed bundle at another path has the naming id among its own or its helpers': two copies),
  `team-shared` (another installed app has the same team id, or the app has none), `link` (`NodeKind::Link`),
  `needs-permission` (`container`/`group-container` while `other_apps_data` does not allow), `not-scanned`
  (`not_scanned` set or sizes not `Complete`), `ignored` (the path, a path inside it or one holding it is ignored).
- `crates/engine/src/apps/list.rs` (new) — `fn rows(…) -> Vec<Listed>`: the design's section 2 rows 1 to 8 in order
  (row 3 uses the located `Applications` and `UserApplications` folders and `Cleaner::protection()`; row 7 the running
  apps asked once per listing: a running app whose `bundle_ids` hold the app's id or a helper's); the summary text
  (`core.apps.summary.unused` when the last use is older than 30 days by the engine's clock, `.used` when newer,
  `.unknown` when there is none: design section 5, "Last used"); the parts summed by kind (kept ones too); the list
  largest first by allocated size, then by name; the filter (`AppsQuery::unused`: an app with no known date is never in
  a filtered list).
- `crates/engine/src/apps/cache.rs` (new) — `AppsCache { state: Option<StateId>, facts: Vec<Option<Result<BundleFacts,
  SystemError>>> }` in `Shared` (`apps: Mutex<AppsCache>`): the bundle facts of the current state, measured on the
  first listing (or page) after a commit, at most four threads (`std::thread::scope`), **outside every engine lock**:
  the state's `Arc` and the installed list are cloned out first, the facts stored after; a state that changed while
  measuring drops the result. A commit makes the cache stale by its state id; nothing is measured during a scan (the
  list answers from the previous state, or `NoScan`).
- `crates/engine/src/apps/page.rs` (new) — `fn detail(…) -> AppDetail`: the summary, one `FootprintItem` per part
  (bundle first, then the table's order, then by path), each choice's `ChoiceView` (the bundle and the parts that go
  with it, not kept: sizes summed, count), the categories of the current visible findings whose `app()` is the app's
  id or a helper's (in the registry's order), `command` = `brew uninstall --cask <token>` when installed by Homebrew,
  `notice` `None` (02/02 sets it).
- `crates/engine/src/engine.rs` — `Shared` gains `apps: Mutex<AppsCache>`; at the end of `impl Engine`: `pub fn
  apps(&self, query: &AppsQuery) -> Result<AppsView, CommandError>` (`NoScan` before a commit), `pub fn app(&self,
  id: AppId) -> Result<AppDetail, CommandError>` (`UnknownApp` for another state's id or an index past the list, or an
  app not listed), `pub fn reveal_app(&self, id: AppId) -> Result<(), CommandError>` (`SystemUi::reveal` of the
  bundle; refused `UnknownApp` as above).
- `app/tauri/src/apps_commands.rs` (new) — `get_apps(query)`, `get_app(app)`, `reveal_app(app)`, each
  `#[tauri::command(async)]` mapping `CommandError` to `Refusal`; registered in `app/tauri/src/lib.rs`'s invoke handler
  and allowed as the existing commands are (follow `storage_commands.rs`); the dev run record wraps them as it wraps
  every command.
- `app/ui/src/ipc/bindings.generated.ts` — regenerated.

## Produces

`Engine::apps`, `Engine::app`, `Engine::reveal_app`, the footprint and the cache, for 02/01 (the parts of a choice) and
03 (the screens).

## Checks

Engine tests on `FixedDiskState`, `FakeKnownFolders`, `FakeInstalledApps::with_bundle`, `FakeRunningApps`,
`FakeClock`, `FakeSystemUi`; no real disk.

- `footprint.rs`, `mod negative`: `a_look_alike_id_is_not_a_part` (`com.example.Editorx`), `a_vendor_folder_is_not_a_
  part` (`com.example`), `a_helper_s_folder_of_another_app_is_not_a_part` (the longest naming id), `a_name_without_the_
  shape_is_not_a_part` (`Preferences/com.example.Editor.plist.bak`, `LaunchAgents/com.example.Editorx.plist`),
  `a_second_copy_keeps_the_data_shared`, `a_group_container_of_a_shared_team_is_kept`, `an_app_without_a_team_keeps_
  its_group_containers`, `a_link_is_kept`, `a_container_without_other_apps_data_is_kept`, `a_folder_not_read_in_full_is_
  kept`, `an_ignored_folder_is_kept`; `mod positive`: `each_place_of_the_table_is_found` (one fixture state with every
  kind), `a_launch_agent_with_a_dotted_name_is_found`, `a_group_container_of_the_app_s_own_team_is_found`.
- `list.rs`, `mod negative`: one test per row 1 to 7 (`nothing_is_listed_when_the_apps_cannot_be_listed`,
  `apple_s_apps_are_not_listed`, `an_app_outside_the_apps_folders_is_not_listed`, `fresher_s_own_bundle_is_not_listed`,
  `an_unmeasured_bundle_is_not_offered`, `a_homebrew_app_is_not_offered`, `a_root_owned_bundle_needs_an_administrator`,
  `a_locked_bundle_is_not_offered`, `an_open_app_is_not_offered`, `an_open_helper_keeps_its_app_from_being_offered`),
  `an_app_with_no_date_is_not_in_a_filtered_list`; `mod positive`: `an_app_is_offered_with_its_size_and_parts`,
  `the_list_is_largest_first`, `the_last_use_is_the_later_of_spotlight_and_the_data`, `the_unused_sentence_names_the_
  date_and_the_size`, `the_filter_keeps_apps_unused_for_its_days`.
- `cache.rs`: `the_bundles_are_measured_once_per_state` (the fake counts `bundle()` calls), `a_new_commit_measures_
  again`, `a_result_of_an_older_state_is_dropped`, `the_measure_holds_no_engine_lock` (a scan started while a slow fake
  measures does not wait).
- `page.rs` and `engine.rs`: `an_id_of_another_state_is_unknown`, `an_index_past_the_list_is_unknown`; `the_page_lists_
  every_part_in_order`, `each_choice_counts_only_what_goes`, `the_page_links_the_app_s_categories`, `a_homebrew_app_s_
  page_gives_the_command`, `reveal_shows_the_bundle_in_finder`.
- `cargo nextest run -p fresher-engine -p fresher-app`; clippy; fmt; `git diff -- crates/cleaner modules` empty.

## Conclusions

- Built in `crates/engine/src/apps/` (`list.rs`, `footprint.rs`, `places.rs`, `cache.rs`, `page.rs`) and
  `app/tauri/src/apps_commands.rs`: the list, the page, `reveal_app`. Engine 883 tests then 1230 with the commands;
  coverage per file 93 to 100 % (`places.rs` 83 %, a `const fn` run at compile time).
- A review's fixes: an incomplete installed list keeps every part but the bundle; one measure for two listings (a
  gate); a scan cancels the measure; a part named only by a helper id outside the app's own prefix is kept; Fresher's
  own id is never listed whatever the protection; during a scan with no measured facts the answer is `Busy`.
- A group container of another team's prefix that names the app is listed and kept `team-shared`; only the app's own
  team's can go. When the running apps cannot be listed every app counts as open (fail closed).
