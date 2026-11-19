# 026/03 — Checker and finish

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** scripts, docs

The agent's tool that reads a record and checks it against the disk, read-only; the checklist of runs by hand
rewritten around one recorded run; the run itself and the conclusions. No version bump: the version stays 0.1.0
([`../../README.md`](../../README.md), "Versions"). The design: [`../00-design.md`](../00-design.md), section 4
(agent's calls 12, 13 and 18).

## Steps

| #   | Step                                                             | Status      | Shared files                                                     |
| --- | ---------------------------------------------------------------- | ----------- | ---------------------------------------------------------------- |
| 01  | [The checker](./01-checker.md)                                   | ✔️ done     | `package.json`                                                   |
| 02  | [The checklist around the record](./02-checklist.md)             | ✔️ done | `docs/manual-testing.md`, `docs/testing.md`                      |
| 03  | [The run by hand and the conclusions](./03-run-and-conclusions.md) | ✅ approved | `docs/plans/`, `docs/known-issues/` (if the run finds one)     |

01 runs beside plan 02 (it reads the shape of the design, `format` 1); 02 after plan 02 and 01; 03 last.

## Conclusions

Not written yet.
