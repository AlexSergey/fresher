# 001/04 — Testing architecture

**Status:** ✔️ done · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** workspace, app, repo

Branch: `feat/001-04-testing-architecture`. The target state: [`../../../testing.md`](../../../testing.md).

## Steps

| #   | Step                                                                 | Status  |
| --- | -------------------------------------------------------------------- | ------- |
| 01  | [Rust unit and integration tests](./01-rust-unit-and-integration.md) | ✔️ done |
| 02  | [Fixtures and core e2e](./02-fixtures-and-core-e2e.md)               | ✔️ done |
| 03  | [UI unit and e2e tests](./03-ui-unit-and-e2e.md)                     | ✔️ done |
| 04  | [Coverage floors](./04-coverage.md)                                  | ✔️ done |

## Conclusions

Implemented 2026-11-15.

- Rust: 29 tests (unit, integration, property, e2e), line coverage 93.9 % over the workspace; `fresher-core` holds
  `ByteSize` and the ports `Clock`, `Trash`, `CommandRunner`; `fresher-testing` holds `FixtureTree`, `TreeReport` and
  the fakes; `fresher-e2e` checks the smoke fixture home against `fixtures/golden/smoke.json`.
- UI: 2 Vitest tests with 100 % coverage; 6 Playwright tests (smoke and visual, WebKit and Chromium) against the
  production build with the core mocked.
- Probes: a run without the `fresher-testing` tests drops line coverage to 37 % and fails the floor; a one-word change
  on the start screen (726 pixels) fails the screenshot test.
- `npm run pre-push`, `npm test` and `npm run test:e2e` pass from the root.
- Deviations from the steps:
  - **Example tests only where there is code**: `core`, `testing`, `e2e` and the shell's `app_version`; the empty
    `scanner`, `rules`, `store` and `cleaner` crates get their tests with their first code (a test of nothing proves
    nothing).
  - **The core e2e prove the harness**, not a pipeline: there is no scanner yet. The golden report is the fixture
    home read back by `TreeReport`.
  - `crates/testing` is not excluded from coverage: its fakes are tested like any code.
  - **Screenshot tolerance `maxDiffPixels: 50`**, not `maxDiffPixelRatio: 0.01`: 1 % of the frame (7 920 pixels) let a
    changed word through.
  - **IPC command keys**: `src/ipc/index.ts` maps camelCase keys to the Rust command names (`appVersion` →
    `app_version`), and the mocks use the keys; ESLint's `camelcase` rule stays strict.
  - The fixture format is documented in `docs/testing.md`, not in a README under `fixtures/` (all docs live in
    `docs/`).
  - `rustfmt.toml` got `use_small_heuristics = "Max"`: short struct literals stay on one line within 120 columns.
  - `cargo-llvm-cov` 0.9.1 was already installed.
