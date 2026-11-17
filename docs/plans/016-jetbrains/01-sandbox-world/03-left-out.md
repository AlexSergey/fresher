# 016/01/03 — The left-out expectation

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
sandbox, app, docs

Plan 011's agent's call 6: the first module that reports a left-out adds its expectation to the case format.

## Files

- `crates/sandbox/src/case.rs` — `[[case.left-out]]` (`category`, `reason`, `items` ≥ 1) on any case; checked: the
  category is in the module's descriptor, the reason is one of its declared problems (`with_problems`).
- `crates/sandbox/src/expected.rs` — "Under '<category name>', <items> left out: <reason's text>" (the texts from the
  module's dictionary).
- `app/tauri/tests/sandboxes/harness.rs` — collects `Event::ModuleFinished.left_out` of the scan; sums the case file's
  expectations per (category, reason) and compares them exactly with the counts (items); a pair the module reports
  and no case expects fails too, naming it. The left-out counts go into the golden report.
- `docs/testing.md` — "Sandboxes", "The case file": `[[case.app]]`, `[[case.running]]`, `[[case.left-out]]`,
  `world.toml`, sandbox mode's simulated open apps.

## Produces

The left-out checks 03/01 relies on.

## Checks

- Negative first: an undeclared reason, an unknown category, `items = 0` are refused when read; the harness fails on an
  unexpected left-out and on a missing one (unit tests of the comparison on plain values).
- Positive: a case file with no left-out expectation and a module that reports none passes (every existing case file
  and golden unchanged).
- `cargo nextest run -p fresher-sandbox`; `npm run sandbox:test`.

## Conclusions

Built as written, with one deviation: `EXPECTED.md` names a left-out by the category id and the reason key, not by
the dictionary's text (the sandbox crate and its binary have no module dictionary). `[[case.left-out]]` (category,
reason, items) is compared exactly both ways by the harness (`tests/sandboxes/left_out.rs`), summed per category and
reason; an unknown category or an undeclared reason is refused when the case file is read (`Engine::problems()`).
The report gets `leftOut` only when it is not empty, so earlier goldens did not change.
