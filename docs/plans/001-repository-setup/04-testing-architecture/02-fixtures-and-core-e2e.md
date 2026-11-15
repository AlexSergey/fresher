# 001/04/02 — Fixtures and core e2e

**Status:** ✅ approved · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** workspace, fixtures

## Files

- `crates/testing/src/fixture_tree.rs` — `FixtureTree::from_toml(path)` builds a fake home folder in a `TempDir`:
  folders, files with a declared size (sparse, so they cost no disk), symlinks, modification times (set from the
  description, never from the wall clock). The temporary folder is the only root a test may scan or clean.
- `fixtures/homes/empty.toml` and `fixtures/homes/smoke.toml` (a `~/Library/Caches/<app>` folder, a symlink pointing
  out of the root, a look-alike folder) with a short README of the format in
  [`../../../testing.md`](../../../testing.md).
- `fixtures/golden/smoke.json` — the expected report of the skeleton pipeline.
- `crates/e2e/tests/smoke.rs` — builds the fixture, runs the skeleton pipeline end to end, compares with the golden
  file; `UPDATE_GOLDEN=1` rewrites it (reviewed in the diff).
- Root script `test:e2e:core` (`cargo nextest run -p fresher-e2e --locked`).

## Checks

- The fixture builder's own tests: the symlink is not followed, declared sizes are reported as logical sizes while the
  disk usage stays near zero.
- `npm run test:e2e:core` passes; a change to the golden file fails it.
