# 037/03/03 — The UI e2e

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** app/ui, e2e

The Trash line and the result's invitation in Playwright, over the core mock: the flows, the visual baselines and the
accessibility checks, in chromium and WebKit. The mock answers only what the core can answer together (a share never
above `waiting`, a line only when something waits, `goneBy` only with a share). The design:
[`../00-design.md`](../00-design.md), section 5.

Read first: `app/ui/e2e/core-mock.ts` (the scripts and `answers`), `app/ui/e2e/support/cleanup-mock.ts` (what a
cleanup answers, `reveal_in_trash`), `app/ui/e2e/flows.spec.ts` (negative and positive blocks),
`app/ui/e2e/visual.spec.ts`, `app/ui/e2e/axe.spec.ts`, step 02/01's texts.

## Files

- `app/ui/e2e/core-mock.ts` — `get_trash` answered from the script (`trash?: TrashView`, default `{ trashes: [] }`),
  and after a script's cleanup finishes, from `trashAfterCleanup` when the script gives one; `open_trash` records its
  index where the tests can read it (as the mock records other actions) and refuses an index the last answer did not
  have with the core's `unknown-trash` refusal. Two sample views beside the other samples: one line (23 GB waiting,
  20 GB from cleanups, no date) and two lines (`Macintosh HD` and `Backup`, the second not measured), each written so
  `waiting >= fromCleanups`.
- `app/ui/e2e/flows.spec.ts` — `negative cases`:
  - "an empty Trash shows no Trash line";
  - "the Trash line offers no way to empty the Trash" (no button but Open Trash in the line; no button anywhere on
    the scan screen or the result whose name matches `/empty/i`);
  - "a Trash that was not measured says so and shows only the share";
  `positive cases`:
  - "the scan screen shows what waits in the Trash and how much is from cleanups here" (the exact sentence of step
    02/01 with the formatted sizes);
  - "Open Trash asks the core to open that Trash" (the recorded index);
  - "after a cleanup to the Trash the line shows the moved items" (`trashAfterCleanup`, the line's new sentence once
    the result is closed);
  - "the result invites the user to look at the Trash and empty it themselves";
  - "two Trashes show a line each with their volume's name";
  - "Finder's 30-day setting shows the date the share leaves by itself".
- `app/ui/e2e/visual.spec.ts` — the scan screen with the one-line Trash and with two lines; the result with its
  invitation (if the result's baseline exists, it is updated, not duplicated). `app/ui/e2e/screenshots/`: the new and
  changed images, each looked at before committing (light and dark, as the suite takes them).
- `app/ui/e2e/axe.spec.ts` — the scan screen with the Trash line: no violation.

## Checks

- `npm run test:e2e -- --project chromium --project webkit` (or the repository's e2e script) green; a flake is run
  again alone and named in the conclusions.
- `npm run lint` on the changed files.
- `git status` clean after the run (no stray screenshot).

## Conclusions

Done. The mock answers `get_trash` from the script (an empty view by default) and, after a cleanup, from the
cleanup script's `trashAfterCleanup`; `open_trash` records the index (`__openedTrash`) and refuses an unlisted index
with `unknown-trash`. Samples live in `app/ui/src/test/trash-view.ts`. Ten flows, four negative and six
positive ones, two visual baselines (one and two lines, dark, both browsers) and two axe checks were added. Full
suite: 624 passed; the known chromium flake `category-details-app-leftovers-chosen` fails also alone, untouched.
