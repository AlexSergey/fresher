# 035/04/03 — The UI e2e

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** app/ui

The Largest screen in the browser against the e2e core mock: the flows, the visual baselines and the accessibility
checks. The mock answers only what the core can answer together (the global constraint on mocks). The design:
[`../00-design.md`](../00-design.md), sections 8 and 9.

Read first: `app/ui/e2e/core-mock.ts` (`SCRIPTS`, how a script answers a command; `appCaches` is a recent model),
`app/ui/e2e/support/` (the cleanup mock), `app/ui/e2e/flows.spec.ts` (the negative and positive blocks),
`app/ui/e2e/visual.spec.ts`, `app/ui/e2e/axe.spec.ts`, `app/ui/src/test/largest-scan.ts` (03/01), `docs/testing.md`
("Where things live": the e2e and the baselines).

## Files

- `app/ui/e2e/core-mock.ts` — a script `largest`: a completed scan (any modules the mock already has), its coverage
  with Documents not entered, `getLargest` answering `largest-scan.ts`'s page (filtered by kind, place and unchanged
  as the core does, by the same rows: a small pure helper in the mock, tested by `core-mock.spec.ts`),
  `getLargestPlace` answering the home's folders (`Movies` withheld `standard-folder` and opening to one file,
  `Library` withheld `library`, `VMs` offered) and one folder's page, `pickOwnItem` answering each row's
  `core.own-file` finding (High, the core's texts), and the cleanup: the picked items `trashed`, then put back. No
  answer the core could not give: a withheld row is never picked, a pick answers the same finding twice.
- `app/ui/e2e/flows.spec.ts` — negatives first:
  - negative: "nothing on Largest is ticked when it opens"; "a withheld row cannot be ticked and says why"; "a
    standard folder opens but cannot be chosen"; "the folders Fresher did not see are named above the list";
  - positive: "Largest lists the largest files with kind, place and size"; "a filter narrows the list";
    "an archive names the folder beside it"; "the folders view opens a folder and goes back by its way";
    "a ticked file is reviewed under Your own files at High"; "a picked file goes to the Trash and leaves the list";
    "a picked file comes back by put back".
- `app/ui/e2e/visual.spec.ts` and `axe.spec.ts` — Largest's Files view (with a withheld row and the coverage notice)
  and its Folders view, in both themes as the file does for other screens.
- `app/ui/e2e/screenshots/` — the new images, looked at before committing; the Settings baseline updated if the new
  setting `core.largest.min-size` shows in the mock's settings (only if the mock lists it; the mock answers the core's
  settings list for the scan it plays).

## Checks

- `npm run test:e2e:ui -- --project chromium --project webkit` (or the app's documented way to run the flows in both
  browsers); the new images looked at; `git status` clean after a second run.
- Commit: `test(ui): the Largest screen's flows, baselines and accessibility`.

## Conclusions

- `app/ui/e2e/support/largest-mock.ts` answers the three Largest commands from the scripted scan; `SCRIPTS.largest`
  states only what the core can produce together. Four negative and seven positive flows, the visuals `largest`,
  `largest-folders` and `largest-review`, and the axe check of the Folders view, in both browsers; every baseline looked
  at.
- The confirmation has no separate High tick yet (its only checkbox is "Delete permanently instead"): the flow
  asserts that the dialog names High with the own-file texts, nothing pre-chosen, Cancel focused, and the Move to Trash
  confirmation for that exact set. A separate High tick is plan 039.
- The scan's not-entered folder is the Desktop (not Documents), since the Files list has an archive in Documents.
- The Docker baselines (scan-described, insight-answered, insight-stopped) were stale since the Largest sidebar item
  landed on plan 032: refreshed. Playwright, both browsers: 706 passed, 1 failed (the known flaky chromium baseline
  category-details-app-leftovers-chosen).
