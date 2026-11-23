# 039/02/01 — The core's High consent

**Status:** 🔨 awaiting the user's walk · **Version:** 0.2.0 · **Date:** 2026-11-23 ·
**Modules:** core, engine, testing, cleaner (two test files), e2e, app/tauri, app/ui, docs

A plan whose highest level is High is confirmed only with the user's explicit "I understand these may be the only
copy", sent with the echo and checked by the core, as the permanent consent is. Nothing remembers it: it belongs to one
confirmation. The design: [`../00-design.md`](../00-design.md), section 2; agent's call 4.

## Read first

`crates/core/src/cleanup/confirm.rs` (the echo, `CleanupPlan::confirm`, its tests and `tests/compile_fail/`),
`crates/core/src/error.rs` (`CommandError`, `CODES`, `one_of_each`), `crates/engine/src/cleanup/operation.rs`
(`confirm`: an exhaustive `match` turns a `ConfirmError` into a `CommandError`; after it, an uninstall's permanent echo
is refused), and every echo builder. On 740cf3c3 they are, every one a JSON object deserialized into a
`Confirmation`:

- core: the tests of `crates/core/src/cleanup/confirm.rs`; `crates/core/tests/compile_fail/confirmation_literal.rs`
  and its `.stderr` (the literal lists the fields: it gains `high_risk_understood: false`, and the `.stderr` is
  regenerated with `TRYBUILD=overwrite` and read);
- `crates/testing/src/cleanup_fakes.rs` (three echoes, all of Low plans);
- `crates/cleaner/src/test_support.rs` (a `#[cfg(test)]` module) and `crates/cleaner/tests/never.rs`: the only change
  in `crates/cleaner` (design call 15);
- `crates/engine/src/cleanup/testing.rs` (`echo`, `echo_with`);
- `crates/e2e/tests/`: `cleanup.rs` (the wrong-count echo), `places.rs`, `uninstall.rs`, `trash_home/mod.rs`,
  `support/home_cleanup.rs`, `cleaning/mod.rs`, `jetbrains_home/mod.rs`;
- `app/tauri/src/`: `commands.rs` (its test), `testing.rs` (`echo`), `sandbox/tests.rs`, `dev_runs/shell_tests.rs`
  (`record_cleanup`); `app/tauri/tests/sandboxes/`: `harness.rs` (`clean`), `uninstall.rs` (`confirm`);
- the UI: `app/ui/src/state/cleanup.ts` (`confirmationOf`) and the tests that write an echo
  (`state/cleanup.test.ts`, `screens/review/review-screen.test.tsx`, `screens/apps/app-screen.test.tsx`,
  `ipc/index.test.ts`); the e2e mock's `confirm` in `app/ui/e2e/support/cleanup-mock.ts`.

Check the list again with `grep -rln '"method"' crates app` and `grep -rln 'Confirmation' app/ui/src app/ui/e2e`.

## Files

- `crates/core/src/cleanup/confirm.rs`:
  - `Confirmation` gains `#[serde(rename = "highRiskUnderstood")] high_risk_understood: bool` (required; the struct
    keeps `deny_unknown_fields`); its doc comment names the field.
  - `ConfirmError::HighRiskNotUnderstood` (`#[error("the plan holds a High item and its risk was not confirmed")]`).
  - `CleanupPlan::confirm`: after the count, allocated and highest checks, `if summary.highest == DangerLevel::High &&
    !echo.high_risk_understood { return Err(ConfirmError::HighRiskNotUnderstood) }`. The plan is consumed, as for every
    refusal. The module doc's list of guarantees gains the sentence.
- `crates/core/src/error.rs` — `CommandError::HighRiskNotUnderstood` (`#[error("the risk of a High item was not
  confirmed")]`), in `CODES` (`high-risk-not-understood`) and `one_of_each`.
- `crates/core/locales/en.json` — `core.error.high-risk-not-understood`: "Tick “I understand these may be the only
  copy” before you confirm. Nothing was removed."
- `crates/engine/src/cleanup/operation.rs` — maps `ConfirmError::HighRiskNotUnderstood` to
  `CommandError::HighRiskNotUnderstood` with a `tracing::warn!` (the UI never sends it false for a High plan); the
  function's doc lists the new refusal. An uninstall of everything (`core.uninstall-everything`, High) is refused
  for it before the uninstall's own permanent check and before it is judged again: nothing is acted on.
