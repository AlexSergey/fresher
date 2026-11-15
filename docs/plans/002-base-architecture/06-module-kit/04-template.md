# 002/06/04 — Template, checklist and the demo through the kit

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** modules, docs

## Files

- `docs/architecture/modules.md` — how a module is written: the crate skeleton (`Cargo.toml`, `src/lib.rs`,
  `rules.toml`, `locales/en.json`, `src/settings.rs`, `tests/`, `fixtures/homes/<module>-*.toml`,
  `fixtures/golden/<module>.json`), the `Module` trait in short, when to use rules and when code, the one line of
  registration in `app/tauri/src/compose.rs`, and **the checklist** a module ticks before it ships: categories
  documented, five texts each, negative fixtures first (look-alike, current version, link out of the root), golden
  report, contract test, no dependency on another module, danger levels never lowered, plain-English texts (rule 7).
- `docs/categories/README.md` — the format: files `<module>-<name>.md` for category `<module>.<name>`; "every file here
  matches a category of a module" (not "a rule in `crates/rules`"); the contract test named.
- `modules/demo/Cargo.toml` — dev-dependency `fresher-testing`.
- `modules/demo/tests/contract.rs` — `assert_module_contract(&DemoModule, Catalogue::DevOnly)` and
  `assert_texts_used` over one `run_module` on `fixtures/homes/empty.toml`.

## Produces

The pattern every module plan follows.

## Checks

- Negative: the demo's contract test fails when a category text is removed from its `en.json` (checked once by hand,
  written in the conclusions).
- Positive: `the_demo_module_passes_the_contract`, `every_demo_key_is_used`.
- Plan 06's conclusions written.

## Notes from the implementation

- **The kit now wants the texts of `<module>.enabled`.** The first run of `every_demo_key_is_used` reported
  `demo.enabled.label` and `demo.enabled.description` as unused: the settings schema adds the switch for every module
  and the UI shows it, but the contract of 06/03 counted only the descriptor's own settings. `fresher-testing` now
  counts `<module>.enabled` as a setting of every module, so a module without its texts breaks the contract
  (`the_contract_fails_without_the_texts_of_the_enabled_switch`); the sample module of the kit's tests got them.
- **`demo.error.failed` is the one text no run shows**: the demo fails only when it cannot build a tool command,
  which its fixed names never provoke, and the error path exists so that the library code does not panic.
  `every_demo_key_is_used` therefore calls `check_texts_used` and expects exactly that one `UnusedText` problem;
  anything else fails it.
- The demo's hand-written `every_text_key_it_emits_is_in_its_dictionary` unit test is removed: the two contract tests
  check the same and more. `fresher-testing` was already a dev-dependency of the demo (02/05).
- `docs/architecture/modules.md` is not yet linked from `docs/architecture/README.md`: that file is rewritten by
  07/02, which adds the link. `app/tauri/src/compose.rs` does not exist yet on this branch (plan 05 creates it); the
  registration line follows `Registry::register` of the engine.
- **The negative check, by hand**: with `demo.volumes.comes-back` removed from `modules/demo/locales/en.json`,
  `the_demo_module_passes_the_contract` fails with "`demo.volumes.comes-back` has no text in the dictionary"; the
  text was put back.
- `docs/testing.md`, "Where things live", now lists `NaiveWalk`, `run_module`, the contract checks and
  `fresher-rules` (left open by 06/03).
