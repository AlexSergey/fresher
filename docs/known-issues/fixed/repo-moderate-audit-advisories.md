# Moderate audit advisories in the lint tooling

**Status:** fixed (an `overrides` entry gives the three plugins the `minimatch` that `eslint` uses; to be removed when they catch up) · **Updated:** 2026-11-26

## Symptom

`npm audit` reports 9 moderate advisories, all one: `minimatch` "potential denial of service via globparts
amplification", reached through `typescript-eslint` (`@typescript-eslint/typescript-estree`), `eslint-plugin-sonarjs`
and `eslint-plugin-jsx-a11y-x`.

## Cause

Those plugins pin a `minimatch` range without the fix. They are dev dependencies: nothing of them ships in the app, and
they only read the repository's own glob patterns.

## What was tried

The CI gate is `npm audit --omit=dev --audit-level=high`, which passes. Waiting for the plugins to raise their range
did not help: they stalled.

## Fix

The root `package.json` has `overrides` for `@typescript-eslint/typescript-estree`, `eslint-plugin-jsx-a11y-x` and
`eslint-plugin-sonarjs`: `minimatch` `^11.0.8`. `npm audit` reports 0 vulnerabilities; the lint, the type checks and
knip pass. The override is to be removed when the plugins raise their `minimatch` range; it is checked at each
dependency update (see `docs/dependencies.md`).
