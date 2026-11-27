# 044/04/03 — The user's walk and the conclusions

**Status:** ✅ approved · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** docs

The user walks the build; the agent writes what was found and the conclusions of every level. The design:
[`../00-design.md`](../00-design.md); [`../../README.md`](../../README.md), "Format" (a plan is done when its
conclusions are written and the user has looked at its result).

## Read first

Every commit of the branch; `docs/test-cases/largest-folders.md`; the `EXPECTED.md` of `largest-folders` and
`large-files`; `docs/manual-testing.md` (how a walk is written down); plan 01's measured scan time and heap.

## The walk (the user)

1. `npm run dev:sandbox -- largest-folders` and `-- large-files`: walk each `EXPECTED.md` and the cases LF-01 to
   LF-04 and HL-03.
2. `npm run dev` on the real home (a development build removes nothing outside a sandbox): LF-05; Files still ticks
   and removes a file at High (HL-03).
3. A second scan after touching one file inside a listed folder: the folder leaves the list (the rescan's newest).

## Files

- `docs/test-cases/README.md` — the walk's results are not a release's column: the cases stay empty under 0.2.0
  until the release runs them; a failure found by the walk is fixed (with its autotest) before this step closes, or
  written to `docs/known-issues/`.
- `docs/plans/044-largest-folders/readme.md`, each plan's `readme.md` and each step's file — "Conclusions": what was
  built, what changed from the design, the calls taken while building, the scan time and the heap a node, the walk's
  findings; statuses ✔️ done once the user has looked.
- `docs/plans/README.md` — 044's row: ✔️ done.

## Checks

- The user's word that the walk passed, recorded (translated) in the global plan's conclusions.
- `npm run pre-push` on the branch's last commit before the squash-merge ([`../../../workflow.md`](../../../workflow.md)).
- `cargo clean` after the squash-merge (the global plan is done).

Commit: `docs(docs): plan 044 conclusions`.
