# Shared helpers of the core e2e homes

**Agreed:** 2026-11-18, by the controller of plan 017 (review of step 03/02) · **Due at:** the next core e2e harness
(a module's `<module>_home`), or the next change to one of the two

## What is postponed

The harness of the `temp` module's core e2e (`crates/e2e/tests/temp_home/mod.rs`, plan
[017/03/02](../plans/017-temp-files/03-end-to-end/02-core-e2e.md)) copies its small helpers from the JetBrains one
(`crates/e2e/tests/jetbrains_home/mod.rs`) rather than sharing them: `until` (the events up to the last one wanted),
`strip` (a key left out of a JSON report), `sort_by_path` (a report's lists in path order), `trash_unnumbered` (the
Trash's order numbers left out of a tree report) and `contents` (a folder's files and their bytes). Copying kept the
JetBrains tests untouched while plan 017 was built; `places.rs` already holds a copy of `until`, and
`cleaning/mod.rs` copies of `until` and `strip`.

## What doing it needs

- Move the five helpers into `crates/e2e/tests/support/` (beside `fixtures` and `check_golden`), one copy each, and
  make the two homes, `places.rs` and `cleaning/` use them.
- No test changes its assertions or its golden; `npm run test:e2e:core` passes before and after.
