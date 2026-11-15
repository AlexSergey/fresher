# 002/06/03 — Test harness and category contract

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** testing

## Files

- `crates/testing/Cargo.toml` — adds `serde_json` (golden findings); stays dependent on `fresher-core` only.
- `crates/testing/src/naive_walk.rs` — `NaiveWalk`: the reference `DiskWalk` over a fixture root with `std::fs`
  (`symlink_metadata`, links listed, never followed; no other device), matching the `InterestSet` simply and slowly.
  Plan 004's scanner is tested against it: the same fixture gives the same matches.
- `crates/testing/src/run_module.rs` — `run_module(&dyn Module, &FixtureTree, Ports) -> ModuleRun { findings,
  outcome }`: walks with `NaiveWalk`, sends matches, calls `finish`; `ModuleRun::golden()` gives a stable JSON report
  (paths relative to the fixture root).
- `crates/testing/src/contract.rs` — `assert_module_contract(&dyn Module, Catalogue)`: the descriptor validates; every
  category has its five texts and every setting its label and description; every key is inside the module's
  namespace; with `Catalogue::Required(docs_dir)`, every category `<module>.<name>` has
  `docs/categories/<module>-<name>.md` and that file names the same danger level; `Catalogue::DevOnly` skips the
  files (the `demo` module). `assert_texts_used(&dyn Module, &[ModuleRun])`: every key in `en.json` is a descriptor key
  or appears in a finding of the given runs; every finding's keys exist and get the args their placeholders name.

## Produces

What every module's tests call (06/04 and every module plan), and the reference walk of plan 004.

## Checks

- Negative: `the_walk_does_not_follow_a_symlink_out_of_the_root`, `the_contract_fails_for_a_category_without_its_text`,
  `the_contract_fails_for_a_category_without_its_catalogue_file`,
  `the_contract_fails_when_the_catalogue_names_another_danger_level`, `an_unused_key_fails`,
  `a_missing_placeholder_arg_fails`.
- Positive: `run_module_returns_the_findings_of_a_test_module`, `the_golden_report_uses_paths_relative_to_the_root`.
- `cargo llvm-cov nextest -p fresher-testing` ≥ 90 % of lines.

## Notes from the implementation

- `NaiveWalk::new(home)`: `Pattern::UnderHome` paths are relative to `home` (the fixture root in a test), and a `*`
  segment matches any one name. `FolderName` matches a folder or a link of that name (links are reported so that a
  module can refuse them), `Extension` a file or a link, `MarkerFile` a real folder holding a regular file of that
  name. Entries are visited in name order. A `Claim::Whole` folder is summed (`Partial` when a folder inside could not
  be read) and not walked inside; a `Claim::Item` entry has its own size and the walk goes on. An entry on another
  device is skipped. Unreadable folders are counted in the summary (`NoPermission` or `Failed`).
- `run_module(&dyn Module, &FixtureTree, &Ports) -> Result<ModuleRun, HarnessError>`: `Ports` is taken by reference,
  so a test can read `FakeCommandRunner::calls` afterwards, and is built with `Ports::at(seconds)` (the clock, no
  command answers, stored settings resolved to the module's defaults, a fresh cancel token). It returns a `Result`
  because the walk can fail on a broken fixture and the library code may not panic: `HarnessError::{Walk, Schema}`.
  A module error ends the run with that outcome (`Failed` or `Cancelled`); a cancelled walk ends it as `Cancelled`.
- `ModuleRun::golden() -> Result<String, serde_json::Error>`: `{ "outcome", "findings" }`, findings sorted by their
  key, every string that is a path under the root made relative (the root itself is `.`), pretty-printed with a final
  newline.
- The contract checks come in two forms: `check_module_contract` / `check_texts_used` return the list of
  `ContractProblem`s (what the negative tests compare), and `assert_module_contract` / `assert_texts_used` fail the
  test with all of them. `Catalogue::Required(PathBuf)` holds the folder of the catalogue files
  (`docs/categories`); a file's danger level is read from its `**Danger level:**` line.
- A plural text (`<key>.one`, `<key>.other`) stands for `<key>`, both for "unused" and for "missing"; a failure's
  reason (`ModuleOutcome::Failed`) counts as a used key like a finding's title and reason.
- `docs/testing.md`, "Where things live", still lists only the old helpers: it is not a shared file of this step, so
  the list is left for step 04 or plan 07.
