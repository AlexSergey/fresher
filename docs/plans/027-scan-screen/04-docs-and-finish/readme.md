# 027/04 — Docs and finish

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** docs, sandbox, e2e

The docs say what the UI now is, the core's place queries left without a UI get their deferred row, and the plan is
checked end to end and walked by hand in a sandbox. The design: [`../00-design.md`](../00-design.md), sections 9 and
10 (agent's call 15).

## Steps

| #   | Step                                                | Status      | Depends on | Shared files                                                                                  |
| --- | --------------------------------------------------- | ----------- | ---------- | --------------------------------------------------------------------------------------------- |
| 01  | [The docs](./01-docs.md)                            | 🔨 in progress | 03/03      | `docs/architecture/README.md`, `docs/manual-testing.md`, `docs/testing.md`, `docs/deferred/README.md` |
| 02  | [Checks and the walk by hand](./02-checks-and-walk.md) | 🔨 in progress | 04/01   | `docs/plans/027-scan-screen/`, `docs/plans/README.md`                                          |

01, then 02.

## Conclusions

- **04/01** the docs match the Scan screen: architecture, testing, manual-testing, the deferred item
  `place-queries-without-ui`, plans 012 and 015 note what the UI no longer shows.
- **04/02** the whole branch checked on the last commit (numbers in [the plan's conclusions](../readme.md)); the
  conclusions are written; one known issue recorded, [`ui-perf-webkit-jitter`](../../../known-issues/ui-perf-webkit-jitter.md).
  The walk by hand is the user's and **pending**, so the plan stays 🔨 until the user has looked.
