# 035/04/01 — The core e2e and the budget

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** e2e, engine, fixtures, docs

**The removal of a user's own file is proven here: use the strongest model.** The Largest queries and the pick through
the real scanner, the trial module, the real settings file, scan cache and journal, and the real `DiskCleaner` reaching
only inside the sandbox with a fake Trash, as the other core e2e flows; and the items query's budget on the generated
`LARGE` tree, run by hand. The design: [`../00-design.md`](../00-design.md), sections 1 to 6 and 9.

Read first: `crates/e2e/tests/logs.rs` and `crates/e2e/tests/logs_home/mod.rs` (a flow over a fixture home: the
model), `crates/e2e/tests/support/` (`home_engine.rs`, `home_scan.rs`, `home_cleanup.rs`, `home_rescan.rs`),
`crates/e2e/tests/storage_budget.rs` and `crates/engine/src/bench.rs` (`SharesBench`: how an ignored by-hand budget
test reaches the engine's internals), `docs/testing.md` ("Fixture homes", "Performance").

## Files

- `fixtures/homes/largest.toml` (new) — a fixture home (sparse files are fine here: set the minimum to `10-mb` and give
  the listed files real blocks with `fill = "data"`, 12 to 16 MiB each; the negatives may be sparse where their size
  does not matter):
  - negatives: a trial cache beside a marker holding a 12 MiB file (`home/project/fresher-trial-cache/blob.bin`,
    claimed); `home/Library/Application Support/Big/data.bin`; `home/.Trash/old.dmg`; `home/.android/avd/disk.img`;
    `home/Applications/Tool.app/Contents/Resources/big.bin`; `home/VMs/Old.utm/Images/disk.qcow2` (inside a package:
    the package is the row, not this file); `home/Downloads/small.zip` (8 MiB, below the minimum); a link
    `home/link-to-big` → `home/Movies/Trip.mov`;
  - positives: `home/Downloads/Installer.dmg` (16 MiB), `home/Movies/Trip.mov` (14 MiB), `home/VMs/Old.utm` (its
    inner disk, 12 MiB), `home/Downloads/Photos.zip` (12 MiB) beside a folder `home/Downloads/Photos/` (with one small
    file), `home/Old projects/archive/data.bin` (12 MiB, for a folder pick).
- `crates/e2e/tests/largest_home/mod.rs` (new) — the flow's helpers: the home, the engine with the trial module, the
  setting `core.largest.min-size = "10-mb"`, the scan, `largest`, `largest_place`, `pick_own_item`, the cleanup with the
  fake Trash, put back.
- `crates/e2e/tests/largest.rs` (new) — the flows; the first scan's Files page and the home's Folders page are the
  golden `fixtures/golden/largest-flow.json` (`UPDATE_GOLDEN=1`, the diff reviewed):
  - `mod negative`: `a_claimed_folder_is_never_listed`, `nothing_under_the_library_the_trash_or_a_hidden_folder_is_listed`,
    `an_app_bundle_and_a_link_are_never_listed`, `a_package_s_inner_file_is_never_listed`,
    `a_file_below_the_minimum_is_not_listed`, `a_standard_folder_cannot_be_picked` (`home/Downloads`: `NotOwnItem`),
    `a_picked_row_is_never_on_scan` (the overview's totals, the category pages and the map's explained part are the
    same before and after the pick), `a_package_in_use_moves_nothing` (`home/VMs/Old.utm`'s disk held open by the fake open
    files: the cleanup skips the item `InUse` and every file of it is still on disk);
  - `mod positive`: `the_files_list_ranks_the_home_s_items`, `a_package_is_one_row_with_its_kind`,
    `an_archive_beside_its_folder_is_pointed_out`, `the_folders_view_opens_through_the_home`,
    `a_picked_file_goes_to_the_trash_at_high_and_leaves_the_list`, `a_picked_file_comes_back_by_put_back`,
    `a_picked_folder_goes_whole`, `a_restored_scan_lists_the_same_rows` (a new engine on the same data folder at
    launch), `a_rescan_after_put_back_lists_it_again`.
- `crates/engine/src/bench.rs` — `LargestBench`, as `SharesBench`: the engine's `HomeItems` rule and the items of a
  state under it, for the budget test only.
- `crates/e2e/tests/largest_budget.rs` (new) — `#[ignore]`, run by hand in release: the real scanner commits the
  `LARGE` tree with the claims of `storage_budget.rs`'s interests, then the items of its home at the default minimum:
  prints the time and the heap of the kept items; asserts under 50 ms and 5 MB
  (`cargo nextest run -p fresher-e2e --release --run-ignored only --no-capture items_of_the_large_tree`).
- `crates/e2e/Cargo.toml` — what the new tests need (no new external dependency).
- `docs/testing.md` — the new flow in the list of core e2e flows; the budget test's command in "Performance", beside
  the shares'.

## Checks

- `cargo nextest run -p fresher-e2e --test largest`; the budget test once by hand in release, its numbers written in
  this step's conclusions (and in 05/01's).
- `git diff -- crates/cleaner` is empty.
- Commit: `test(e2e): the largest files end to end and the items budget`.

## Conclusions

- `crates/e2e/tests/largest.rs`: 17 flows (8 negative, 9 positive) over `fixtures/homes/largest.toml`, all passing;
  golden `fixtures/golden/largest-flow.json` (the first scan's Files page and the home's Folders page, node ids and
  times stripped).
- The flow home's cleaner had only the sandbox's places protection, so `home/Downloads` was offered. The shared
  `Extras` gained `protection`, joined to the sandbox's places; the Largest home joins this Mac's real protection of
  the sandbox's home (`fresher_macos::protection`, as sandbox mode does), which makes the home and its standard folders
  whole. The temporary fixture lies in the user's `T` below `/private/var/folders`, which the protection refuses
  whole, so the flow gives that folder as the user's own temporary folder and it is carved out, as the app does.
- A link and the inside of a claimed folder are no node of the tree: nothing there can be picked, which the flows
  assert instead of a refused pick.
- `Items::of` takes any `ItemRule`; `LargestBench::build` (the engine's rule) and `build_with` (another rule) are the
  budget test's entry points.
- The budget, `items_of_the_large_tree` in release on this Mac (2026-11-22, two runs): at the default 100 MB, 0 items in
  3 µs (the home is below the minimum: the tree's files are sparse); at no minimum, 0 items in 6 ms (every unclaimed
  folder read; the tree keeps no small file as a node); with every folder from depth 8 an item, the list full at
  10 000 items in 8.7 ms and 3 248 576 heap bytes. All under 50 ms and 5 MB.
