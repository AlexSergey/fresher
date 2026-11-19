# 020/04/02 — The core e2e flows

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** e2e, fixtures

The module's flows on the real scanner, engine, settings, cache and journal files and the real `DiskCleaner` over a
fixture home, the access included: the first module whose every category needs a guarded folder. The design:
[`../00-design.md`](../00-design.md), section 4, "Core e2e".

## Files

- `fixtures/homes/downloads-flow.toml` (new) — a home with `Downloads/` holding one item per category and per kept
  reason of the flows (every entry with `age` and `changed`), a folder of the user's `Downloads/Project/` with
  `node_modules/` inside (to show it stays open to the walk), and a file outside Downloads (`Documents/keep.txt`).
- `fixtures/golden/downloads-flow.json` (new) — the first scan's findings and left-outs, `UPDATE_GOLDEN=1`, reviewed.
- `crates/e2e/tests/downloads.rs` (new) and its harness `crates/e2e/tests/downloads_home/mod.rs` (new), on the shared
  `support/home.rs` (as `app_leftovers_home`): the `downloads` module registered beside `jetbrains`, `temp`,
  `dev-caches` and `app-leftovers` (as the app registers them); `FakeClock`, `FakeTrash`, `FakeRunningApps`,
  `FakeInstalledApps` (Firefox 133.0 and an `Example` app with `changed_at`), `FakeInstallRecords` (a receipt, macOS
  15.4), `FakeOpenFiles`, `FakeChangeJournal`, and a `FakeAccess` whose home is the fixture's, guarding `Downloads`
  with `Capability::Folder(GuardedFolder::Downloads)`. The flows, negatives first within each:
  1. `nothing_is_read_in_downloads_before_the_opt_in` — Downloads `NotAsked`, the switch off: the module's outcome
     `Unavailable { missing: [Downloads] }`; `FakeAccess::tried()` empty; no finding or left-out of the module; the
     coverage names Downloads not entered.
  2. `the_first_scan_after_the_opt_in_offers` — `core.access.downloads` switched on, `answering("Downloads",
     Granted)`: in that same scan an `AccessChanged` with Downloads `Granted`, the module `Completed`, the golden
     matches.
  3. `a_denied_folder_keeps_the_module_unavailable` — answering `Denied`: `Unavailable`, coverage `Denied`, nothing
     offered.
  4. `scan_confirm_trash_and_put_back` — every category selected: the plan's highest danger High; every item in the
     Trash and the journal, the placeholder with its `.part`; put back restores them byte for byte; the user's folder,
     `node_modules` inside it and `Documents/keep.txt` untouched.
  5. `an_unfinished_download_goes_once_the_browser_quits` — Chrome running: `browser-open`; `FakeRunningApps` changed;
     a rescan (the journal scripted, asserted `ScanKind::Rescan`) offers it.
  6. `an_unfinished_download_goes_once_the_days_pass` — 20 days old: `recent`; `FakeClock` 15 days on; a rescan
     (asserted, nothing changed on disk) offers it.
  7. `an_installer_goes_once_the_app_is_updated` — `Example-2.0.dmg` newer than `Example`'s change: `not-older`;
     `FakeInstalledApps` answers `Example` 2.0 changed after it; a rescan (asserted) offers it, for the app.
  8. `the_storage_map_counts_an_installer_in_its_apps_line` — the offered Firefox installer in Firefox's line of the
     by-meaning view; the browse view opens `Downloads/Project` (its children listed, not one opaque entry).
  9. `ignoring_the_big_files_keeps_them_out_of_every_total` — `downloads.large-untouched` ignored: its finding in no
     total, the others stay; restore brings it back.
- `crates/e2e/Cargo.toml` — `fresher-module-downloads` as a dev-dependency.
- `crates/e2e/tests/support/home_engine.rs` — `Parts` gains `access` and `installs`; the temp, dev-caches and
  app-leftovers homes give the old values (`unknown_access`, `no_installs`).
- `crates/testing/src/fake_access.rs` — a try that answers `Granted` or `Denied` sets the capability's state, as the
  system's probe remembers it.

## Produces

The flows, for the acceptance's "nothing in Downloads is read without access" and "the first scan after the opt-in".

## Checks

- `cargo nextest run -p fresher-e2e --test downloads` (the nine flows); the other e2e binaries unchanged
  (`cargo nextest run -p fresher-e2e`).
- `npm run test:e2e:core`; clippy (`--all-targets`); fmt.

## Conclusions

- The nine flows and a tenth (`full_disk_access_needs_no_try`) pass; the golden `fixtures/golden/downloads-flow.json`
  holds flow 2's offers and left-outs. The fixture's files are under `home/` (as the app-leftovers home), the scan root
  is set to the sandbox's home so the fake's guarded place lies inside it.
- `FakeAccess` did not remember a try, so the cleanup's access check (it probes again) saw Downloads `NotAsked` and left
  every item out `MissingCapability`; it now remembers `Granted` and `Denied` as the real probe does.
- With Full Disk Access granted the real probe reports the folder `Granted` too; the flow states both.
- Flow 6 moves the clock 12 days, not 15: past 14 days the scanner trusts no rescan (a full scan ran).
- Files smaller than the scanner keeps are not rows of the browse view: `Project` lists `node_modules` and counts its
  two files.
