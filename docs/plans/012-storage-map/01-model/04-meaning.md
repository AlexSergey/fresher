# 012/01/04 — By meaning and the apps

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
engine

## Files

- `crates/core/src/finding.rs` — `NewFinding::for_app(self, app: BundleId) -> Self` and `fn app(&self) ->
  Option<&BundleId>`; serialized as `app` (absent when `None`), so the IPC finding gains an optional field. A module
  that knows the app sets it; nothing else changes for the modules (`demo` and `trial` keep `None`).
- `crates/core/src/system/ids.rs`, `crates/core/src/system/running.rs` — the private `named_after` of
  `RunningSet::owner_of` becomes `BundleId::names(&self, name: &str) -> bool` (the id as the whole name, or followed by
  a dot, case ignored), used by both; `owner_of` behaves as before.
- `crates/engine/src/storage/attribution.rs` (new) — `AppPlaces`: worked out at the commit, inside the scan's session,
  with the scan's `SystemFacts`:
  - for each known folder of app data (`AppCaches`, `AppSupport`, `Containers`, `GroupContainers`, `Preferences`,
    `Logs`, `SavedState`), its node (`DiskState::find`), and each first-level child whose name one of an installed
    app's ids names (`BundleId::names`, over the app's own id and its helpers') → that app;
  - each installed app's bundle when a root holds it;
  - the Trash's node (`KnownFolder::Trash`) as a named place.
  The installed apps are listed at most once per scan (the facts are lazy: a scan whose modules never asked lists
  them here); a listing that fails leaves the apps out of the map, logged, never the scan. Nothing is written to disk.
- `crates/engine/src/storage/meaning.rs` (new) — `meaning(state, shares, places, findings) -> Vec<MeaningGroup>`:
  1. the apps, largest first: per app, the counted shares of its findings by category, plus its data folders' and
     bundle's **unclaimed** bytes (`node.sizes − shares.explained(state, node)`), so a folder partly claimed is never
     counted twice; an app no longer installed is named by its bundle id with `installed: false`;
  2. the categories of findings that name no app, largest first;
  3. the Trash, then the largest unclaimed top-level folders of each root (at most 8), each a `PlaceShare`;
  4. the rest of the unclaimed bytes as one `Part(YourFolders)` — the folders the place view lists, not a bucket: its
     row opens the place view.
  The system's parts are appended by 02/04.

  Built: an app's data also holds the unclaimed bytes of its own findings' nodes, so an ignored finding of an app is
  its data wherever it is. Claimed places are taken once each, and only the outermost: an ignored finding of app B
  inside a data folder of app A is A's data (`an_ignored_finding_inside_another_apps_folder_is_that_apps_data`). A
  finding counts in its app's category only where the tree's owner of its node is its own category, each node once.
- `crates/engine/src/results.rs`, `crates/engine/src/session.rs` — the results keep the `AppPlaces` of the committed
  state beside it, replaced with it; a cancelled scan keeps the previous ones.
- `crates/engine/src/storage/mod.rs` — `mod attribution; mod meaning;`.

## Produces

The by-meaning groups for the query (03/02).

## Checks

- Negative first: `a_folder_named_like_an_app_is_not_its_without_the_dot` (`com.example.Appx` for
  `com.example.App`), `a_folder_outside_the_app_data_folders_is_not_attributed`, `an_app_folder_claimed_by_a_module_counts_once`
  (its finding counts in the category, the rest of the folder as the app's data), `an_ignored_finding_of_an_app_is_its_data`,
  `a_cancelled_scan_keeps_the_previous_places`.
- Positive: `an_app_gathers_its_findings_and_its_data`, `apps_come_largest_first`, `the_trash_is_a_named_place`,
  `an_uninstalled_app_is_named_by_its_id`; a **property test**: the leaves of the groups (categories, data, places,
  your folders) add up to the roots' sizes, whatever the owners, apps and ignore lists.
- `cargo nextest run -p fresher-core -p fresher-engine`; the bindings test (`crates/core/tests/typescript.rs`) for the
  new `app` field.

## Conclusions

Built in `cda6565`, minors fixed in `e49b794`, reviewed approved.

- `meaning(state, shares, places, findings, roots)` gives apps largest first, the categories left, the Trash, up to
  eight largest folders of a root and your folders last. `AppPlaces` is computed at every commit with the installed
  apps listed then; a failed listing leaves the apps out and keeps the Trash. `NewFinding::for_app` and
  `BundleId::names` are the only core additions.
- Deviation: an ignored finding that names an app stays in that app's data, unclaimed (design, decision 7, amended
  after the final review). Later: `largest_folders` keeps only folders; a file at the top stays in your folders.
- Measured: the property test (3000 cases) says the leaves add up to the roots; three mutations each make it fail.
  `cargo nextest run -p fresher-core -p fresher-engine` 583 passed.
