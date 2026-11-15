# 002/05/05 — End to end on the demo module

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** e2e

## Files

- `crates/e2e/Cargo.toml` — dev-dependencies `fresher-engine`, `fresher-store`, `fresher-module-demo`,
  `fresher-testing`.
- `crates/e2e/tests/demo.rs` — a temporary data folder with `FileSettingsStore`, the engine with the `demo` module, a
  walk that finds nothing, `FakeClock`: start a scan, collect every event until `ScanFinished`, then the overview;
  compare with `fixtures/golden/demo.json`. Then change `demo.count`, build a new engine on the same folder and check the
  value survived ("a restart").
- `fixtures/golden/demo.json` — the expected events and overview.

## Produces

Goal 6 of the design proven on the Rust side: command → engine → module → events, with the settings file.

## Checks

- Negative: `a_second_scan_during_the_first_is_busy`, `a_settings_file_from_a_newer_version_is_not_overwritten`.
- Positive: `the_demo_scan_matches_its_golden_report`, `a_setting_survives_a_restart`.
- `npm run test:e2e:core` passes; plan 05's conclusions written.

## Notes from the implementation

- `crates/e2e/tests/demo.rs` builds the engine as the app does, with the real `FileSettingsStore` on a temporary data
  folder, the `demo` module, `FakeClock`, `FakeCommandRunner`, a walk that finds nothing and an access probe that
  answers `Unknown` (both written in the test: the app's adapters are private to `fresher-app`). The busy check uses a
  walk that holds the scan until it is cancelled.
- `fixtures/golden/demo.json` is `{ "events", "overview" }` (keys sorted, as `serde_json::json!` writes them): the
  scan's events up to `ScanFinished` (one batch of the eight default findings, flushed when the module finishes) and
  the overview after it.
- "A restart" is a second engine on the same folder after the first is dropped: `demo.count` = 3 is read back from
  `settings.json`, the file is `loaded`, and the next scan finds 3 findings.
- A newer settings file (version 2) is refused with `SettingsReadOnly`, its state is `newer-version`, and its bytes are
  unchanged.
- The golden helpers moved to `crates/e2e/tests/support/mod.rs`, shared with `smoke.rs`; `check_golden` takes any
  `Serialize` value. Dev-dependencies: `fresher-core`, `fresher-engine`, `fresher-store`, `fresher-module-demo`,
  `fresher-testing`, `serde`, `serde_json`, `tempfile`.
