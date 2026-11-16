# 008/02 — Contracts

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** core, testing, macos, engine, app

Every type, port and text of the design in `fresher-core`, with their fakes in `fresher-testing`, so plans 03, 04, 05
and 06 can run in parallel against a frozen core. No IO in the core; nothing here removes anything. The design:
[`../00-design.md`](../00-design.md), sections 2, 5, 7, 8 and 10.

After this plan the core is frozen for the rest of 008: a missing contract stops the plan that needs it, and the change
goes into `crates/core` in the main session.

## Steps

| #   | Step                                                                 | Status   | Shared files                                                                                                                                                   |
| --- | -------------------------------------------------------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01  | [The plan and its confirmation](./01-plan-and-confirmation.md)       | ✔️ done | root `Cargo.toml`, `crates/core/{Cargo.toml,src/lib.rs}`, `Cargo.lock`                                                                                         |
| 02  | [Outcomes, events, commands, errors](./02-outcomes-events-errors.md) | ✔️ done | `crates/core/src/{event,command,error,texts,lib}.rs`, `crates/core/locales/en.json`, `crates/engine/`, `app/tauri/tests/bindings.rs`, `bindings.generated.ts`  |
| 03  | [Protection](./03-protection.md)                                     | ✔️ done | `crates/core/src/lib.rs`                                                                                                                                       |
| 04  | [Ports and fakes](./04-ports-and-fakes.md)                           | ✔️ done | `crates/core/src/{ports,volume,lib}.rs`, `crates/macos/src/volume.rs`, `crates/testing/src/`, `crates/engine/`, `crates/scanner/`, `app/tauri/src/adapters.rs` |

In order: each step builds on the types of the one before (03 needs only 01 and may run beside 02).

## Conclusions

The four steps are built as written, with the changes each step's conclusions give; `npm run pre-push` passes on the
last commit. The main ones: the `Trash` port takes the held handle and answers `Landed` (call 37, from plan 01); the
sandbox marker is opt-in (`FixtureTree::with_sandbox`), so no scan test and no golden file changed; the ignored paths
get their own answer (`Protected::Ignored`); and contracts later plans need were added now, since the core is frozen
(`QuitRequested`, the commands' inputs, `NotInTrash`, `BlockedReason::DiskFull`, `Failure`, `SkipCode`,
`RecordedOutcome`). Dependencies: `trybuild` 1.0.125 (dev, `core`); `objc2` 0.6.5 and `objc2-foundation` 0.3.2 are
pinned in the root for plan 03.

**How each guarantee is held**:

| Guarantee                                              | Held by                                                                                                          |
| ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------- |
| only a confirmation makes a `ConfirmedPlan`            | private fields, `CleanupPlan::confirm` its only constructor (`confirmed_plan_literal`)                           |
| a `Confirmation` exists only as the UI's JSON          | private fields, no constructor, no `From`, no `Default` (`confirmation_literal`, `confirmation_from_plan`)        |
| the echo must match                                    | `confirm` compares the plan id, count, allocated bytes, highest level; property `any_changed_field_…_refuses`    |
| an old plan cannot be confirmed                        | `PLAN_LIFETIME` (600 s) against the injected time; a time before the plan refuses too                            |
| a confirmed plan runs once                             | not `Clone`, consumed by `Cleaner::execute` (`confirmed_plan_clone`, `confirmed_plan_used_twice`)                |
| an unconfirmed plan cannot be executed                 | `execute` takes a `ConfirmedPlan` (`unconfirmed_plan_execute`)                                                   |
| no act without a confirmed plan                        | `Act` has a private field and is lent only by `ConfirmedPlan::act` (`act_literal`)                               |
| permanent needs its own consent                        | `PermanentConsent(())` made only by `confirm` for `Method::Permanent` (`permanent_consent_literal`)              |
| a summary that disagrees with the items cannot be made | `CleanupPlan::new` computes it; it refuses more than `MAX_PLAN_ITEMS`, a finding twice, an overflow              |
| protected places, by components                        | `Protection::check`, with a property test against a plain-loop model and one that no ancestor passes             |
| no path from the UI                                    | the inputs take ids only, `deny_unknown_fields`; `no_view_holds_a_path_but_the_display_one`                       |

