# 028/02 — Docs and finish

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** docs, sandbox, e2e

The deferred item goes, the docs say what a finding row now offers, the branch is checked end to end, and the walk by
hand is left for the user. The design: [`../00-design.md`](../00-design.md), "Docs to update" and "Testing".

## Steps

| #   | Step                                                    | Status      | Depends on | Shared files                                                                                       |
| --- | ------------------------------------------------------- | ----------- | ---------- | -------------------------------------------------------------------------------------------------- |
| 01  | [The docs](./01-docs.md)                                | 🔨 in progress | 01/03      | `docs/deferred/README.md`, `docs/architecture/README.md`, `docs/testing.md`, `docs/manual-testing.md` |
| 02  | [Checks and the walk](./02-checks-and-walk.md)          | 🔨 in progress | 02/01      | `docs/plans/028-ignore-item/`, `docs/plans/README.md`                                               |

01, then 02.

## Conclusions

The docs went in 5d771d65, the final review's fixes in 40c669ac, the checks and the conclusions in the last commit. The
checks are green; the user's walk is pending. See [`../readme.md`](../readme.md), "Conclusions".
