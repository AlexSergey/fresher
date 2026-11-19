# 020/01/03 — An app's change time and the install records

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 ·
**Modules:** core, system, engine, testing, modules, e2e, app, docs

An installer is proven an installed app's only when the app was copied in, replaced or updated after the installer
was downloaded or opened (the bundle's status change), or when the system's installer receipt names it; an old macOS
installer only when the running macOS has passed it. None of these reaches a module today. The design:
[`../00-design.md`](../00-design.md), section 1.3 (agent's calls 2, 3 and 7).

## Files

- `crates/core/src/system/apps.rs` — `InstalledApp` gains `pub changed: Option<Timestamp>`, documented as in the
  design.
- `crates/core/src/system/installs.rs` (new) — `pub struct Receipt { pub package_file: String, pub installed:
  Timestamp }`, `pub struct SystemVersion { pub major: u16, pub minor: u16 }` (`Clone, Copy, Debug, PartialEq, Eq,
  PartialOrd, Ord`, so versions compare as `(major, minor)`) with `SystemVersion::parse(text) -> Option<Self>`
  (`"15"`, `"15.4"`, `"10.15"`; a third part is ignored, anything else is `None`), and `pub trait InstallRecords:
  Send + Sync { fn receipts(&self) -> Result<Vec<Receipt>, SystemError>; fn system_version(&self) ->
  Result<SystemVersion, SystemError>; }`. `crates/core/src/system/mod.rs` and `lib.rs` re-export them.
- `crates/core/src/system/facts.rs` — `SystemPorts` gains `pub installs: Arc<dyn InstallRecords>`; `SystemFacts`
  gains `receipts: OnceLock<Result<Vec<Receipt>, SystemError>>` and `version: OnceLock<Result<SystemVersion,
  SystemError>>`, and `pub fn receipts(&self) -> Result<&[Receipt], SystemError>` and `pub fn system_version(&self) ->
  Result<SystemVersion, SystemError>`, each asked at most once per scan (doc as the other facts). The `Nothing` and
  `Counting` test ports answer no receipts and `15.0`.
- `crates/system/src/apps.rs` — `MacInstalledApps` fills `changed` from `fs::symlink_metadata(bundle)` and
  `std::os::unix::fs::MetadataExt::{ctime, ctime_nsec}` (whole seconds, as the walk's `Timestamp`); `None` when the
  metadata cannot be read (the app is still listed).
- `crates/system/src/installs.rs` (new) — `pub struct MacInstallRecords { root: PathBuf }`, `new(root)`:
  - `receipts`: the entries of `<root>/var/db/receipts` (`RECEIPTS`, a relative path constant) ending `.plist`, at
    most `MAX_RECEIPTS = 10_000` (more → an error: something is wrong with the folder), sorted by name; each read with
    `bundle::read_capped` and `plist::read_strings`, its `PackageFileName`; `installed` its modification time. A
    receipt that cannot be read, has no `PackageFileName`, or no time, is skipped (logged with its file name only).
    The folder unreadable or missing → an error ("the installer receipts cannot be read").
  - `system_version`: `macos_version(&root)` mapped to `SystemVersion`; `None` → an error.
  `lib.rs` re-exports `MacInstallRecords`.
- `crates/testing/src/system_fakes.rs` — `FakeInstallRecords` (`Default`: no receipts and a failing version, "no
  version stated"; `with_receipts(Vec<Receipt>)`, `with_version(major, minor)`, `failing_receipts(reason)`,
  `calls()`); `fake_system(home)` gains `installs: Arc::new(FakeInstallRecords::default())`; `installed_app(…)` keeps
  `changed: None`, and a helper `changed_at(app: InstalledApp, seconds: u64) -> InstalledApp`.
- `crates/testing/src/run_module.rs` — `Ports` gains `pub installs: Arc<FakeInstallRecords>` (the default above),
  handed to `SystemPorts`.
- Every other `SystemPorts {` site gains `installs` (a `FakeInstallRecords::default()` in tests):
  `crates/core/src/system/facts.rs` (tests),
  `crates/e2e/tests/{places.rs,support/home_engine.rs,jetbrains_home/mod.rs}`,
  `crates/engine/src/{facts_tests.rs,storage/attribution_tests.rs,storage/meaning_tests.rs}`,
  `modules/{temp,app-leftovers,jetbrains}/src/tests.rs`; `app/tauri/src/compose.rs` gets
  `Arc::new(MacInstallRecords::new(PathBuf::from("/")))` (02/02 makes it a field of `Shared`).
- Every `InstalledApp {` literal gains `changed: None` (26 sites: `crates/core/src/system/{facts,apps}.rs`,
  `crates/testing/src/system_fakes.rs`, `crates/system/src/apps.rs`, `crates/e2e/tests/{app_leftovers_home,
  jetbrains_home}/mod.rs`, `crates/engine/src/storage/{meaning,attribution}_tests.rs`, `modules/temp/tests/support/
  mod.rs`, `modules/app-leftovers/{tests/support/mod.rs,src/tests.rs,src/decide/tests.rs}`, `modules/jetbrains/
  {tests/support/mod.rs,src/decide.rs}`).
- `docs/architecture/modules.md` — "The system's facts": `receipts()`, `system_version()` and `InstalledApp.changed`
  (what each means, asked once per scan). `docs/architecture/README.md` — "Ports": `InstallRecords`.

## Produces

`InstalledApp.changed`, `Receipt`, `SystemVersion`, `InstallRecords`, `SystemPorts.installs`,
`SystemFacts::{receipts, system_version}`, `MacInstallRecords`, `FakeInstallRecords`, `changed_at`, `Ports.installs`,
for 02/02 (the world's installs), 03/02 (the decision), 04/02 (the e2e harness).

## Checks

- `installs.rs` (core), negatives first: `a_version_that_is_not_numbers_is_refused` (`""`, `"x"`, `"15."`,
  `"15.x"`); then `versions_parse_and_compare_as_major_then_minor` (`"10.15" < "11.0" < "15.4" < "26.0"`, `"15"` is
  `15.0`, `"15.4.1"` is `15.4`).
- `facts.rs`: `receipts_and_the_version_are_asked_once_per_scan`; an error is kept and answered again.
- `installs.rs` (system), on temporary roots made by the test (never `/`), negatives first:
  `a_missing_receipts_folder_is_an_error`, `a_receipt_without_a_package_name_or_unreadable_is_skipped` (a plist with
  only `PackageIdentifier`, a file that is not a plist), `more_receipts_than_the_limit_is_an_error` (with the limit
  lowered for the test through a `pub(crate)` constructor), `a_missing_system_version_is_an_error`; then
  `receipts_are_read_with_their_package_name_and_time` (an XML and a binary property list written by the test's
  fixture helpers, their times set with `File::set_modified`), `the_system_version_is_read` (the existing
  `macos_version::fixture::system_version` helper).
- `apps.rs` (system): `an_installed_app_carries_its_bundles_status_change` (a bundle made by the existing fixture
  helper; `changed` is within a few seconds of the test's start).
- Every crate whose sites changed builds and passes: `cargo nextest run -p fresher-core -p fresher-system -p
  fresher-testing -p fresher-engine -p fresher-module-jetbrains -p fresher-module-app-leftovers -p
  fresher-module-temp -p fresher-e2e -p fresher-app`; clippy; fmt; the crate guard (`#![forbid(unsafe_code)]`
  everywhere touched).
- To verify by hand (05/02), with this step's probe: `stat -f "%Sc" /Applications/<an app>.app` moves when an app is
  updated by its own updater (an app updated during the plan), and `plutil -p /var/db/receipts/<id>.plist` keeps its
  `PackageFileName` (read on this Mac on 2026-11-19: every receipt has it; its modification time is within 5 s of
  `InstallDate` on 8 of 8).

## Conclusions

Not written yet.
