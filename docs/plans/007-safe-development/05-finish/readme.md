# 007/05 — Finish

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** docs, repo

The docs as built, the rule every later plan follows, a run by hand in a sandbox, the conclusions.

After every other plan.

## Steps

| #   | Step                                                          | Status   | Shared files                                                     |
| --- | ------------------------------------------------------------- | -------- | ---------------------------------------------------------------- |
| 01  | [The docs and the rule](./01-docs-and-rule.md)                | ✅ approved | `docs/` (see the step)                                           |
| 02  | [A run by hand, the conclusions](./02-run-and-conclusions.md) | ✅ approved | `docs/plans/README.md`, `docs/deferred/` |

In order.

## Conclusions

**Built:** 05/01 wrote the docs (sandbox, case file rule, the known issue and the deferred panel). 05/02 prepared the run
by hand; it waits for the user.

**Measured:** after the final fix wave `npm run pre-push` passes (1694 Rust tests, 4 skipped, lines at or above 90 %; UI
lines 98.36 %, branches 92.05 %), and `npm run test:e2e` passes (core 34, 1 skipped; UI 225).

**Left:** the run by hand (the user's, 05/02).

**Agent's calls taken during the build (to check):**

- No version bump anywhere; the version stays 0.1.0 (the user's decision of 2026-11-16 overrides any older number left in a step). Costs, if wrong: nothing, a version is set later.
- 05/02 runs only the release-gating tests under `--release`: the demo and trial modules exist in debug builds only, so 12 older tests of `fresher-app` cannot pass in release. Costs, if wrong: none, they are debug tests.
- The scan offering an item inside `~/.Trash` (then left out as protected) is recorded as a known issue, not fixed here. Costs, if wrong: one confusing finding stays until the product question is decided.
