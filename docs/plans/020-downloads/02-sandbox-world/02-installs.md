# 020/02/02 — Installs and an app's change time in the world

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** sandbox, testing, app, docs

A sandbox's installers must not depend on the Mac's receipts and macOS version, and a case must be able to say "this
app was installed 10 days ago" although the generator writes every bundle now. A case states `macos = "15.4"`,
`[[case.receipt]]` and `changed` on its `[[case.app]]`; sandbox mode and the autotest take them from the world. The
design: [`../00-design.md`](../00-design.md), section 2 (agent's call 6).

## Files

- `crates/testing/src/tree_description.rs` and `crates/testing/src/app_bundle.rs` — `[[app]]` may state `changed`
  (an age), checked as a file's; `TreeDescription::stated(now)` yields it for the bundle's path (a `Stated` with only
  `changed`). `docs/testing.md`'s tree vocabulary says so.
- `crates/testing/src/stated_apps.rs` (new) — `pub struct StatedInstalledApps { inner: Arc<dyn InstalledApps>,
  stated: StatedFacts }`, `new(inner, stated)`: `list()` is the inner list with each app's `changed` replaced by the
  stated `changed` of its bundle path when one is stated; `find` and `search_complete` pass through. Re-exported from
  `lib.rs`. The lookup is the existing `StatedFacts::get(path)`.
- `crates/sandbox/src/case.rs` — `RawCase` gains `macos: Option<String>` (checked with `SystemVersion::parse`) and
  `receipt: Vec<RawReceipt>` (`[[case.receipt]]`: `package`, a plain file name ending `.pkg`; `installed`, an age);
  `Case::{macos, receipts}`; both named among the keys that are not files.
- `crates/sandbox/src/world.rs` — `World` gains `macos: Option<SystemVersion>` (two cases stating different versions →
  refused, naming both) and `receipts: Vec<(String, u64)>` (installed times counted back from `now`, in seconds);
  `world.toml` writes `macos` and `[[receipt]]` only when stated; `pub fn install_records(&self) ->
  FakeInstallRecords` (the receipts; the version when stated, else failing "the sandbox states no macOS version");
  `pub fn declares_installs(&self) -> bool`.
- `crates/sandbox/src/expected.rs` — `EXPECTED.md` names the stated macOS version and the receipts.
- `app/tauri/src/compose.rs` — `Shared` replaces `app_search` by `apps: Arc<dyn InstalledApps>` and gains `installs:
  Arc<dyn InstallRecords>`; the normal path builds `MacInstalledApps::new(known_folders, AppSearch::Spotlight(…))` and
  `MacInstallRecords::new("/")` as today.
- `app/tauri/src/sandbox.rs` — `SandboxPorts` gains `installs: Arc<dyn InstallRecords>` (`world.install_records()`);
  `compose` passes `apps: StatedInstalledApps::new(MacInstalledApps::new(…, AppSearch::Declared(…)), ports.stated
  .clone())` and the ports' `installs`; `build()` adds `Simulated::Installs` when `declares_installs`.
- `app/tauri/tests/sandboxes/harness.rs` — `fakes()` takes `installs` from the world as well.
- `docs/testing.md` — the vocabulary: `macos`, `[[case.receipt]]`, `changed` on `[[case.app]]`; `world.toml` gains
  `macos` and `[[receipt]]`. `docs/deferred/sandbox-world.md` — "Partly taken by plan 020" names the installs and an
  app's change time; its row updated.

## Produces

`macos`, `[[case.receipt]]` and `[[case.app]] changed` in the case files and `world.toml`, `World::install_records`,
`StatedInstalledApps`, `Shared.{apps, installs}`, for 04/01.

## Checks

- `tree_description.rs`, negatives first: `an_apps_changed_that_is_not_an_age_is_refused`; then
  `an_apps_changed_is_stated_for_its_bundle`.
- `stated_apps.rs`, negatives first: `an_app_without_a_stated_change_keeps_its_own`; then
  `a_stated_change_replaces_the_apps_change`, `find_and_search_pass_through`.
- `case.rs`, negatives first: `a_macos_version_that_does_not_parse_is_refused`,
  `a_receipt_that_is_not_a_plain_pkg_name_is_refused` (`a/b.pkg`, `x.dmg`, no `installed`); then
  `macos_and_receipts_parse`.
- `world.rs`, negatives first: `two_different_macos_versions_are_refused`, `a_world_without_installs_writes_neither`
  (byte for byte as before); then `installs_round_trip_through_world_toml`,
  `install_records_answer_the_stated_version_and_receipts`, `an_unstated_version_fails`.
- `app/tauri/src/sandbox/tests.rs`: `sandbox_mode_sees_an_apps_stated_change` (a sandbox with `[[case.app]] changed =
  "10d"`: the engine's installed app has `changed` 10 days before the world's `now`),
  `the_banner_lists_installs_when_the_world_states_them`.
- `cargo nextest run -p fresher-testing -p fresher-sandbox -p fresher-app`; `npm run sandbox:test` (unchanged
  goldens); clippy; fmt.

## Conclusions

`macos` and `[[case.receipt]]` go through the case, the world and `world.toml` as access does; an age is checked with
the testing crate's new `time_at_age`, which `World::of` also uses to turn a receipt's age into a moment. A world that
states no installs still answers from `World::install_records` (no receipt, a failing version): sandbox mode never
reads the Mac's receipts or version, and the normal build keeps `MacInstallRecords::new("/")`. The sandbox's `Shared.apps`
is built by `installed_apps`, a testable function of `sandbox.rs`; `FakeInstallRecords::failing_version` carries the
"no macOS version" message. `Simulated::Installs` already existed (02/01), so the banner and the UI needed no change.