- Every echo builder above sends `highRiskUnderstood`: the test helpers take it from the plan
  (`summary.highest == High`) unless a test asks otherwise (`echo_with` gains the option); the sandbox harness's
  `clean` and `sandboxes/uninstall.rs`'s `confirm` send `true` only for a High plan (they confirm as a user who
  ticked); the cleaner's, `crates/testing`'s and the core's own fixed echoes send the value their plan needs.
- `app/ui/src/state/cleanup.ts` — `confirmationOf(plan, method, highRiskUnderstood: boolean)`; `cleanup-actions.ts`'s
  `confirm(store, method, highRiskUnderstood)`; the confirm dialog's `onConfirm` passes `false` from both owners,
  `screens/review/review-screen.tsx` and `screens/apps/app-screen.tsx`, until 02/02 adds the tick (a High plan, an
  uninstall of everything or a Largest pick, cannot be confirmed from the UI between the two steps; 02/02 follows at
  once). The bindings regenerated (`UPDATE_BINDINGS=1 cargo nextest run -p fresher-app --test bindings`).
- `app/ui/e2e/support/cleanup-mock.ts` — `confirm` refuses, after its echo checks, a plan whose `highest` is `high`
  with `highRiskUnderstood` not `true`, as the core does: `HIGH_RISK_NOT_UNDERSTOOD = refusal({ code:
  'high-risk-not-understood' }, 'core.error.high-risk-not-understood')` beside `PLAN_MISMATCH` in
  `app/ui/src/test/cleanup-plan.ts`, and the plan is gone. The e2e flows that confirm a High plan fail until 02/02
  ticks; run 02/02 at once (its list names them).
- `docs/restrictions/deletion-safety.md` — next to the permanent consent: a High plan needs the user's tick in the
  confirmation, sent with the echo and checked by the core; never remembered, never a setting.

## Tests (negatives first)

- `confirm.rs` negative: "a High plan without the risk understood is refused" (a sample plan with a High item);
  "a High plan with a wrong count is refused for the count, not the risk" (the order); "the refusal consumes the plan";
  "a confirmation without the field is not read" (`deny_unknown_fields` and a missing field fail to deserialize);
  positive: "a High plan with the risk understood is confirmed"; "a plan below High is confirmed with the field false";
  "the field true on a Low plan changes nothing".
- `operation.rs` (or its tests file): negative — "confirm of a High plan without the risk is refused and acts on
  nothing" (no journal record, the tree unchanged).
- A new `crates/e2e/tests/high_risk.rs` on fixtures that already state a High finding on 740cf3c3
  (`crates/e2e/tests/downloads_home/`, the large untouched file; `uninstall_home/`, an uninstall of everything):
  negative — "a High cleanup without the tick moves nothing"; "an uninstall of everything without the tick moves
  nothing"; positive — "a High cleanup with the tick goes to the Trash and comes back by put back"; "an uninstall
  that keeps the data needs no tick" (Medium).
- `app/ui/src/state/cleanup.test.ts`: "the confirmation sends the risk as given".
- `app/ui/e2e/core-mock.spec.ts`: "the mock refuses a High echo without the risk understood".

## Checks

- `CARGO_TARGET_DIR=<target> cargo nextest run --workspace` (the core's compile-fail tests included);
  `cargo clippy --workspace --all-targets -- -D warnings`; `npm run sandbox:test`; `npm run lint:ts`;
  `npx vitest run app/ui/src/state app/ui/src/ipc app/ui/src/screens/review app/ui/src/screens/apps`.
- `git diff -- crates/cleaner` touches only `src/test_support.rs` and `tests/never.rs`, and only their echo;
  `git diff -- crates/scanner crates/macos crates/system crates/store crates/rules modules` is empty.

## Conclusions

- `Confirmation.high_risk_understood` (`highRiskUnderstood`) is required; `ConfirmError::HighRiskNotUnderstood` is
  checked after count, allocated size and highest level, and the plan is consumed. The engine answers
  `CommandError::HighRiskNotUnderstood`, before anything an uninstall adds. It is never remembered and never a
  setting.
- The echo builders changed in 19 places (the core, `crates/testing`, the e2e helpers, the shell's tests, the sandbox
  harness). `crates/cleaner` changed in two test files only (`src/test_support.rs`, `tests/never.rs`: one line each).
- The e2e cleanup mock refuses a High echo without the field. `high_risk.rs` proves a High cleanup and an uninstall of
  everything are refused and move nothing, and that keeping the data needs no tick.
- Between this step and 02/02 the four High flows of the UI e2e were `test.fixme`; 02/02 lifted them.
