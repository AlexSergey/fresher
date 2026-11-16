# 004/05/06 — The app on the real scanner

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** app, e2e, docs

## Files

- `app/tauri/Cargo.toml` — dependencies `fresher-scanner`, `fresher-macos`.
- `app/tauri/src/compose.rs` — before the engine starts: `fresher_macos::io_policy::never_download_cloud_files()`
  (a failure stops the start: scanning without it could download cloud files) and `leave_access_times()` (a failure
  is logged). The walk is `Scanner::new(MacFolderReader, MacVolumes, home_anchor, options)` with
  `ScannerOptions::standing_avoid = places_that_prompt(home)`, added to every request's own `avoid` (the engine's
  request carries none until plan 006 feeds it from the capability state; then the standing list goes). The home
  folder is looked up here, in the composition root, and nowhere else.
- `app/tauri/src/adapters.rs` — `NoWalk` is removed.
- `docs/hacks/README.md` — "No disk walk until plan 004" removed; a new entry "Places that prompt are avoided until
  plan 006": what (the stand-in list), where (`crates/macos/src/places.rs`, `compose.rs`), why (no wall of system
  prompts on the first scan; plan 006 decides with the capability state), if cleaned up (the first scan raises a
  prompt per protected folder).
- `crates/e2e/tests/scan.rs` (new) — the engine with the real scanner (`StdReader`, so it runs anywhere) and the
  `demo` module plus a test module with interests of every pattern, on `fixtures/homes/scan-*.toml`: the events, the
  overview and the committed state's top folders against `fixtures/golden/scan.json`.
- `fixtures/homes/scan-basic.toml`, `fixtures/homes/scan-nested-claims.toml` — negative cases first in the test file:
  a look-alike folder, a link out of the root, an avoided folder; then nested claims and hard links.

## Produces

The app scanning the disk. What 07 makes incremental.

## Checks

- Negative: `an_avoided_folder_is_reported_as_not_scanned` (e2e), `the_scan_never_leaves_its_root` (e2e, a link out of
  the root), `the_app_refuses_to_start_without_the_io_policy` (a unit test of `compose` with a policy setter that
  fails).
- Positive: `a_fixture_home_scans_to_its_golden_report` (e2e), `the_overview_counts_nested_claims_once` (e2e).
- `npm run pre-push` passes; `npm run tauri dev` scans the configured roots (checked by hand: findings stream, the
  overview fills, a second scan works); the time of a full scan of the developer's home is written in the notes,
  with no path.

## Conclusions

- The app scans with `Scanner::new(MacFolderReader, MacVolumes, home_anchor(home), options)`; `fresher-app` takes
  `fresher-scanner`, `fresher-macos` and `tracing`. Before the engine starts, `compose` turns off the download of cloud
  files (a refusal stops the start: `ComposeError::IoPolicy`) and the access-time updates (a refusal is logged).
- `ScannerOptions::standing_avoid` = `places_that_prompt(home)`, added to every request's own `avoid`; the hack "No disk
  walk until plan 004" is replaced by "Places that prompt are avoided until plan 006".
- Not spelled out, taken the simplest way: the home folder comes from Tauri's `home_dir()` in `lib.rs`'s setup and is
  passed to `compose::engine(data_folder, home)`, so the tests give a temporary folder and nothing reads the real
  home; the policy setters are injected (`IoPolicy`) so a refusing one can be tested. `NoWalk` is gone from the app;
  the shell's own tests keep a `NoWalk` of their own in `testing.rs` (they drive commands, not the disk).
- `crates/e2e/tests/scan.rs`: the engine with the real scanner (portable reader, two threads, a fake clock and
  volumes), the `demo` module and a probe module with interests of every pattern, on `fixtures/homes/scan-basic.toml`
  (a look-alike, a link out of the root, an avoided `Documents`, a cache claimed whole, a dependency folder with its
  sibling, a tagged folder, an installer) and `scan-nested-claims.toml` (an item claim around whole claims, hard
  links). The golden `fixtures/golden/scan.json` holds the events but the findings batches, the findings sorted and
  without ids, the overview and the committed state's top folders and places not scanned, the home written `~`, node
  ids and folder times left out (the fixture makes folders now); it was stable over seven runs.
- `docs/architecture/README.md`: the `DiskWalk` and `DiskState` rows name `Scanner` and `TreeState`.
- Checks: `npm run pre-push` passes (692 Rust tests; Rust line coverage 97.6 %, the scanner 96.2 %, the engine 99.5 %,
  the app 90.5 %).
- **Not done here, by hand:** `npm run tauri dev` (findings stream, the overview fills, a second scan works) and the
  time of a full scan of the developer's home — the agent cannot drive the app's window, and no test may read the real
  home; both wait for the user's run, the numbers to be written here.
