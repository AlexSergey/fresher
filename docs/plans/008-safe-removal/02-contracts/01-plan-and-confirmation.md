# 008/02/01 — The plan and its confirmation

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** core

## Files

- `Cargo.toml` (root) — `[workspace.dependencies]` pins the three new crates of the design, section 12, at their
  current versions (`trybuild`; `objc2` and `objc2-foundation` for plan 03, so no later plan edits the root file); each
  checked with `cargo deny check`.
- `crates/core/Cargo.toml` — `trybuild` as a dev-dependency.
- `crates/core/src/cleanup/mod.rs` (new) — the module and its exports.
- `crates/core/src/cleanup/plan.rs` (new) — `PlanId`, `CleanupId` (`new`, `get`, serialized as numbers),
  `CleanupSelection` and `CategoryPick` (`Deserialize` only), `Method` (`trash`, `permanent`), `ItemIdentity`,
  `Action` (`remove`, `empty`, `tool`; `Action::from_removal(&Removal)`), `PlanItem`, `LeftOut`, `PlanSummary` and
  `CleanupPlan` with read accessors; `CleanupPlan::new(id, made, scan, roots, items, left_out)` computes the summary
  itself (count, summed sizes with the weakest completeness, highest danger level, the number of tool items), so no
  caller can hand it a summary that disagrees with its items. `MAX_PLAN_ITEMS = 10_000`; `PLAN_LIFETIME` = 10 minutes.
- `crates/core/src/cleanup/confirm.rs` (new) — `Confirmation` (`Deserialize` only; private fields; no constructor, no
  `Default`), `ConfirmError` (`thiserror`: `Mismatch { field }`, `Expired`, `WrongPlan`), `ConfirmedPlan` (not
  `Clone`, no serde), `ConfirmedMethod`, `PermanentConsent(())`, `Act<'a>`; `CleanupPlan::confirm(self, &Confirmation,
  now) -> Result<ConfirmedPlan, ConfirmError>` compares the plan id, the count, the allocated bytes, the highest danger
  level, and the age against `PLAN_LIFETIME`; `ConfirmedPlan::{items, method, act, permanent}`.
- `crates/core/src/lib.rs` — `mod cleanup;` and the exports.
- `crates/core/tests/types.rs` (new) — the `trybuild` runner for `tests/compile_fail/*.rs`.
- `crates/core/tests/compile_fail/` (new) — one case per guarantee of the design, section 2, each with its `.stderr`:
  `confirmed_plan_literal.rs`, `permanent_consent_literal.rs`, `confirmation_literal.rs`, `act_literal.rs`,
  `confirmed_plan_clone.rs`, `confirmed_plan_used_twice.rs`, `confirmation_from_plan.rs` (no `From`, no builder).

## Produces

The types the plan builder (06/01), the cleaner (04) and the journal (05) are written against; the only path from a
plan to an act.

## Checks

- Negative (first): `a_confirmation_of_another_plan_is_refused`, `a_different_count_is_refused`,
  `a_different_size_is_refused`, `a_different_highest_level_is_refused`, `a_plan_older_than_its_lifetime_is_refused`,
  `a_permanent_method_alone_gives_no_consent_for_the_trash` (a `Trash` confirmation's plan has no consent),
  `a_selection_with_an_unknown_field_is_refused` (`deny_unknown_fields`).
- Positive: `a_matching_confirmation_confirms`, `a_permanent_confirmation_carries_the_consent`,
  `the_summary_is_computed_from_the_items`, `an_action_follows_the_removal`, `ids_serialize_as_numbers`.
- Property: `any_changed_field_of_the_echo_refuses` (proptest over the five fields).
- The compile-fail cases fail to compile with the expected message; `cargo nextest run -p fresher-core`; `cargo
  clippy --workspace --all-targets -- -D warnings`; `cargo deny check`.

## Conclusions

- Built as written in `crates/core/src/cleanup/` (`plan.rs`, `confirm.rs`), exported from the crate root; seven
  compile-fail cases in `crates/core/tests/compile_fail/` run by `tests/types.rs` (`TRYBUILD=overwrite` rewrites the
  `.stderr` files). The root `Cargo.toml` pins `trybuild` 1.0.125, `objc2` 0.6.5 and `objc2-foundation` 0.3.2
  (`default-features = false`, `std` only: plan 03 names the classes it uses as features of its own crate); `cargo
  deny check` passes. `trybuild` is listed in `docs/dependencies.md`; the two `objc2` crates are listed by plan 03,
  which is the first to use them.
- Not spelled out by the step, taken the simplest or the safest way:
  - **`SkipReason` comes with this step** (`cleanup/outcome.rs`), since `LeftOut` holds one; 02 adds its texts.
  - **`CleanupPlan::new` returns a `Result`** (`PlanError`): more than `MAX_PLAN_ITEMS` items (`TooManyItems`), a
    finding twice (`Duplicate`: nothing is acted on twice), sizes that do not fit (`Overflow`). The cap is held by
    the type as well as by the engine's refusal.
  - **`PlanItem::new(&Finding)`** takes every field from the published finding (its action from the removal) with no
    identity; `with_identity` sets the one the rehearsal read. `PlanSummary::of(items)` is public, so the engine sums
    a group of the view the same way. `ActionKind` (`remove`, `empty`, `tool`) is the action without its target.
  - **`ItemIdentity::new(device, file, kind)`** is public: it is data the cleaner reads, not a licence.
  - **Expiry**: refused when `now - made` is over `PLAN_LIFETIME` (600 s), and when `now` is before `made` (a clock
    set back): fail closed. Checks run in this order: the plan id (`WrongPlan`), the age (`Expired`), then the count,
    the allocated bytes and the highest level (`Mismatch { field: EchoField }`).
  - **The property test** varies the four compared fields (each either the plan's value or any value) and the
    method: it confirms exactly when the four equal the plan's, and the method is carried as sent (a permanent one
    with its consent). The method itself cannot mismatch: it is the user's choice.
  - `Confirmation` has two accessors (`plan`, `method`) for the engine; `Act` is `Copy` (it is borrowed from its plan
    and cannot outlive it) and gives back its plan (`Act::plan`). `CleanupPlan` is neither `Clone` nor
    (de)serializable, so the engine cannot keep a second copy to confirm.
  - `confirmed_plan_used_twice.rs` calls a local `fn execute(ConfirmedPlan)`; the `Cleaner` port of 04 takes the plan
    the same way (its own case is added there).

