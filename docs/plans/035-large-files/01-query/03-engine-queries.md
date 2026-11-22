# 035/01/03 — The engine's queries

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** engine

The two queries of the Largest screen over the last committed state, and the one judgement of whether a row can be
chosen: `Engine::largest` (Files) and `Engine::largest_place` (Folders). Pure reads, never blocking a scan; nothing is
removed or picked here (02/01). The design: [`../00-design.md`](../00-design.md), sections 1, 3 and 4.

Read first: `crates/engine/src/disk.rs`, `crates/engine/src/storage/query.rs` (`Snapshot`, how a query takes the
committed state under the results' lock and builds after letting it go), `crates/engine/src/storage/place.rs`
(`place_view`, `Descent::Stay`), `crates/engine/src/storage/shares.rs` (`Shares::explained`), `crates/engine/src/results.rs`
(`Committed`, `keep_state`, `shares`), `crates/engine/src/settings.rs` (`Ignored::hides_place`, reading a setting),
`Engine::home()` in `crates/engine/src/engine.rs`, and the cleaner's `Protection::check` (`crates/core/src/cleanup/protection.rs`,
reached as `engine.rs` does: `self.shared.cleanup…cleaner.protection()`).

## Files

- `crates/engine/src/largest/mod.rs` (new) — `impl Engine`:
  - `pub fn largest(&self, query: &LargestQuery) -> Result<LargestPage, CommandError>`;
  - `pub fn largest_place(&self, folder: Option<NodeId>, offset: u32, limit: u32) -> Result<OwnPlace, CommandError>`;
  - both: `NoScan` before a commit, `Busy { running: Operation::Scan }` while a scan runs, as `storage_place`
    (make `storage/query.rs`'s `snapshot` `pub(crate)` and reuse it rather than copy it); `StaleNode` for a node of
    another state; `NotOwnItem` for a folder or a place filter outside the home; a limit capped at `MAX_PAGE` (Files)
    and the place view's cap (Folders). The home is `Engine::home()`, found in the state with `find`; no home in the
    state (another root was scanned) answers an empty page, never an error.
- `crates/engine/src/largest/rule.rs` (new) — `HomeItems`, the `ItemRule` of the design's section 1, pure: depth 1
  `Library` and `.Trash` (the names of `KnownFolder::Library` and `KnownFolder::Trash` as resolved under the home, given
  to the rule when it is made) → `Skip`; a name starting with `.` → `Skip`; `is_app_bundle` → `Skip`; a folder that
  `is_package` → `Item`; any other folder → `Enter`; a file and a collapsed subtree → `Item`; a link → `Skip`.
- `crates/engine/src/largest/items.rs` (new) — the items of one state and minimum, computed once and kept:
  `pub(crate) struct Item { id: NodeId, path: PathBuf, allocated: u64, kind: ItemKind, modified: Option<Timestamp>,
  top: Option<NodeId> }` and `pub(crate) struct Items { min: ByteSize, items: Vec<Item>, truncated: bool }`, built
  from `state.items(home, &rule, min, MAX_ITEMS + 1)` (one more than kept, to tell `truncated`). Kept in `Results`
  beside the shares: `largest: Arc<Mutex<Option<Arc<Items>>>>` (or a `OnceLock` per minimum), reset by `keep_state`
  and rebuilt when the setting's minimum differs.
- `crates/engine/src/largest/judge.rs` (new) — `pub(crate) fn judge(input: &Judged<'_>) -> (Offer, bool)` (the offer and
  whether the folder opens), pure over plain values: the node's `NodeInfo`, its path, whether any ancestor below the
  home is hidden, the home's Library and Trash paths, the protection's answer for the path, the explained sizes, and
  whether the ignore list hides the path. The thirteen rows of the design's section 3, in order, first match wins.
- `crates/engine/src/largest/page.rs` (new) — the Files page from `Items`: drop the ignored paths and the `gone` ones
  (an empty list until 02/01 fills it; take it as a parameter now), filter by `kind`, `place` (`top == place`) and
  `unchanged` (`modified` strictly before `Unchanged::before(state.taken())`; a row with no time is kept only for
  `Any`), count `kinds` and `places` on the list filtered by everything but that filter itself (so a filter never
  hides its own other choices), sum `sizes`, then page; for each row of the page: `NodeInfo` (`state.node`), `place`
  (the parent's path from the home, written `~/a/b`, lossy, `~` for the home itself), `kind`, `offer` (`judge`),
  `unpacked` (for an archive, `archive_stem` names a folder beside it: `state.find(parent.join(stem))` is a folder),
  `picked: None` (02/01 fills it).
- `crates/engine/src/largest/place.rs` (new) — the Folders page: `place_view(state, shares, folder, Descent::Stay,
  offset, limit)` of the folder (the home when `None`), its `path` replaced by the way from the home (each ancestor's
  `NodeInfo`, the home first), each row with `kind_of`, `judge` and `opens`; a folder whose judgement does not open is
  refused as a folder to open (`NotOwnItem`): Folders never shows the inside of `Library`, the Trash, a package, an
  app, a claimed or hidden folder.
- `crates/engine/src/lib.rs` — `mod largest;`. `crates/engine/src/results.rs` — the items' slot and its reset.

## Produces

`Engine::largest`, `Engine::largest_place`, `judge` (02/01 reuses it to refuse a pick that is not offered).

## Checks

Tests on `FixedDiskState` trees with the engine's test builder (`crates/engine/src/testing.rs`), negatives first:

- `rule.rs`: `mod negative` `the_library_is_skipped_only_at_the_top` (a `Library` deeper is entered), `the_trash_is_skipped`,
  `a_hidden_name_is_skipped`, `an_app_bundle_is_skipped`, `a_link_is_skipped`; `mod positive` `a_package_is_one_item`,
  `a_plain_folder_is_entered`, `a_file_is_an_item`.
- `judge.rs`: one test per row, in the table's order, a withheld row before the offer: `a_claimed_node_is_explained`,
  `the_library_is_withheld_and_does_not_open`, `the_trash_is_withheld`, `a_hidden_folder_and_everything_in_it_are_withheld`,
  `an_app_is_withheld`, `a_link_is_withheld`, `a_folder_not_scanned_is_not_seen`, `a_standard_folder_opens_but_is_withheld`,
  `a_protected_place_is_withheld`, `a_folder_holding_explained_bytes_opens_but_is_withheld`,
  `a_folder_read_in_part_is_withheld`, `an_ignored_place_is_withheld`; then `a_plain_file_is_offered`,
  `a_plain_folder_is_offered_and_opens`, `a_package_is_offered_and_does_not_open`.
- `page.rs` / `mod.rs`: `mod negative` `no_scan_is_refused`, `a_running_scan_is_busy`, `a_place_of_another_state_is_stale`,
  `a_place_outside_the_home_is_refused`, `nothing_below_the_minimum_is_listed`, `a_claimed_file_is_not_listed`,
  `nothing_under_the_library_or_the_trash_is_listed`, `an_ignored_path_is_not_listed`, `a_file_changed_after_the_cutoff_is_filtered_out`,
  `a_scan_of_another_root_answers_an_empty_page`; `mod positive` `the_largest_come_first`, `a_package_is_one_row`,
  `the_place_is_written_from_the_home`, `the_kinds_and_places_count_the_other_filters_only`, `the_sizes_sum_the_filtered_rows`,
  `an_archive_beside_its_folder_names_it`, `the_items_are_computed_once_per_state_and_minimum` (a counting
  `DiskState`), `a_changed_minimum_recomputes_them`, `more_than_the_cap_says_truncated`, `the_page_is_capped`.
- `place.rs`: `mod negative` `the_library_does_not_open`, `a_package_does_not_open`, `a_claimed_folder_does_not_open`;
  `mod positive` `the_home_opens_with_no_folder_given`, `the_way_back_starts_at_the_home`, `rows_carry_their_explained_part`.
- `cargo nextest run -p fresher-engine`; clippy; fmt; `npm run check:crate-deps` (the engine still depends on the core
  alone).
- Commit: `feat(engine): the largest files and folders queries`.

## Conclusions

- `Engine::largest` and `Engine::largest_place` over `largest/` (`judge.rs` with 13 rows, `items.rs`, `page.rs`,
  `place.rs`, `rule.rs`); the items are computed once per state and minimum and kept beside the state.
- Two review rounds changed the design's judgement: the judge is blind to nothing above a row, so `Ground::judge_reached`
  asks every folder above (a package, a claimed folder, a protected place); a folder holding any claimed node is
  withheld (an ignored owner counts too), and a folder holding an ignored path is withheld but opens. The design's table
  (rows 1, 4, 10, 12) and call 12 were updated.
- `Items.items` is `Items.list`; `Item.allocated` was dropped; `FixedDiskState::with_modified` was added for the
  `Unchanged` filter's tests. 1 163 tests passed in engine, scanner and testing after the second round.