`Verified` (not `Clone`, consumed by each removal) is the cleaner's (plan 04), with its own compile-fail cases.

**The frozen contracts** (`fresher-core`, exported from the crate root), which plans 03 to 07 build on:

- Plan and confirmation (`cleanup/plan.rs`, `cleanup/confirm.rs`): `PlanId`, `CleanupId`, `CleanupSelection`,
  `CategoryPick`, `Method`, `ItemIdentity`, `Action`, `ActionKind`, `PlanItem` (`new(&Finding)`, `with_identity`),
  `LeftOut`, `PlanSummary` (`of`), `CleanupPlan` (`new`, `confirm`), `PlanError`, `MAX_PLAN_ITEMS`, `PLAN_LIFETIME`,
  `Confirmation`, `ConfirmError`, `EchoField`, `ConfirmedPlan` (`items`, `method`, `permanent`, `act`, `plan`),
  `ConfirmedMethod`, `PermanentConsent`, `Act`.
- Outcomes and texts (`cleanup/outcome.rs`): `SkipReason` (`code`, `text`, `how`), `SkipCode`, `Failure`,
  `ItemOutcome` (`text`, `recorded`), `RecordedOutcome`, `ItemReport`, `Ended`, `OutcomeCounts`, `CleanupReport`
  (`new`, `count`, `count_recorded`), `TrashState`, `PutBackOutcome`, `BlockedReason`.
- The IPC views and inputs (`cleanup/view.rs`): `CleanupPlanView`, `PlanGroup`, `PlanItemView`, `CleanupsQuery`,
  `CleanupsPage`, `CleanupSummary`, `CleanupDetail`, `CleanupItemView`, `PutBackRequest`, `CleanupItemRef`,
  `PutBackReport`, `PutBackItem`; the events `CleanupStarted`, `CleanupItems`, `CleanupFinished`, `FindingsRemoved`,
  `QuitRequested`; `EngineStatus::Cleaning { cleanup }`; `LastScan.changedSince`; the errors `UnknownCategory`,
  `NothingToClean`, `TooManyItems`, `PlanGone`, `PlanMismatch`, `CleaningBlocked`, `UnknownCleanup`,
  `UnknownCleanupItem`, `NotInTrash`; every key of `core.cleanup.*` and the new `core.error.*` in `locales/en.json`.
- Protection (`cleanup/protection.rs`): `Protection` (`new`, `with_ignored`, `check`), `Protected`, `ProtectionError`.
- Ports: `Trash` (`move_to_trash(path, held) -> Landed`, `put_back`), `Landed`, `TrashError`; `OpenFiles`, `OpenSet`,
  `OpenFilesError`; `Volumes::available`; `Cleaner` (`rehearse`, `execute`, `locate`, `put_back`), `CleanupContext`,
  `Depth`, `Rehearsal`, `ItemDone`, `TrashedItem`; `CleanupJournal`, `JournalState`, `JournalProblem`,
  `JournalError`, `BeginRecord`, `JournalItem`, `VolumeSpace`, `ItemRecord`, `EndRecord`, `PutBackRecord`,
  `RecordedCleanup` (`outcome`, `summary`), `JournalPage`; `SANDBOX_MARKER`, `SANDBOX_SIGNATURE`.
- Fakes (`fresher-testing`): `FakeTrash` (`without_trash`, `reporting_nothing`, `refusing`),
  `FakeVolumes::answering_available`, `FakeOpenFiles`, `MemoryCleanupJournal`, `FakeCleaner` with `ExecutedPlan`,
  `FixtureTree::{with_sandbox, mark_sandbox}`.
