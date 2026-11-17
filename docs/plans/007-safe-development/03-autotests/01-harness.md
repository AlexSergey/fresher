# 007/03/01 — The harness

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** app, repo

## Files

- `app/tauri/tests/sandboxes.rs` (new) — `check_case_file(name)`:
  1. reads `fixtures/sandboxes/<name>.toml` and checks it against the registered modules' descriptors;
  2. generates it into a temporary folder with the generator's own `generate` (the "real" protection list is the macOS
     list for another temporary folder standing in for the real home), "now" the fixed `FIXTURE_NOW`;
  3. builds `sandbox_engine` with `SandboxPorts` of fakes: `FakeClock` at "now", `FakeVolumes`, `FakeOpenFiles`
     (none), `FakeRunningApps` (none), `FakeCommandRunner` (no answers), `FakeAccess`; the registry is the app's, or
     only the case file's `module`;
  4. scans; every negative: no finding at or below its paths; every positive: a finding at its `item`, in its
     `category`, with its `danger`;
  5. prepares a cleanup of every positive's finding, confirms it with the summary it got, executes it;
  6. every positive as its `after-cleanup` says; every negative's `TreeReport` the same as before; the folder that
     holds the temporary sandbox unchanged;
  7. the report — the findings (paths relative to the sandbox), the cleanup's outcome, the tree after — equals
     `fixtures/golden/sandbox-<name>.json` (`UPDATE_GOLDEN=1` rewrites it).

  Every failure names the case. One `#[test]` per case file, written by a small `macro_rules!` list.
- The same file: `every_case_file_has_a_test` (the macro's list equals `fixtures/sandboxes/*.toml`) and
  `every_registered_module_has_a_case_file` (the app's `registry()` against the files' `module`; `demo` exempt: its
  findings are made up, not on disk).
- `package.json` — `"sandbox:test": "cargo nextest run -p fresher-app --test sandboxes --locked"` (a feature name after
  `--` filters).

## Produces

The autotest of every case file, present and future.

## Checks

- Negative: `a_broken_expectation_fails_with_its_case_name` (an inline case file whose positive names another
  category; the check returns the failure, it does not panic), `a_negative_that_is_offered_fails`,
  `a_negative_that_changes_fails` (a positive whose item holds the negative's folder).
- Positive: the tests of the case files of 02.
- `npm run sandbox:test`.

## Conclusions

`cargo nextest run -p fresher-app --test sandboxes` runs every registered case file through scan, cleanup and the golden
report, registered by the `case_files!` list; `SandboxMode::only(module)` was added for it. The "not offered" check is wider
than the step text (also a finding at a folder above a negative's paths). Review findings: markers and the Trash are checked,
the failure message lists what vanished and what appeared, and two negative tests were added. Final review: the kept case
has its own marker.
