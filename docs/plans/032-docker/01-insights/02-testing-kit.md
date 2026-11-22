# 032/01/02 — The test kit

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** testing

Read first: [`../00-design.md`](../00-design.md) section 1, `crates/testing/src/run_module.rs` (`Recorder`,
`ModuleRun`, `golden`), `crates/testing/src/contract.rs` (`check_texts_used`, `run_texts`, `text_problems`,
`ContractProblem`), and 01/01's `crates/core/src/insight.rs`.

`run_module` records what a module describes, the golden report shows it, and the contract checks its texts, so the
`docker` module (plan 02) is tested like every other.

## Files

- `crates/testing/src/run_module.rs`
  - `Recorder` keeps insights (`FindingSink::describe`); `ModuleRun` gains `pub insights: Vec<Insight>` ("What the
    module described and does not offer, in the order it gave it.").
  - `ModuleRun::golden` adds `"insights"` (the insights as serialized, in order) when the run has any, after
    `leftOut`; paths stay relative as today. A run without insights keeps its golden byte for byte (every existing
    golden is unchanged).
- `crates/testing/src/contract.rs`
  - `check_texts_used` counts every text of an insight as used and checks each as it checks a finding's (a missing
    text, a missing placeholder): the title, the summary, each figure's label, each part's label, detail and effect,
    each note. A plural text (`.one`/`.other`) stands for its key, as today.
  - `ContractProblem::ForeignInsightText { key }` ("the insight text `{key}` is not in the module's namespace"): an
    insight's key whose namespace is not the module's id.
  - `ContractProblem::DuplicateInsight { identity }` ("two insights of one run share the identity `{identity}`").
  - Update the doc comment of `check_texts_used` to name insights.

## Produces

`ModuleRun.insights`, the golden's `insights`, the contract's insight checks, for plan 02.

## Tests (negative first)

In `crates/testing` (with a small test module made in the test, as the contract's existing tests do):

- `mod negative`: `an_insight_text_without_a_dictionary_entry_is_missing`, `an_insight_text_of_another_namespace_is_foreign`,
  `an_insight_text_missing_a_placeholder_is_reported`, `two_insights_with_one_identity_are_a_problem`,
  `a_run_without_insights_keeps_its_golden_unchanged` (no `insights` key).
- `mod positive`: `an_insight_s_texts_count_as_used`, `the_golden_lists_insights_in_order`,
  `run_module_records_what_a_module_describes`.

## Done when

- `cargo nextest run -p fresher-testing` and every module's tests (`cargo nextest run --workspace` once: goldens
  unchanged) pass; clippy; fmt.
- One commit: `test(testing): run_module and the contract know insights`.

## Conclusions

Not written yet.
