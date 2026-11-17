# 011/02/01 — The module's API

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core

Renamed 2026-11-17: `LeftOut` was taken by plan 008's cleanup type; the module's item is `LeftOutItem`.

## Files

- `crates/core/src/left_out.rs` (new) — `LeftOutItem { category: CategoryId, reason: TextKey, sizes: Sizes }`: an item a
  module recognized and did not offer, with no path and no args; `LeftOutCount { category: CategoryId, reason: Text,
  items: u32, sizes: Sizes }` (`Serialize`, `TS`, camelCase): the engine's sum for one category and one reason.
  `LeftOutCount::add(&mut self, sizes)` saturates `items` and keeps the weakest completeness, as `Sizes` sums do.
- `crates/core/src/module.rs`:
  - `FindingSink::left_out(&mut self, left_out: LeftOutItem)`, required. `impl FindingSink for Vec<NewFinding>` drops it,
    and its doc says so: it is for tests that look at findings only (agent's call 3).
  - `ModuleDescriptor::with_problems(self, keys: Vec<TextKey>) -> Result<Self, DescriptorError>` and `problems()`:
    the keys the module may report as a failure or a left-out reason. `DescriptorError::ForeignProblem { key,
    module }` for a key outside the module's namespace, `DuplicateProblem { key }` for a key listed twice.
  - The doc of `ModuleError::Failed`: its `reason` is a declared problem, and a technical detail goes under
    `DETAIL_ARG`.
- `crates/core/src/event.rs` — `Event::ModuleFinished` gains `left_out: Vec<LeftOutCount>` (`leftOut` in TypeScript),
  sorted by category, then by reason key.
- `crates/core/locales/en.json` — `core.left-out.summary.one`: "1 item left out ({size}):" and `.other`: "{count}
  items left out ({size}):".
- `crates/core/src/texts.rs` — the key joins `core_keys()`.
- `crates/engine/src/session.rs` — the stub of `Output::left_out` (above).
- `crates/core/src/lib.rs` — the exports; `crates/core/tests/typescript.rs` and `app/ui/src/ipc/bindings.generated.ts`
  regenerated (`ModuleFinished`, `LeftOutCount`).

## Produces

`LeftOutItem`, `LeftOutCount`, `FindingSink::left_out` and `ModuleDescriptor::problems` for 02/02; `leftOut` on the
`module-finished` event for 02/03.

## Checks

- Negative: `a_problem_outside_the_module_is_refused`, `a_problem_listed_twice_is_refused`,
  `a_vec_sink_keeps_no_left_out`, `a_left_out_count_never_overflows`.
- Positive: `a_descriptor_keeps_its_problems_in_order`, `left_out_counts_add_items_and_sizes`,
  `module_finished_serializes_its_left_outs` (JSON shape, camelCase), `the_summary_text_is_in_the_core_dictionary`
  (through `core_keys()`, both plural forms).
- `cargo nextest run -p fresher-core`; the TypeScript bindings test passes; the rest of the workspace builds with
  `left_out` added to every sink (`cargo clippy --workspace --all-targets -- -D warnings`). The one other sink, the
  engine's `Output` (`crates/engine/src/session.rs`), gets a stub that keeps nothing until 02/02, marked so in its doc;
  the harness collects into a `Vec` until 02/02 too.

## Conclusions

Built (039c2a9). The new type is `LeftOutItem { category, reason, sizes }`, not `LeftOut`: that name is plan 008's
cleanup type. `LeftOutCount` adds items and sizes with saturation and keeps the weakest completeness.
`FindingSink::left_out` is required; the `Vec<NewFinding>` sink drops left-outs and says so. `with_problems` refuses a
foreign or a doubled key (`ForeignProblem`, `DuplicateProblem`). `ModuleFinished` gained `leftOut`, and the two
`core.left-out.summary` texts (`.one`, `.other`) joined the core dictionary.

### Agent's calls taken during the build (to check)

1. The new type is `LeftOutItem`, because `LeftOut` is plan 008's. Cost if wrong: none beyond a different name in the docs.

