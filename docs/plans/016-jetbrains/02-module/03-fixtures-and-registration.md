# 016/02/03 — Fixture tests and registration

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
modules, app

## Files

- `fixtures/homes/jetbrains-current.toml`, `jetbrains-similar-names.toml`, `jetbrains-running.toml`,
  `jetbrains-uninstalled-fresh.toml`, then `jetbrains-stale.toml` and `jetbrains-uninstalled.toml` (new; the two
  homes of a removed IDE added by 02/04, their folder ages with `[[dir]]` `age`) — the homes of the design, section 8, "Fixture
  tests"; the apps are given to `Ports` as `FakeInstalledApps` and `FakeRunningApps` (the homes hold only files).
- `modules/jetbrains/tests/fixtures.rs` (new) — `run_module` on each home, negatives first: nothing offered and the
  expected left-outs (`ModuleRun::left_out`); then the stale home's findings, their danger levels, their apps, their
  removals. The golden report `fixtures/golden/jetbrains.json` (`UPDATE_GOLDEN=1`, reviewed).
- `modules/jetbrains/tests/contract.rs` — `assert_texts_used(&module, &runs)` over every fixture run (every left-out
  reason is used by one), `check_removals_matched(&runs)` empty.
- `app/tauri/Cargo.toml` and `app/tauri/src/compose.rs` — `JetBrainsModule::new()?` registered in every build, first in
  `modules()`; `ComposeError` gains the module's error; the composition test lists `jetbrains` in a release build too.
- `Cargo.lock` (regenerated).

## Produces

A registered module; the guard "every module the app registers has a case file" now fails until 03/01 adds
`jetbrains.toml`: this step and 03/01 land on the branch back to back, or 03/01 first if plan 01 is ready
(written in the conclusions).

## Checks

- `cargo nextest run -p fresher-module-jetbrains -p fresher-app`; coverage of the crate at or above 90 %
  (`cargo llvm-cov nextest -p fresher-module-jetbrains`).
- The module checklist of [`architecture/modules.md`](../../../architecture/modules.md) ticked, box by box, in the
  conclusions.

## Conclusions

Built: six fixture homes `fixtures/homes/jetbrains-*.toml` (ages by `[[dir]] age` from the fixture's now), 12
negative and 5 positive fixture tests, the golden `fixtures/golden/jetbrains.json`, the contract test with
`assert_texts_used` over 11 runs and a check that the reasons seen equal the eight declared. `JetBrainsModule` is
registered first in `modules()` in every build. Coverage of the crate: 98.0 % of lines, 100 % of functions.
