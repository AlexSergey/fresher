# 031/02/02 — The core e2e flows

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** e2e, fixtures, docs

The module through the whole engine on a fixture home, beside `dev-caches` (the other module of
`~/Library/Developer`): scan, left-outs, cleanup to the Trash and emptied folders, put back, a rescan after Xcode quits,
a restored scan. The design: [`../00-design.md`](../00-design.md), sections 7 and 8. Read first, as the shape to
follow: `crates/e2e/tests/app_caches.rs`, `crates/e2e/tests/app_caches_home/mod.rs`, `crates/e2e/tests/dev_caches.rs`,
`crates/e2e/tests/support/home_engine.rs`, `fixtures/homes/app-caches-flow.toml` and `fixtures/golden/app-caches-flow.json`.

## Files

- `fixtures/homes/xcode-flow.toml` (new) — under `Library/Developer/`: `Xcode/iOS DeviceSupport/15.0 (19A346)/Symbols/x`,
  `…/16.4.1 (20E252)/Symbols/x`, `…/17.2 (21C62)/Symbols/x`, `Xcode/UserData/Previews/Simulator
  Devices/device_set.plist`, `CoreSimulator/Caches/dyld/24D70/runtime/cache` and `…/runtime/cache.map`,
  `Xcode/Archives/2026-03-01/Shop 01-03-2026, 10.15.xcarchive/{Info.plist,dSYMs/x}`, and a DerivedData project
  `Xcode/DerivedData/App-abcdefghijklmnopqrstuvwxyzab/{info.plist,Build/o}` (`dev-caches`'). Small files with data. Xcode is installed (`com.apple.dt.Xcode`, given the way `app_caches_home`
  gives its installed apps), so the map's line is Xcode's.
- `fixtures/golden/xcode-flow.json` (new) — `UPDATE_GOLDEN=1`, reviewed: the scan's findings of both modules and the
  storage map's Xcode line.
- `crates/e2e/tests/xcode.rs` (new) and `crates/e2e/tests/xcode_home/mod.rs` (new) — the engine with the modules
  `xcode` and `dev-caches` registered, over the fixture home, with `FakeRunningApps` scripted per scan and the
  sandbox's Trash (as `app_caches.rs`).
- `crates/e2e/Cargo.toml` — `fresher-module-xcode` as a dev-dependency.
- `docs/testing.md` — one paragraph beside the `tests/app_caches.rs` one: what `tests/xcode.rs` runs and proves.

## Produces

The module's flows through the engine, for 03.

## Checks

The tests of `crates/e2e/tests/xcode.rs`, negatives first:

- `mod negative`
  - `no_folder_is_offered_by_both_xcode_and_dev_caches` — the DerivedData project is offered by `dev-caches` only;
    every `xcode` finding lies outside `Xcode/DerivedData`.
  - `everything_waits_while_xcode_is_open_until_a_rescan_after_it_quits` — scan with Xcode running: no `xcode`
    finding, the six candidates left out `app-open`; rescan with nothing running: all six offered.
  - `everything_waits_while_the_simulator_is_open` — the same with `com.apple.iphonesimulator` running.
  - `an_entry_held_open_skips_the_emptied_cache_s_entry` — an open file inside `CoreSimulator/Caches` (`FakeOpenFiles`)
    keeps that entry in place; the item ends `Emptied` with one skipped entry and the other entries in the Trash.
- `mod positive`
  - `a_cleanup_moves_device_support_and_the_archive_and_empties_the_caches` — select every `xcode` finding, confirm to
    the Trash: three device support folders and the archive are in the sandbox's Trash, `Previews` and `Caches` still
    exist and are empty.
  - `put_back_restores_the_archive_byte_for_byte` — after the cleanup, put back the archive: every file as it was.
  - `put_back_restores_the_emptied_caches_beside_a_file_written_since` — a file written into `CoreSimulator/Caches`
    after the cleanup stays; put back brings every entry back.
  - `a_restored_scan_offers_the_same_while_xcode_is_closed_and_leaves_out_while_it_is_open`.
  - `the_storage_map_counts_every_finding_in_xcode_s_line` — the golden `fixtures/golden/xcode-flow.json`.
- `cargo nextest run -p fresher-e2e --test xcode`; clippy; fmt.

## Conclusions

Built as the step says: 9 flows in `crates/e2e/tests/xcode.rs` and the golden `xcode-flow.json`. One flow differs from
the brief: a file held open inside `CoreSimulator/Caches` does not leave the item `Emptied` with one skipped entry. The
cleaner's in-use check is per item, so one open file anywhere inside skips the whole item (`Skipped`, `InUse`) before
anything is moved, and the other entries stay in place; this is how `app-caches` behaves too (its flow 4). The flow is
named `a_file_held_open_skips_the_simulator_caches_in_use_and_the_rest_go`: the caches item is skipped, while the archive
and the three device support folders (moved) and the previews (emptied) go. This is the safer direction; the core is not
changed. The step's wording above ("keeps that entry in place; the item ends `Emptied`") describes the design before
this finding.
