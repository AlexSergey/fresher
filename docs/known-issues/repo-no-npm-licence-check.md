# No licence check of npm dependencies yet

**Status:** open · **Updated:** 2026-11-15

## Symptom

[`../restrictions/dependencies.md`](../restrictions/dependencies.md) says the licences of npm production dependencies
are checked, but no script or CI job does it: a new npm package with a licence outside the allowlist would pass every
hook and the CI. Rust is covered by `cargo deny check`.

## Cause

The check was described when the restriction was written and never built. Found on 2026-11-15 while adding
`lucide-react` in plan 003/01.

## What was tried

`lucide-react` (ISC, no dependencies) was checked by hand before it was added.

## Options left

- Build the check together with the licence collector of [`../deferred/licence-notices.md`](../deferred/licence-notices.md),
  which already lists "a check in CI that fails on a licence outside the allowlist, for npm as for Rust".
- Until then, every new npm package is checked by hand (its licence and those of its production dependencies) and the
  result is written in the commit or the plan that adds it.
