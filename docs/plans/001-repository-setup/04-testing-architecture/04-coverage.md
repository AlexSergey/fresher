# 001/04/04 — Coverage floors

**Status:** ✔️ done · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** workspace, app, repo

## Files

- Root script `test:cov:rust` — `cargo llvm-cov nextest --workspace --locked --fail-under-lines 90
--ignore-filename-regex '(app/tauri/src/main.rs|crates/testing/)'`, lcov report to `coverage/rust/lcov.info`.
- `app/ui/vitest.config.ts` — thresholds 90 % for lines, branches, functions, statements; excludes `main.tsx`,
  `*.d.ts`, test files, `src/ipc/mock.ts`.
- Root script `test:cov` (both); the pre-push hook gets it (step 03/03).
- `docs/quality.md` "Setting up a machine" already lists `cargo-llvm-cov` (it is not installed locally yet).

## Checks

- Deleting the core's example tests makes `npm run test:cov` fail under the floor.
