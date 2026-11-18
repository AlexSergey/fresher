# The module harnesses of the core e2e share their code

**Agreed:** 2026-11-18, by the controller of plan 018 (review of step 03/02) · **Due at:** the next core e2e module
harness (the uninstalled app leftovers module's)

## What is postponed

`crates/e2e/tests/dev_caches_home/mod.rs` repeats about 150 lines of `crates/e2e/tests/temp_home/mod.rs`: the sandbox
layout, restore and walk, the findings, the clean, the outcomes, the journal, the relative paths, the user and the
engine. Plan 018 moved the small helpers (`until`, `strip`, `sort_by_path`, `trash_unnumbered`, `contents`) into
`crates/e2e/tests/support/`, one file per concern, and left the homes' own code as copies so the temp and JetBrains
tests stayed untouched.

## What doing it needs

- Move the shared part of the two homes into one file under `crates/e2e/tests/support/`, pulled in with `#[path]` by the
  homes that use it (as the helpers are), each home keeping only what is its module's own (its fixture, its fakes, its
  scripted facts).
- No test changes its assertions or its golden; `npm run test:e2e:core` passes before and after, and `cargo clippy -p
  fresher-e2e --all-targets -- -D warnings` stays clean.
