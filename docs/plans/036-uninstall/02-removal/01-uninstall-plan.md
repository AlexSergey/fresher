# 036/02/01 — The uninstall plan

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** engine, app

`Engine::prepare_uninstall(app, choice)`: one finding made by the engine (never in the results) that moves the bundle
and the choice's parts together, rehearsed and kept as the pending plan, confirmed with the existing
`confirm_cleanup`. Around it: the cleanup context that names the apps folder as a root and refuses it for every plan
that is not an uninstall, and the results and the apps list that drop what went. The highest-risk step of the plan:
everything it builds feeds the cleaner. The design: [`../00-design.md`](../00-design.md), sections 6 and 7.

## Before you start

Read `docs/restrictions/deletion-safety.md`, `docs/architecture/removal.md` (the flow, the guard, "Together", put
back), `crates/engine/src/cleanup/` (all of it: `pending.rs`, `plan.rs`, `context.rs`, `operation.rs`, `history.rs`,
`testing.rs`), `crates/engine/src/results.rs` (`remove`, `take_id`,
`pick`, `gone`: plan 035's), `crates/engine/src/own.rs` (`check_picks`), `crates/core/src/removal.rs` (`Parts`,
`MAX_PARTS`), `crates/core/src/cleanup/plan.rs` (`CleanupPlan::new`, `PlanItem::new`), `crates/core/src/finding.rs`,
and 01/04's `crates/engine/src/apps/`. 01/02's `Protection::with_refused` exists. The cleaner (`crates/cleaner`) is not
changed by this step: if it seems to need a change, stop and report.

## Files

- `crates/engine/src/cleanup/context.rs` — `Context::now(shared, scope: Scope)` with `pub(crate) enum Scope {
  Cleanup, Uninstall, PutBack }`: the roots as today plus the located `KnownFolder::Applications` (when the system
  locates it and no root holds it); for `Scope::Cleanup` the protection also refuses that folder
  (`with_ignored(&ignored)?.with_refused(&[apps])`, a refusal failure blocks the cleanup as an ignore-list failure does).
  Every existing caller passes its scope: `prepare` and an ordinary `confirm` → `Cleanup`; `history::put_back` →
  `PutBack`.
- `crates/engine/src/cleanup/pending.rs` — the pending plan carries `purpose: Purpose` (`Selection` or `Uninstall {
  app: BundleId, bundle: PathBuf }`); `take` answers both. The shared tail of `prepare` (blocked check, rehearsal with a
  context, `freeze`, `keep`) becomes one function both `prepare` and `prepare_uninstall` call, so the uninstall plan is
  made by the same code as every plan.
- `crates/engine/src/apps/uninstall.rs` (new) — `pub(crate) fn prepare(shared, id: AppId, choice: UninstallChoice) ->
  Result<CleanupPlanView, CommandError>`, in this order:
  1. `machine.idle()?` (`Busy`); the pending plan dies (`kill`).
  2. The app by id from the current state (`UnknownApp`), its row of the list asked again **now**: the running apps
     anew; not offered → `UninstallRefused { reason }`.
  3. The parts of `choice` from the footprint (`FootprintKind::goes_with`), kept ones left out, in the table's order;
     with the bundle more than `MAX_PARTS` paths → `UninstallRefused { reason: TooManyFiles }`.
  4. One `NewFinding`: category `UninstallChoice::category()`'s descriptor (`uninstall_categories()`), identity
     `uninstall/<bundle id>`, `Subject::Path(bundle)`, removal `MoveTogether(Parts::new([bundle, parts…]))` or
     `MoveToTrash(bundle)` when no part goes, sizes the sum (the weakest completeness, saturating), title
     `core.uninstall.title` with `app` (`TextArg::Text`), reason `choice.reason_key()` with `app` and `size`
     (`TextArg::Size` of the logical sum), `for_app(bundle id)`; published as `Finding::new(id, finding,
     bundle_facts.removability())` with `id` the results' next id (`Results::take_id`, plan 035's: answers `next_id`
     and moves it on, so no scan's finding ever has it). It is not kept with `Results::pick` (the uninstall finding is
     never in the results); `own::check_picks` looks only at `core.own-file` items, so it passes an uninstall item.
  5. The plan's roots: `results.walked_roots()` plus the bundle's apps folder when none of them holds it; the rehearsal
     (`Depth::Quick`) with `Context::now(shared, Scope::Uninstall)`; `plan::sort_out` and `freeze` as for any plan (a
     refused part leaves the whole item out with its reason; the plan then has no item and the confirmation shows why).
  6. Kept pending with `Purpose::Uninstall`.
