# 029/01/01 — The core's types and the engine

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** core, engine, testing, app

`Removal::MoveTogether(Parts)` from a module's finding to a plan item: its validation, its removability, the plan
builder's rules, the claims, the confirmation's view, and the fakes. The cleaner, the journal's file and the UI come in
02 to 04. The design: [`../00-design.md`](../00-design.md), section 1.

## Files

- `crates/core/src/removal.rs` — `MAX_PARTS` (64); `Removal::MoveTogether(Parts)` (serialized `move-together` with
  the list of paths); `pub struct Parts(Vec<PathBuf>)` with `Parts::new(Vec<PathBuf>) -> Result<Parts, RemovalError>`
  and `subject()`, `all()`, `rest()`. `RemovalError` gains `TooFewParts`, `TooManyParts { count }`, `NotPlain {
  index }` (not absolute, or a `.` or `..` component), `PartTwice { index }`, `PartInsidePart { inner, outer }`
  (compared by components, never by string prefix).
- `crates/core/src/removability.rs` — `of_removal(removal, facts: impl Fn(&Path) -> Option<EntryFacts>)`: a single
  path as today; a `MoveTogether` is the removability of the first part, in order, that is not `Removable`, and
  `NeedsAdministrator` for a part with no facts; a tool's removal unchanged.
- `crates/core/src/cleanup/plan.rs` — `Action::RemoveTogether(Parts)` from `Removal::MoveTogether`; `kind()` is
  `ActionKind::Remove`; `path()` the subject; `parts() -> &[PathBuf]` (`[path]` for `Remove` and `Empty`, all parts,
  `[]` for a tool). `PlanItem::with_part_identities(Vec<ItemIdentity>)` (sets `identity()` to the first) and
  `part_identities() -> &[ItemIdentity]` (empty for every other action).
- `crates/core/src/cleanup/port.rs` — `Rehearsal::PassedTogether(Vec<ItemIdentity>)`, one per part in order; the
  `Cleaner` trait's docs say so.
- `crates/core/src/cleanup/outcome.rs` — `Failure::Partly { done: u32, of: u32 }`, text `core.cleanup.failed.partly`
  with the counts as `TextArg::Count`; `crates/core/locales/en.json` — its sentence (design section 1).
- `crates/core/src/cleanup/view.rs` — `PlanItemView.more: Vec<PathBuf>`: the parts after the subject, empty otherwise.
- `crates/engine/src/session.rs` — `push` gives `of_removal` the scan's matched facts by path.
- `crates/engine/src/cleanup/plan.rs` — `refused`: `(MoveTogether(parts), Subject::Path(subject))` passes when
  `parts.subject() == subject`, else `NotTheSubject`; `sort_out`: every part of a kept item enters `first` (so a
  single item of the same path or inside a part is `Covered`), and a together item is left out whole when any part is
  covered by another kept item (`Covered`) or is ignored, inside an ignored path or above one (`Ignored`);
  `action_order` puts it with the moved items; `freeze` takes `PassedTogether` into `with_part_identities`.
- `crates/engine/src/claims.rs` — a together finding claims each part for the storage map; it counts its sizes when it
  owns every part and nothing otherwise; nesting is compared by components.
- `crates/testing/src/` — `FakeCleaner` rehearses a together item part by part (its scripted refusals by path apply to
  any part) and answers `PassedTogether`; `assert_module_contract` checks every part of every removal is a matched
  path; the finding builders of the engine's tests gain a `together(…)` helper.
- `app/ui/src/ipc/bindings.generated.ts` — regenerated (the new `Removal` variant and `PlanItemView.more`); the UI's
  type checks pass with no other change (04 shows `more`).

## Produces

`Removal::MoveTogether`, `Parts`, `MAX_PARTS`, `Action::RemoveTogether`, `Action::parts`,
`Rehearsal::PassedTogether`, `PlanItem::part_identities`, `Failure::Partly`, `PlanItemView.more`, for 02 to 04 and the
module.

## Checks

Negatives first, in each file's `mod negative`, then `mod positive`:

- `removal.rs`: `parts_refuse_fewer_than_two`, `parts_refuse_more_than_the_limit` (65), `parts_refuse_a_relative_or_dotted_path`,
  `parts_refuse_a_path_twice`, `parts_refuse_a_part_inside_another` (and `/a/b` beside `/a/bc` is accepted: components);
  positive `parts_keep_their_order_and_the_first_is_the_subject`, `move_together_serializes_as_a_list`.
- `removability.rs`: `a_together_removal_is_the_first_part_that_cannot_go`, `a_part_without_facts_needs_the_administrator`;
  positive `a_together_removal_of_removable_parts_is_removable`.
- `cleanup/plan.rs` (engine): `a_together_removal_whose_first_part_is_not_the_subject_is_not_the_subject`,
  `a_together_item_with_a_covered_part_is_left_out_whole`, `a_together_item_with_an_ignored_part_is_left_out_whole`,
  `a_single_item_inside_a_part_is_covered`; positive `a_together_item_goes_with_the_moved_items`,
  `a_passed_together_rehearsal_keeps_every_identity`.
- `claims.rs`: `a_together_finding_that_loses_a_part_counts_nothing`; positive `a_together_finding_claims_each_part`.
- `session.rs`: positive `a_together_findings_removability_reads_every_part`.
- `cargo nextest run -p fresher-core -p fresher-engine -p fresher-testing`; the modules' contract tests still pass;
  `npm run typecheck -w app/ui`; clippy; fmt.

## Conclusions

Built as written, plus what the preflight and the reviews added. `Removal::MoveTogether(Parts)` validates its
parts (two to 64, plain, no part twice or inside another, compared by components); `of_removal` reads every part's
facts; the plan builder leaves a together item out whole when any part is covered or ignored, and a dropped item covers
nothing; the claims count a together finding's sizes only when it owns every part. `PassedTogether` carries one identity
per part and `freeze` refuses a count that differs (preflight F23). Calls taken: while only this step existed the
cleaner failed closed on a together item (`Skipped(CheckFailed)` in every arm, never the single-path arm: preflight F1),
and step 02 replaced those arms; `FakeCleaner` keys its scripted skips by finding, so they apply to the whole item
(F7); seven e2e journal goldens and `cleanup-plan.ts` gained `more` (F2). Review fix (final wave): the claims' `losers`
holds only together findings, so a single finding that lost a part to a together finding that then lost is counted
again. Review: approved.
