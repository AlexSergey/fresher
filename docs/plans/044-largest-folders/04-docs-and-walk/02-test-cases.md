# 044/04/02 — The test cases

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** docs

What only a person can see of Folders and Browse (the Finder opening, a real home's list), written as cases every release runs
again. The design: [`../00-design.md`](../00-design.md), sections 8 and 10.

## Read first

04/01's commit; `docs/test-cases/{README,high-and-largest}.md` (the form, the results table);
`fixtures/sandboxes/largest-folders.toml` and its `EXPECTED.md`; `docs/manual-testing.md`.

## Files

- `docs/test-cases/largest-folders.md` (new) — setup: a development build in sandbox mode (`npm run dev:sandbox --
  largest-folders`), then a real home for LF-05; the cases, each with its steps and what must happen:
  - **LF-01 The folders left alone** (`largest-folders`): after a scan, Largest's Folders shows the heading, the rule
    line (500 MB, 365 days), the bar, the tiles (3 folders, about 1.6 GB, 1 project) and the rows
    `~/Code/old-app` (Project), `~/Archive`, `~/Public/old-export`, largest first, each "Not changed for 13 months";
    `~/Code`, `~/Archive/2019`, `~/Public`, `~/Small`, the ignored `~/Old-tools` and the copied `~/Restored` are not
    listed; no row has a checkbox.
  - **LF-02 Show in Finder** (`largest-folders`): Show in Finder on `~/Code/old-app` opens a Finder window with the
    folder selected; nothing is ticked, the selection bar does not change, the folder stays on disk.
  - **LF-03 The settings** (`largest-folders`): Settings, Scan, "Smallest folder on Largest" set to 1 GB empties the
    list (its empty text names 1 GB and 365 days); back to 500 MB lists the three again without a scan.
  - **LF-04 Browse is read-only** (`large-files`): Browse lists the home's folders; Open goes into one and the
    breadcrumb comes back; no row has a checkbox; Show in Finder on a file and on a folder opens the Finder with it
    selected; `~/Public` says it is a standard folder and still opens.
  - **LF-05 A real home** (`npm run dev`): the list holds no standard folder of the home, nothing in `~/Library`, no
    app and no hidden folder; the rows' ages agree with the Finder's "Date Modified" of the newest file inside one
    row picked by hand.
- `docs/test-cases/high-and-largest.md` — HL-03: "a standard folder on Folders cannot be ticked and says why" becomes
  "Folders and Browse have no checkbox (LF-01, LF-04)".
- `docs/test-cases/README.md` — the file's row in "Files"; rows LF-01 to LF-05 in "Results", "—" under 0.1.0, empty
  under 0.2.0.

## Interfaces produced

- The cases 04/03's walk runs and the 0.2.0 release runs again.

## Tests

None (documentation). Each case's expectations are checked against `largest-folders`' `EXPECTED.md` and the e2e
specs of 03/04, so a case asks nothing the app does not do.

## Checks

- `npm run check:readme`, `npx prettier --check docs/test-cases`
- `npm run dev:sandbox -- largest-folders` once by the agent to confirm LF-01's names, sizes and ages match the
  screen (the walk is the user's, 04/03)

Commit: `docs(docs): test cases for Largest's folders`.
