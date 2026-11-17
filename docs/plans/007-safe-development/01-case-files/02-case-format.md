# 007/01/02 — The case format

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** sandbox, repo

## Files

- `crates/sandbox/` (new, `fresher-sandbox`; the workspace's `edition`, `rust-version`, `license`, `publish = false`,
  `[lints] workspace = true`, `#![forbid(unsafe_code)]`; depends on `fresher-core` and `fresher-testing`, and from 03 on `fresher-macos`) —
  `src/case.rs`: `CaseFile { feature, module: Option<ModuleId>, cases }`,
  `Case { name, kind: CaseKind, tree: TreeDescription, expect: Option<Expectation> }`, `CaseKind::{Negative, Positive}`,
  `Expectation { item, category: CategoryId, danger: DangerLevel, after: AfterCleanup }`,
  `AfterCleanup::{InTrash, Emptied, Skipped(SkipCode)}`; `CaseFile::parse(text)` and
  `CaseFile::check(&[CategoryDescriptor])`; `CaseFile::tree()` merges every case's tree.
- `crates/sandbox/src/error.rs` — `CaseError` (`thiserror`), each naming its case: a duplicate name, a positive before
  a negative, a positive without `expect` or a negative with one, an `item` that is not one of the case's paths or
  their folders, an unknown category, a danger below the category's, two cases writing one path.
- `Cargo.toml` (member), `scripts/check-crate-deps.ts` and its test — `fresher-sandbox` may depend on core,
  testing and macos, and only `fresher-app` and tests may depend on it.
- `.commitlintrc.cjs` — the scope `sandbox`.

## Produces

`CaseFile` for the generator (03) and the autotests (plan 03).

## Checks

- Negative first: one test per `CaseError`, then `an_unknown_field_is_refused` (`deny_unknown_fields`) and
  `a_path_outside_the_sandbox_is_refused`.
- Positive: `a_case_file_parses_with_negatives_then_positives`, `a_raised_danger_is_accepted`,
  `the_cases_merge_into_one_tree`.
- `cargo nextest run -p fresher-sandbox`; `npm run check:crate-deps`.

## Conclusions

The case format (`fresher-sandbox`) parses a feature's file into negatives and positives with the expected findings, danger
and after-cleanup outcome, and refuses a bad file before anything is written. Review findings: a refused file keeps no ids,
a hard link's target is checked too, and the docs were corrected.
