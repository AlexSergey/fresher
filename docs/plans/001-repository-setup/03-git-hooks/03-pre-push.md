# 001/03/03 — Pre-push

**Status:** ✔️ done · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** repo

Depends on plan 04 for `test:cov`; until then the hook runs the lint part only, and 04/04 adds the coverage line.

## Files

- Root script `pre-push`: `npm run lint && npm run knip && npm run check:machete && npm run check:deny && npm run
test:cov`.
- `.husky/pre-push` — `npm run pre-push`.

## Checks

- A push with a Clippy warning, an unused npm dependency or an unused crate dependency fails.
- After plan 04: a push that drops coverage under 90 % fails.