- `crates/engine/src/cleanup/operation.rs` — `confirm` takes the pending plan with its purpose and builds the context
  with `Scope::Uninstall` for an uninstall, `Scope::Cleanup` otherwise. `Run` keeps every item's paths by index (the
  action's parts, copied from the confirmed plan before it goes to the cleaner); an item whose outcome clears (as today,
  `clears`) adds **all** its paths to `went`. `Run::end` calls `results.remove(&removed, &cleared, &went)` and, for an
  uninstall, `apps.forget(&went)` (the apps cache hides an app whose bundle path went and the parts that went).
- `crates/engine/src/results.rs` — plan 035 made `remove(&mut self, removed: &[FindingId], cleared: &[FindingId])`:
  it already drops every finding and pick at or below the path of a cleared finding (by components) and keeps the
  dropped picks' paths in `gone`. It gains `went: &[PathBuf]`, the paths of a cleared item that is no finding of the
  results (an uninstall's bundle and parts), which count as the paths of `cleared` do. Today's callers pass `&[]`; its
  tests follow. `take_id` exists.
- `crates/engine/src/apps/cache.rs` — `forget(&mut self, gone: &[PathBuf])`.
- `crates/engine/src/engine.rs` — `pub fn prepare_uninstall(&self, app: AppId, choice: UninstallChoice) ->
  Result<CleanupPlanView, CommandError>` at the end of `impl Engine`.
- `app/tauri/src/apps_commands.rs` — `prepare_uninstall(app, choice)`; registered in `lib.rs`.
- `app/ui/src/ipc/bindings.generated.ts` — regenerated.

## Produces

The uninstall plan for 02/02 (the outcome it learns from), 03/02 (the flow), 04/01 and 04/02.

## Checks

Engine tests on fixtures (`crates/engine/src/cleanup/testing.rs`, `FakeCleaner`, `MemoryCleanupJournal`, the fakes of
01/04), negatives first:

- `uninstall.rs`, `mod negative`: `prepare_uninstall_while_scanning_is_busy`, `an_unknown_app_is_refused`,
  `an_app_not_offered_is_refused_with_its_reason` (each of `installed-by`, `needs-administrator`, `unmeasured`),
  `an_app_opened_since_the_list_is_refused_open`, `more_than_64_paths_is_refused_too_many_files`,
  `a_kept_part_is_never_in_the_plan` (shared, link, needs-permission, ignored), `keep_data_never_takes_settings_or_
  support`, `a_part_the_rehearsal_refuses_leaves_the_whole_app_out`, `a_new_scan_kills_the_uninstall_plan`, `the_
  uninstall_finding_is_never_in_the_results` (`findings` and `overview` unchanged); `mod positive`:
  `keep_data_moves_the_bundle_caches_logs_saved_state_and_agents_together`, `everything_adds_settings_web_data_and_
  support`, `an_app_with_no_part_is_one_path`, `the_finding_is_medium_for_keep_data_and_high_for_everything`, `the_
  plan_s_roots_name_the_apps_folder`, `the_finding_id_is_past_every_scan_id`.
- `context.rs`, `mod negative`: `an_ordinary_cleanup_refuses_the_apps_folder` (the `Scope::Cleanup` context's
  protection answers `Never` for `/Applications/A.app` and `Ancestor` for nothing new; the real cleaner's skip is
  proven in 04/02), `put_back_adds_no_refusal`; `mod positive`: `an_uninstall_context_names_the_apps_folder_and_does_
  not_refuse_it`.
- `operation.rs`: `an_uninstall_confirms_with_the_uninstall_context`, `every_finding_below_a_part_that_went_leaves_the_
  results` (an `app-caches`-like finding under a removed `Caches/<id>`), `an_uninstalled_app_leaves_the_list`, `a_
  failed_uninstall_keeps_the_app_and_its_findings`.
- `results.rs`: the existing `remove` tests pass with `&[]`; `a_finding_below_a_path_that_went_is_dropped`.
- `cargo nextest run -p fresher-engine -p fresher-app`; clippy; fmt; `git diff -- crates/cleaner modules` empty.

## Conclusions

- Built in `crates/engine/src/apps/uninstall.rs`, `cleanup/{context,pending,operation}.rs`, `results.rs`: the one
  engine-made finding of `MoveTogether([bundle, parts...])`, the context's `Scope` (`Cleanup`, `Uninstall`, `PutBack`)
  with the second wall, the plan judged again at confirm, the paths that went. Engine, app and neighbours 2319 tests;
  the compose-level and e2e tests are mutation-checked against the wall.
- Changed from the design: no `AppsCache::forget`: every moved path joins the state's gone paths and the list hides
  them (one source of truth); `Purpose::Uninstall` carries the app's id for the re-judge; the wall refuses both apps
  folders; a failed canonical check answers `Unmeasured`; `went` takes the first parts of a partly done item.
- A new port method `InstalledApps::is_as_listed` (core, system, testing): the canonical path equals the listed one
  byte for byte, a real folder with `Contents/Info.plist`, asked at prepare and at confirm.
- Ordinary cleanups now refuse `~/Applications` too, and every cleared path joins gone (Largest hides it until the
  next scan).
