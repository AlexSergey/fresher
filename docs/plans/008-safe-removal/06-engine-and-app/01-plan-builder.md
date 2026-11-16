# 008/06/01 — The plan builder and the pending plan

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** engine

## Files

- `crates/engine/src/engine.rs` — `Ports` gains `cleaner: Arc<dyn Cleaner>`, `journal: Arc<dyn CleanupJournal>`,
  `volumes: Arc<dyn Volumes>`; `Engine::prepare_cleanup(&CleanupSelection) -> Result<CleanupPlanView, CommandError>`
  and `Engine::discard_cleanup(PlanId)`.
- `crates/engine/src/cleanup/mod.rs` (new) — the module.
- `crates/engine/src/cleanup/plan.rs` (new) — the builder of the design, section 3, pure over the results:
  refusals (`Busy`, `NoScan`, `UnknownFinding`, `UnknownCategory`, `NothingToClean`, `TooManyItems`), the expansion of
  categories minus their exceptions (a finding named twice counts once), and the left-out reasons in this order: not
  removable, not the subject, a forced tool, covered by another item (by path components), ignored. The items keep the
  order the dialog shows: by category as the overview orders them, then largest first.
- `crates/engine/src/cleanup/blocked.rs` (new) — the fail-closed preconditions, checked at prepare and again at
  confirm: the settings file is `Loaded`, the journal is `Ready`; otherwise `CleaningBlocked { reason }`.
- `crates/engine/src/cleanup/pending.rs` (new) — at most one pending plan: the rehearsal (`Depth::Quick`, the context
  built from the last scan's roots, the current settings, the access status after `refresh_access`, the guarded
  places) gives each item its identity or its skip reason; then `CleanupPlan::new`. The plan dies (`PlanGone` on a
  later confirm) when a scan or a restore starts, a newer plan is prepared, it is discarded, a setting that changes
  the roots, a module's switch or the ignore list changes, or `PLAN_LIFETIME` has passed on the `Clock`.
- `crates/engine/src/cleanup/view.rs` (new) — `CleanupPlanView` from a plan: groups by category with their danger
  level, sizes, items and how they go; the left-out list; the summary as the plan computed it.
- `crates/engine/src/lib.rs` — the module.

## Produces

`prepare_cleanup` and `discard_cleanup` for the commands (04) and the UI (plan 07); the pending plan 02 confirms.

## Checks

With `FakeCleaner`, `MemoryCleanupJournal`, `FakeClock` (negative first):

- `prepare_while_scanning_is_busy`, `prepare_before_any_scan_is_no_scan`, `an_unknown_finding_is_refused`,
  `an_unknown_category_is_refused`, `an_empty_selection_is_nothing_to_clean`, `more_than_the_limit_is_refused`,
  `an_unloaded_settings_file_blocks_cleaning`, `a_read_only_journal_blocks_cleaning`,
  `a_not_removable_finding_is_left_out`, `a_removal_that_is_not_its_subject_is_left_out`,
  `a_forced_tool_command_is_left_out`, `an_item_inside_another_is_covered`,
  `a_rehearsal_skip_is_left_out_with_its_reason`, `a_new_scan_kills_the_pending_plan`,
  `a_newer_plan_replaces_the_pending_one`, `a_changed_root_kills_the_pending_plan`, `a_plan_past_its_lifetime_is_gone`.
- Positive: `a_category_expands_to_its_findings_minus_its_exceptions`, `a_finding_picked_twice_counts_once`,
  `the_view_groups_by_category_in_the_overviews_order`, `the_items_carry_the_rehearsals_identities`,
  `discard_drops_the_pending_plan`.
- Property: `the_plan_never_holds_an_item_that_was_not_selected` (random selections over a random scan).

## Conclusions

- Built as written: `crates/engine/src/cleanup/` with `plan.rs` (expand, sort out, freeze), `blocked.rs`,
  `pending.rs` (the pending plan and `prepare`), `view.rs`, and `context.rs` (the `CleanupContext` as it stands now:
  the roots in the settings, the access probed again, the guarded places under the roots). Every check of the step
  passes, with the property test over random selections.
- Changed from the step, each the simpler or the safer way:
  - **`Ports` keeps its fields**: the cleanup's ports are a struct of their own, `CleanupPorts { cleaner, journal,
    volumes }`, given by `Engine::with_cleanup(registry, ports, cleanup)`. `Engine::new` still builds an engine, one
    that refuses every cleanup (`CleaningBlocked { reason: journal }`) and has no history. New fields on `Ports` would
    break every place that builds it (`app/tauri`, `crates/e2e`), which belong to steps 04 and 06; step 04 wires
    `with_cleanup`.
  - **A scan or a restore kills the plan in `Engine::start`**, under the state's lock, the moment the command is
    taken, rather than on the scan's thread.
  - **A plan prepared while one died is not kept** (`PlanGone`): a count of deaths taken before the rehearsal must be
    the same when the plan is stored, so a scan started during a rehearsal cannot leave a plan of the old findings.
  - **The order of the items**: by category as the overview orders them, then by how they go (moved, emptied, by a
    tool), then largest first, ties by id; so the view's groups (one category, one way) follow the plan's order, which
    is the order the cleaner acts in.
  - **Covered** only by an item that may go: an item under a finding left out for an earlier reason or ignored stays
    in the plan. An item under one the rehearsal later skips is still left out as covered: nothing is acted on twice,
    and the next plan offers it again.
  - **A forced tool** is also one with a cluster of short flags holding `f` (`-rf`); an id among a category's
    exceptions that the scan did not produce is refused like any other (`UnknownFinding`).
  - **The ignore list** does not exist yet (plan 005): `sort_out` takes the ignored paths and is tested with them; the
    engine gives none, and the context's `Protection` holds no ignored path (the cleaner keeps the protected places it
    is built with). The settings that kill the plan are `core.scan.roots` and every `<module>.enabled`.
  - **The roots**: the rehearsal is given the roots of the last scan (kept with the results when a scan starts), the
    context the roots in the settings now; both as the settings write them (`~` is filled in by the walk's adapter,
    so the cleaner must read them the same way).
  - **The lifetime** is checked by the core's `CleanupPlan::confirm` when the plan is confirmed (step 02); the plan
    carries the time from the `Clock`.
