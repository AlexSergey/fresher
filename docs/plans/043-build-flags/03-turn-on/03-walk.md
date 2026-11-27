# 043/03/03 — The user's walk and the conclusions

**Status:** ✅ approved · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** docs

The user walks the build with High and Largest on; the agent writes what was found and the conclusions of every
level. The design: [`../00-design.md`](../00-design.md); [`../../README.md`](../../README.md), "Format" (a plan is done
when its conclusions are written and the user has looked at its result).

## Read first

Every commit of the branch; `docs/test-cases/high-and-largest.md`; the `EXPECTED.md` of `app-leftovers`, `downloads`,
`large-files` and `uninstall`; `docs/manual-testing.md` (how a walk is written down).

## The walk (the user)

1. `npm run dev:sandbox -- app-leftovers`, `-- downloads`, `-- large-files`, `-- uninstall`: walk each `EXPECTED.md`
   and the cases HL-01 to HL-04.
2. `npm run dev` on the real home (a development build removes nothing outside a sandbox): Largest is in the sidebar,
   the "Only if you are sure" group appears when a module finds something at High.
3. `FRESHER_FLAGS=high=withheld,largest=hidden npm run dev`: the app is 0.1.0's again (no High group, no Largest), and
   `FRESHER_FLAGS=largest=nope npm run dev` stops the build naming the flag and the value.
4. `node scripts/release-version.ts notes 0.1.0`: the text ends with "This build" and both flags on.

## Files

- `docs/test-cases/README.md` — the walk's results are not a release's column: the cases stay "—" until the 0.2.0
  release runs them; a failure found by the walk is fixed (with its autotest) before this step closes, or written to
  `docs/known-issues/`.
- `docs/plans/043-build-flags/readme.md`, each plan's `readme.md` and each step's file — "Conclusions": what was
  built, what changed from the design, the calls taken while building, the walk's findings; statuses ✔️ done once the
  user has looked.
- `docs/plans/README.md` — 043's row: ✔️ done.
- `docs/deferred/tester-feedback-0.1.0.md` — nothing of it is taken by this plan (High and Largest were the user's own
  addition); unchanged unless the walk says otherwise.

## Checks

- The user's word that the walk passed, recorded (translated) in the global plan's conclusions.
- `npm run pre-push` on the branch's last commit before the squash-merge ([`../../../workflow.md`](../../../workflow.md)).
- `cargo clean` after the squash-merge (the global plan is done).

Commit: `docs(docs): plan 043 conclusions`.
