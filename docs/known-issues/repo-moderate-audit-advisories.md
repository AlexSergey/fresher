# Moderate audit advisories in the lint tooling

**Status:** open · **Updated:** 2026-11-15

## Symptom

`npm audit` reports 9 moderate advisories, all one: `minimatch` "potential denial of service via globparts
amplification", reached through `typescript-eslint` (`@typescript-eslint/typescript-estree`), `eslint-plugin-sonarjs`
and `eslint-plugin-jsx-a11y-x`.

## Cause

Those plugins pin a `minimatch` range without the fix. They are dev dependencies: nothing of them ships in the app, and
they only read the repository's own glob patterns.

## What was tried

Nothing yet: the CI gate is `npm audit --omit=dev --audit-level=high`, which passes.

## Options left

- Wait for the plugins to raise their `minimatch` range (checked with each dependency update).
- An `overrides` entry pinning the fixed `minimatch` for those parents, if a plugin stalls; it must be removed when the
  plugin catches up.
