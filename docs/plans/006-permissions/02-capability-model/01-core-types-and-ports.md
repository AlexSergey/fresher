# 006/02/01 — Core types and ports

**Status:** ✔️ done · **Version:** 0.4.0 · **Date:** 2026-11-16 · **Modules:** core

## Files

- `crates/core/src/access.rs` (new) — `Capability` (`AllFiles`, `Folder(GuardedFolder)`, `OtherAppsData`,
  `Volume(VolumeKind)`, `Tool(ToolId)`), `GuardedFolder` (`Desktop`, `Documents`, `Downloads`), `VolumeKind`
  (`Removable`, `Network`), `Access` (`Granted`, `Denied`, `NotAsked`, `Unknown`, `NotApplicable`), `CapabilityState
  { capability, access, note: Option<Text> }`, `AccessStatus { capabilities }` with `access(&Capability) -> Access`
  (a capability not listed is `NotApplicable`) and `check(&[Need]) -> Result<(), Vec<Capability>>`, `GuardedPlace
  { path, capability }`, `Need { capability, kind }` with `Need::required` / `Need::sees_more`, `NeedKind`
  (`Required`, `SeesMore`), `Availability` (`Ready`, `Limited { missing }`, `Unavailable { missing }`) and the pure
  `Availability::of(&[Need], &AccessStatus)`.
- `crates/core/src/ports.rs` — `AccessProbe` extended with `guarded_places(&self, roots: &[PathBuf]) ->
  Vec<GuardedPlace>` and `try_place(&self, &GuardedPlace) -> Access` (documented: may prompt, called only inside a
  user-started scan for an opted-in place); `AccessStatus` and `Access` move to `access.rs`. New port `SystemUi`
  (`open_access_settings(&Capability)`, `reveal(&Path)`) and `SystemUiError` (`thiserror`: `Unsupported`,
  `Failed { reason }`).
