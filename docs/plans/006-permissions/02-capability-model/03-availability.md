# 006/02/03 — Availability, tools and opt-ins

**Status:** ✔️ done · **Version:** 0.4.0 · **Date:** 2026-11-16 · **Modules:** core, engine, app

## Files

- `crates/engine/src/access.rs` (new) — the engine's view of access: the last `AccessStatus` of the probe merged with
  the tool states; `refresh(&self) -> bool` (probes again, `true` when anything changed); the tool checks — for every
  `with_tool_check` of the registered modules, run through `CommandRunner`: status 0 → `Granted`, `NotInstalled` →
  `Denied` with `core.access.tool.not-installed`, any other answer → `Denied` with `core.access.tool.not-ready`.
- `crates/engine/src/engine.rs` — `Ports` gains `system_ui: Arc<dyn SystemUi>`. New methods: `access_status()` (the
  merged view), `refresh_access()` (OS permissions only; sends `AccessChanged` when something changed),
  `open_access_settings(&Capability)` (refused with `CommandError::NoSettingsPane { capability }` when the capability
  is `NotApplicable`, and for a `Tool`, which has no settings pane). `modules()` fills each category's `Availability`.
- `crates/engine/src/session.rs` — at scan start: the tool checks run once; a module whose every category is
  `Unavailable` is not run and ends `ModuleFinished { Unavailable { missing } }`; a `Limited` category's totals are
  `Partial`.
- `crates/core/src/settings/core_schema.rs` — the switches `core.access.desktop`, `core.access.documents`,
  `core.access.downloads`, `core.access.other-apps-data`, off by default, with their label and description texts
  ("Let Fresher look in Desktop" — "macOS will ask you once. Fresher reads names and sizes only, and removes nothing
  unless you confirm it."). Plan 005/01 owns this file's catalogue: the four keys are added in a separate commit that
  005 rebases over.
- `crates/core/src/command.rs` — `ModuleInfo.categories` carries `Availability` next to each descriptor
  (`CategoryInfo { descriptor, availability }`).
- `app/tauri/src/adapters.rs`, `compose.rs` — a stand-in `NoSystemUi` (answers `Unsupported`) until 03/03; listed in
  `docs/hacks/README.md` with the existing access stand-in.

## Produces

`Engine::refresh_access`, `Engine::open_access_settings`, availability in `ModuleInfo`, `ModuleOutcome::Unavailable`,
and the opt-in settings — what 02/04 (coverage), 03/04 (commands, focus) and plan 04 (UI) build on.

## Checks

- Negative (engine, on `FakeAccess` and `FakeCommandRunner`): `a_module_whose_tool_is_not_installed_is_not_run`
  (its outcome names the missing tool), `a_tool_that_answers_non_zero_is_not_ready`,
  `refresh_without_a_change_sends_no_event`, `opening_settings_for_a_capability_this_system_lacks_is_refused`,
  `a_limited_category_has_partial_totals`, `the_opt_in_switches_are_off_by_default`.
- Positive: `refresh_after_a_grant_sends_access_changed`, `a_ready_tool_lets_its_module_run`,
  `open_access_settings_reaches_the_system_ui_port`, `modules_report_the_availability_of_each_category`.
- The engine's existing tests pass with the fakes as defaults.

## Conclusions

- Two commits, as the step asks: the four opt-in switches `core.access.{desktop, documents, downloads,
  other-apps-data}` (off by default, with their label and description) alone in `core_schema.rs`, so plan 005/01 can
  rebase over them; then the engine's part.
- `crates/engine/src/access.rs` holds `AccessView`: the probe's last `AccessStatus`, then one state per tool any
  registered module checks, in registration order (`NotAsked` until its first check). `Engine` gained
  `refresh_access() -> bool` (publishes `AccessChanged` on a change), `open_access_settings(&Capability)` and
  `Ports::system_ui`; `access_status()` answers the merged view; `modules()` fills `CategoryInfo { descriptor,
  availability }` (`ModuleInfo::new(descriptor, &AccessStatus)` replaces `From<&ModuleDescriptor>`, and
  `Registry::modules` takes the status).
- At scan start the scan's modules' tool checks run once, after `ScanStarted`; a change publishes `AccessChanged`. A
  module whose every category is `Unavailable` gets no interests, is not called, and ends at once with
  `ModuleFinished { Unavailable { missing } }` (the missing capabilities of its categories, once each). The results
  keep the scan's limited categories; their totals are `Partial`.
- Not spelled out by the step, taken the simplest way:
  - **An unavailable module does not make the scan `Partial`** (accepted by the user on 2026-11-16): it reports itself; a scan without Docker installed is
    still complete. A module without categories always runs.
  - Only the checks of the scan's (enabled) modules run; a disabled module's tool stays as it was.
  - `open_access_settings` maps the port's `Unsupported` to `NoSettingsPane` and `Failed` to `Internal` (logged).
  - The registry refuses a module that needs a tool without a check (`RegistryError::Descriptor` with
    `DescriptorError::MissingToolCheck`, the second half of 02/01's rule).
  - `FakeCommandRunner::exiting(command, status)` is new, for a tool that answers non-zero. The engine's test
    `FixedAccess` is gone: its `Fixture` takes a `FakeAccess`, a `FakeCommandRunner` and a `FakeSystemUi` (defaults:
    no capability, no command answers, a recording interface); `TestModuleBuilder` gained `needing` and
    `tool_check`. The e2e's own probe was replaced by `FakeAccess`.
  - The engine-level `the_opt_in_switches_are_off_by_default` checks the settings view; the core has a test of the
    same name on the catalogue.
- The app wires `NoSystemUi` (answers `Unsupported`) until 03/03; it is listed in `docs/hacks/README.md` next to the
  access stand-in, and `docs/architecture/README.md` lists the `SystemUi` port and the two fakes.
- Regenerated: the bindings (`ModuleInfo.categories: Array<CategoryInfo>`, `CategoryInfo`, `Availability`); the
  golden file did not change (it holds no module list).
