# 001/05/01 — CI workflow

**Status:** ✔️ done · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** ci

## Files

- `.github/workflows/ci.yml` — modelled on prod-forge's `ci.yml`: on `pull_request` and push to `main`,
  `permissions: contents: read`, concurrency group with `cancel-in-progress`, release commits skipped; runner
  `macos-latest`; Node from `.nvmrc`, Rust from `rust-toolchain.toml`; npm and cargo caches (`Swatinem/rust-cache`);
  cargo tools installed with `taiki-e/install-action`.
  - `lint`: `npm ci`, `npm run lint`, `npm run knip`, `npm run check:machete`, `npm run check:deny`, `typos`.
  - `test` (needs `lint`): `npm run test:cov`, coverage reports uploaded as artifacts.
  - `e2e` (needs `test`): `npm run test:e2e` with Playwright browsers installed; the report uploaded on failure.
  - `build` (needs `e2e`): `npm run build` (unsigned bundle).

## Checks

- The plan's pull request runs all four jobs green; a deliberate Clippy warning on a test branch turns `lint` red.