- `crates/core/src/walk.rs` — `Unreadable::NeedsFullDiskAccess` renamed `Unreadable::Guarded` (doc: refused by the
  system's privacy protection), `NoPermission` documented as POSIX permissions. If plan 004 has changed this file,
  the rename goes on top of its version.
- `crates/core/src/category.rs` — `CategoryDescriptor::with_needs(Vec<Need>)` and `needs()`; serialized with the
  descriptor, so `ModuleInfo` carries them.
- `crates/core/src/module.rs` — `ModuleDescriptor::with_tool_check(ToolCommand)` and `tool_checks()`; refused when the
  tool is not named by a `Tool` need of one of its categories (`DescriptorError::UnusedToolCheck`), and a `Tool` need
  without its check is refused too (`DescriptorError::MissingToolCheck`).
- `crates/core/src/event.rs` — `ModuleOutcome::Unavailable { missing: Vec<Capability> }`.
- `crates/core/src/error.rs` — `CommandError::MissingCapability { capability }` and
  `CommandError::NoSettingsPane { capability }`; their texts `core.error.missing-capability` and
  `core.error.no-settings-pane`.
- `crates/core/src/lib.rs` — `mod access;` and its exports.
- `crates/core/locales/en.json` — per capability: `core.access.<id>.name`, `.unlocks`, `.how` (numbered steps),
  `.missing` (the one sentence a disabled section shows); per state: `core.access.state.<state>`; the tool notes
  `core.access.tool.not-installed` and `.not-ready` (with `{tool}`); `core.module.unavailable`. Every text plain
  English, rule 7 of [`../../../rules.md`](../../../rules.md).

## Produces

Every type and port of the design, sections 2 and 3, for the fakes (02/02), the engine (02/03), the macOS adapter
(plan 03), the UI (plan 04) and plan 008. The `AccessStatus` shape in the bindings changes here; the app's
`UnknownAccess` stand-in is updated to answer `Unknown` for `AllFiles` and `NotAsked` for the folders.

## Checks

- Negative: `a_capability_that_is_not_listed_is_not_applicable`, `a_required_need_that_is_denied_makes_the_category_unavailable`,
  `a_tool_need_without_its_check_is_refused`, `a_tool_check_without_a_need_is_refused`,
  `check_reports_every_missing_capability`.
- Positive: `a_sees_more_need_that_is_not_granted_limits_the_category`, `granted_needs_make_the_category_ready`,
  `capabilities_serialize_with_kebab_case_tags` (`{ "kind": "folder", "value": "desktop" }`),
  `exports_the_access_types_to_typescript`, `every_capability_has_its_texts` (the core dictionary test).
- `cargo clippy --workspace --all-targets -- -D warnings` and the bindings test pass after regenerating
  `bindings.generated.ts` (`UPDATE_BINDINGS=1`).

## Conclusions

- The types and ports are in place as written: `crates/core/src/access.rs` (`Capability`, `GuardedFolder`,
  `VolumeKind`, `Access`, `CapabilityState`, `AccessStatus` with `access` and `check`, `GuardedPlace`, `Need`,
  `NeedKind`, `Availability::of`), `AccessProbe::{guarded_places, try_place}`, `SystemUi` and `SystemUiError`,
  `CategoryDescriptor::{with_needs, needs}`, `ModuleDescriptor::{with_tool_check, tool_checks}`,
  `ModuleOutcome::Unavailable`, `CommandError::{MissingCapability, NoSettingsPane}` and the texts.
- Already done by plan 004/02 and not redone: the rename `Unreadable::NeedsFullDiskAccess` → `Unreadable::Guarded`
  and the `EACCES` documentation of `NoPermission`.
- Not spelled out by the step, taken the simplest way:
  - **What counts as allowed**: `Granted` and `NotApplicable` (nothing guards what the capability would unlock);
    `Denied`, `NotAsked` and `Unknown` are missing (`Access::allows`). `Availability::Unavailable` lists the missing
    required capabilities only; `Limited` the missing `SeesMore` ones. `check` looks at every need, whatever its
    kind, and names each missing capability once, in order.
  - **The tool-check rule in two halves**: `with_tool_check` refuses a check no category needs (`UnusedToolCheck`)
    and a second check of the same tool (`DuplicateToolCheck`, new, like the other duplicates); a needed tool with no
    check is refused by `ModuleDescriptor::check_tools()`, which the registry calls (02/03) — a descriptor cannot
    refuse it while it is built, before its checks are added.
  - `CategoryDescriptor` is no longer `Copy` (it holds its needs); `fresher-rules` clones it.
  - Texts: `Capability::SYSTEM` lists the seven capabilities the system grants; each has `core.access.<id>.{name,
    unlocks, how, missing}` with ids `all-files`, `desktop`, `documents`, `downloads`, `other-apps-data`,
    `removable-volume`, `network-volume` (`Capability::text_key`; a tool has none). The tool notes are built by
    `CapabilityState::tool_not_installed` / `tool_not_ready`, so their keys are the core's. `core.module.unavailable`
    is `ModuleOutcome::unavailable_key()`. The two new errors take no text argument.
  - **`Capability` is also `Deserialize`** (for the command of 03/04, which takes one): a hand-written impl reads the
    system's capabilities and refuses a `tool`, which only the modules name (a `ToolId` is a `&'static str`).
- The app's `UnknownAccess` answers `Unknown` for `AllFiles` and `NotAsked` for the other six, knows no guarded place
  and answers `Unknown` to `try_place`; its hack entry is updated. The engine's and the e2e's test probes got the two
  new methods.
- Regenerated: the bindings (`AccessStatus`, `CapabilityState`, `Capability`, `GuardedFolder`, `VolumeKind`,
  `Access`, `Need`, `NeedKind`; `CategoryDescriptor.needs`, `ModuleOutcome.unavailable`, the two errors). The UI's
  two test answers of `AccessStatus` use the new shape.
- Final review: `CapabilityText` gains `Denied` (`core.access.<id>.denied`): where in the system's settings to turn a
  refused capability on, shown by the notices and the Permissions group of 04.
