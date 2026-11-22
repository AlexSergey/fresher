# 031/01/03 — Fixtures, golden and the contract

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** modules, testing, fixtures

The module on fixture trees through `run_module` (the reference walk), its golden report, and the contract test over
the fixture runs. The app's registration is made in 02/01, not here: the app's test that every registered module has a
case file would fail until then. The design: [`../00-design.md`](../00-design.md), section 8. Read first, as the shape
to follow: `modules/app-caches/tests/{fixtures.rs,support/mod.rs,contract.rs}`, `fixtures/homes/app-caches-*.toml`
and `fixtures/golden/app-caches.json`.

## Files

All paths below are under the fixture home's `Library/Developer/`. Small files, real sizes.

- `fixtures/homes/xcode-not-claimed.toml` (new) — `Xcode/iOS DeviceSupport/Notes/Symbols/x` (no version),
  `Xcode/iOS DeviceSupport/17.0 (21A329)/Info.plist` only (no `Symbols`), `Xcode/iOS DeviceSupport/16.0 (20A362)` a
  file, a link `Xcode/iOS DeviceSupport/16.4 (20E247)` to a folder outside holding `Symbols/x`, an empty folder
  `Xcode/tvOS DeviceSupport/17.0 (21J354)/Symbols/` (logical 0), `Xcode/UserData/KeyBindings/Default.idekeybindings`
  and `Xcode/UserData/IDEPreferencesController.xcuserstate` (Xcode's settings), `Xcode/UserData/Previews/x` (no
  `Simulator Devices`), `Xcode/Archives/Loose.xcarchive/Info.plist` (no dated folder),
  `Xcode/Archives/2026-03-01/Fake.xcarchive/Products/x` (no `Info.plist`), `Xcode/Archives/2026-03-01/Notes/Info.plist`
  (no suffix), `Xcode/DerivedData/App-abcdefghijklmnopqrstuvwxyzab/info.plist` (`dev-caches`'),
  `CoreSimulator/Devices/<a UUID>/device.plist` (devices: deferred). Nothing is claimed and nothing left out.
- `fixtures/homes/xcode-kept.toml` (new) — `Xcode/iOS DeviceSupport/15.0 (19A346)/Symbols/x` with a socket inside
  (`special`); `Xcode/iOS DeviceSupport/15.1 (19B74)/Symbols/x` with a file `flags = ["user-lock"]` (`not-yours`);
  `CoreSimulator/Caches/dyld/24D70/x`; `Xcode/Archives/2026-03-01/Shop 01-03-2026, 10.15.xcarchive/Info.plist`. Run
  three times: with Xcode among the running apps (every clean candidate `app-open`), with only the Simulator running
  (the same), and with the running apps failing (`no-system` for the candidates that reach row 8). The `special` and
  `not-yours` folders give their reasons in every run.
- `fixtures/homes/xcode-offered.toml` (new) — `Xcode/iOS DeviceSupport/15.0 (19A346)/Symbols/x`, `…/16.4.1
  (20E252)/Symbols/x`, `…/iPhone15,2 17.2 (21C62)/Symbols/x` (16.4.1 and 17.2 recent, 15.0 not),
  `Xcode/watchOS DeviceSupport/Watch6,1 10.0 (21R355)/Symbols/x` (recent),
  `Xcode/UserData/Previews/Simulator Devices/device_set.plist`, `CoreSimulator/Caches/dyld/24D70/<runtime>/cache`,
  `Xcode/Archives/2026-03-01/Shop 01-03-2026, 10.15.xcarchive/{Info.plist,dSYMs/Shop.app.dSYM/x}`. No app running.
- `fixtures/golden/xcode.json` (new) — `UPDATE_GOLDEN=1`, reviewed: the findings (titles, reasons, sizes, app,
  dangers, removals), the left-outs of every run.
- `modules/xcode/tests/fixtures.rs` (new) and `tests/support/mod.rs` — the three homes through `run_module` with the
  running apps as each run states them, as `modules/app-caches/tests/fixtures.rs` passes them; the golden comparison.
- `modules/xcode/tests/contract.rs` — the fixture runs added (five runs), so every text and every declared left-out is
  used and every removal is a `Trash` or an `EmptyFolder` of a matched path.
- `modules/xcode/Cargo.toml` — `serde_json` as a dev-dependency (the golden); its used-by column in
  `docs/dependencies.md`.

## Produces

The module's fixtures, golden and contract, for 02.

## Checks

- `tests/fixtures.rs`, negatives first: `mod negative` `nothing_in_the_not_claimed_home_is_offered_or_left_out`,
  `derived_data_and_simulator_devices_are_not_this_module_s`, `the_kept_home_offers_nothing_and_leaves_out_each_with_its_reason`
  (the three runs); `mod positive` `the_offered_home_offers_every_kind_in_its_category`,
  `the_two_newest_versions_have_the_recent_reason`, `the_caches_are_emptied_and_the_rest_moved_to_the_trash`,
  `the_golden_report_matches`.
- `tests/contract.rs` passes with the fixture runs.
- `cargo nextest run -p fresher-module-xcode`; coverage of the crate at or above 90 % of lines
  (`cargo llvm-cov nextest -p fresher-module-xcode` or the project's coverage script).

## Conclusions

Built as the step says: the fixture homes, the golden report and the contract over the fixture runs with
`Catalogue::Required`. 85 tests in the crate, 99.29 % of its lines (419 of 422). The golden's `allocated` is 0 (the test
kit). The identities are `previews`, `simulator-caches` and `archives/<date>/<name>`.
