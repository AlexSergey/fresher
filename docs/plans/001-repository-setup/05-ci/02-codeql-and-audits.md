# 001/05/02 — CodeQL and audits

**Status:** ✅ approved · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** ci

## Files

- `.github/workflows/codeql.yml` — prod-forge's, with languages `javascript-typescript` and `rust`; on pull requests,
  on `main` and weekly.
- `ci.yml` `lint` job: `npm audit --omit=dev --audit-level=high` (prod-forge's gate) and `cargo deny check advisories`
  (already part of `check:deny`).
- `actionlint` run over `.github/workflows/` in the `lint` job.

## Checks

- CodeQL runs on the plan's pull request and reports no alert.
