# 016/03/02 — The core e2e flows

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** testing

## Files

- `fixtures/homes/jetbrains-flow.toml` (new) — WebStorm 2024.3 and 2025.3 in the three places, GoLand 2024.3 and
  2025.2, a sandbox marker at the root; files filled with data where the put back is compared byte for byte.
- `crates/e2e/tests/jetbrains.rs` (new) — the engine with the real scanner, the `jetbrains` module, the real settings
  file and the real `DiskCleaner` in the fixture's sandbox (`Reach::Sandbox`), `FakeInstalledApps`,
  `FakeRunningApps`, `FakeTrash`, `FakeClock`; the flows of the design, section 8, "Core e2e":
  1. scan → review → confirm → Trash → put back;
  2. an open IDE left out, then offered after it quits and a rescan;
  3. an update between scans;
  4. ignore the module, then the group Development, and restore (plan 015);
  5. the storage map: explained in WebStorm's line; the version in use is not in WebStorm's data, since the engine
     counts an app's data only in folders named after its bundle id (corrected 2026-11-17; deferred:
     [a module names its app's data folders](../../../deferred/module-names-app-data.md));
  6. the installed apps failing: everything left out `no-system`, the scan complete.
  7. (added 2026-11-17, the controller's ruling on 03/01) the cases a sandbox cannot express: a removed IDE that
     Spotlight finds elsewhere (`FakeInstalledApps::with_elsewhere`) keeps its data, and a restored scan never offers
     a removed IDE's data (`used-recently`).
  Negative flows first (6, then 2's first half), then the rest. The report of each flow, as plan 008's
  `cleanup.rs` does, into `fixtures/golden/jetbrains-flow.json` (`UPDATE_GOLDEN=1`, reviewed).
- `crates/e2e/Cargo.toml` — `fresher-module-jetbrains` as a dev-dependency.

## Produces

The module's whole Rust path, end to end.

## Checks

- `cargo nextest run -p fresher-e2e --test jetbrains`; stable over 20 runs (`--retries 0`, a loop by hand, written in
  the conclusions).

## Conclusions

Built: `crates/e2e/tests/jetbrains.rs` with 10 tests (negatives first: a failing system, an open IDE, Spotlight
elsewhere, a restored scan; then quit and rescan, an update between scans, Trash and put back byte for byte, ignoring
the module and the group, the storage map, a golden of all flows), `fixtures/homes/jetbrains-flow.toml` and
`fixtures/golden/jetbrains-flow.json`. 150 consecutive runs with no retries were green after the order races of the
first loops were fixed (one scanner thread, the golden drops finding ids and Trash numbering, sorted by path).
Changed from the design: the version in use is not in WebStorm's line of the storage map (the engine attributes app
data to folders named by the bundle id only); the test asserts that, and the design wording is corrected.
