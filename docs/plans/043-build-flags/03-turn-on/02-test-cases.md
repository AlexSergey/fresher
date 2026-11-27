# 043/03/02 — The test cases

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** docs

The High parts of plan 042's walk, written as test cases by hand that every release runs again (the draft's
decision): the "Only if you are sure" group, the High tick in the confirmation, Largest's ticks, an uninstall of
everything. The design: [`../00-design.md`](../00-design.md), section 8.

## Read first

03/01's commit; `docs/test-cases/README.md` (the files, the results table) and `docs/test-cases/sandbox-cleanup.md`
(a setup in a sandbox with the system's Trash, the case form); `fixtures/sandboxes/{app-leftovers,downloads,
large-files,uninstall}.toml` and their `EXPECTED.md`; `docs/restrictions/deletion-safety.md`, "A High plan needs its
own tick"; `docs/manual-testing.md`.

## Files

- `docs/test-cases/high-and-largest.md` (new) — setup: a development build in sandbox mode (`npm run dev:sandbox --
  <feature>`, the feature named per case), the sandbox's own Trash; then the cases, each with its steps and what must
  happen:
  - **HL-01 The "Only if you are sure" group** (`app-leftovers`): after a scan the group lists the removed app's data
    folder, unticked; "Select all" is not offered for the group; its details say why it is High.
  - **HL-02 The High tick in the confirmation** (`app-leftovers`): with the data folder ticked, Review then Remove
    opens the confirmation with its own High tick unchecked and the remove button disabled until it is ticked; ticked,
    the folder goes to the sandbox's Trash and Cleanups lists it; put back returns it.
  - **HL-03 Largest's ticks** (`large-files`): Largest is in the sidebar; Files lists the sandbox's large files
    largest first; ticking one and removing it asks the High tick (HL-02's dialog) and moves it to the Trash; a
    standard folder on Folders cannot be ticked and says why.
  - **HL-04 Uninstall everything** (`uninstall`): an app's page offers "Keep my settings" and "Everything"; choosing
    Everything lists its data with the app and asks the High tick; the app and its data go to the Trash.
- `docs/test-cases/README.md` — the file's row in "Files"; rows HL-01 to HL-04 in "Results" with "—" under 0.1.0 (not
  run: High was withheld) and a new column 0.2.0, empty until the release's run.

## Interfaces produced

- The cases 03/03's walk runs and the 0.2.0 release runs again.

## Tests

None (documentation). Each case's expectations are checked against the case file's `EXPECTED.md` and the e2e specs
that already hold the same screens with a mocked core (`flows.spec.ts`'s High group and confirmation,
`visual.spec.ts`'s `cleanup-confirm-high*.png`), so a case asks nothing the app does not do.

## Checks

- `npm run check:readme` (the links of `docs/`), `npx prettier --check docs/test-cases`
- `npm run dev:sandbox -- app-leftovers` once by the agent to confirm HL-01's names match the screen (the full walk is
  the user's, 03/03)

Commit: `docs(docs): test cases for High and Largest`.
