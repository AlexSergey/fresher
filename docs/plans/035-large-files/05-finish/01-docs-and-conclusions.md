# 035/05/01 — Docs and conclusions

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** docs

Every doc the change affects, updated as built; the deferred item of the place queries closed and the new one written;
the automated checks run on the branch's last code commit; the conclusions of every level written as built, the
statuses 🔨 "awaiting the user's walk"; and this plan's section of the user's review list. The sandbox walk and the
look at the real Mac are the user's: their boxes stay unchecked.

Read first: `CLAUDE.md` ("Documentation"), `docs/plans/README.md` ("Format", the priority list), the conclusions of
plan 030 (`docs/plans/030-installed-app-caches/readme.md`) as the model of conclusions written before the user's walk,
`docs/deferred/README.md`, `docs/deferred/user-review-2026-11-22.md` (its format: one section per plan), every step's
conclusions of this plan.

## Files

- `docs/architecture/README.md` — "Queries": `items` beside `largest_files`; "The storage map": the place view's UI is
  back as Largest → Folders (the sentence about plan 027 and the deferred item replaced); a short section "The largest
  files" (plan 035): the two queries, the judgement and its reasons, the items kept per state and minimum, the pick as
  a `core.own-file` finding held apart from the scan's (never in totals, cards, the map or pages), the commands
  `get_largest`, `get_largest_place`, `pick_own_item` and the removal of `get_storage_place`; "UI": the screen
  `screens/largest/`, the component `product/own-item-row/`, the sidebar's entries (Scan, Largest, Review, Cleanups,
  Settings), `Screen` `largest`.
- `docs/architecture/removal.md` — one sentence where the plan builder's inputs are described: a picked own file joins
  the plan as a picked finding, nothing else changes.
- `docs/categories/README.md` and `core-own-file.md` (02/01) — checked against the texts as built.
- `docs/testing.md` — checked: the case file, `[case.largest]`, the core e2e flow and the budget command (04/01,
  04/02).
- `docs/manual-testing.md` — a section "1.20 The largest files ([035/05/01](./plans/035-large-files/05-finish/01-docs-and-conclusions.md))":
  `rm -rf sandbox/large-files && npm run dev:sandbox -- large-files`, walk `EXPECTED.md` line by line, negatives first;
  the Files list, the filters, the Folders view and its way back, a withheld row's sentence, the Documents notice;
  tick the disk image, the package, the archive and the folder; Review shows "Your own files" at High; confirm to the
  Trash; the rows leave; Cleanups → put back; rescan: they are listed again. And under "2. The real Mac" (item 2.18): open Largest
  after a scan of `~` (a development build is a dry run): the ten largest rows checked against
  `find ~ -xdev -size +100M -not -path "$HOME/Library/*" -not -path "*/.*" 2>/dev/null | head -50` and `du -sh`; no
  row under `~/Library`, the Trash, a hidden folder or an app; a module's folder is not listed.
- `docs/deferred/place-queries-without-ui.md` — removed, with its row in `docs/deferred/README.md` (closed by this
  plan: the view is back; the "way up" is the breadcrumb; the roots' explained sizes are not needed since the roots
  list did not return).
- `docs/deferred/largest-files-more.md` (new) and its row — **Agreed** 2026-11-22 by the user's delegation · **Due
  at** after plan 035's walk on the developer's Mac; what waits, with what each needs: "last opened" (Spotlight's
  last-used date through a port, per page, and the access time kept for kept files, measured; the facts of this
  design's table), "the app that made it" (Spotlight's creator or the package's handler, a port), a treemap of the same
  rows, a sync client's folder marker in the home, places outside the home (other scan roots, volumes), a folder's
  "Changed" from its newest file ([`folder-age-from-children.md`](./folder-age-from-children.md)).
- `docs/ideas/README.md` — the idea "Large Files Finder", if it still has a row or a file, marked as replaced by plan
  035 (or removed, as the README says for an idea that became a plan).
- `docs/plans/README.md` — the row of 035: 🔨 in progress.
- `docs/plans/035-large-files/readme.md`, `00-design.md` and every plan's and step's "Conclusions" — written as built:
  what was built, what changed from the design and why, the numbers (Rust and UI tests and coverage, the items budget
  on `LARGE` from 04/01, `npm run sandbox:test`, Playwright in both browsers), the by-hand mutation checks of 04/02,
  the calls taken while building; statuses 🔨 in progress, **awaiting the user's walk**.
- `docs/deferred/user-review-2026-11-22.md` — append a section `## 035 — The largest files`, with plan 035's link
  and "awaiting the user's walk", short bullets:
  - **Walks by hand**: the sandbox walk above; the real-Mac look above.
  - **Facts not verified**: whether Spotlight's last-used date and the access time behave on another Mac as on this
    one (checked here on macOS 27.2.1); whether any app keeps a large, hidden-free folder in the home that should not
    be offered (a legacy sync client's folder); how many items a real home holds over 100 MB and whether 10 000 is
    enough; the items query's time on the real home.
  - **Rulings worth a second look**: the design's agent's calls 2 (a core category for picked rows), 3 ("Changed"
    instead of "last opened"), 5 (ranked by allocated size), 7 (removability left to the rehearsal), 8 (no sync
    marker), 9 (hidden names never offered), 12 (a claimed place skipped even when ignored), 13 (10 000 items).
  - **Known gaps**: [`largest-files-more.md`](./largest-files-more.md); the folders' sizes after a cleanup stay the
    scan's until a rescan; a folder's "Changed" is its own time.

## Checks

- `npm run pre-push` on the branch's last commit (lint, guards, knip, machete, deny, script tests, tests with
  coverage ≥ 90 % in Rust and the UI); Playwright in both browsers; `npm run sandbox:test`.
- `git diff <base> -- crates/cleaner crates/scanner/src/walk crates/scanner/src/codec crates/scanner/src/rescan` empty.
- `npm run check:repo` (links, the docs' rules) passes; no product or reference project named.
- Commit: `docs(docs): plan 035 conclusions and the user's review list`.

## Conclusions

- Docs updated as built: `architecture/README.md` (Queries, the storage map, the new section "The largest files", UI),
  `architecture/removal.md`, `categories/` (core-own-file, checked), `testing.md` (checked), `manual-testing.md`
  (sections 1.20 and 2.18, the table row), `deferred/largest-files-more.md` (new, with its row),
  `deferred/place-queries-without-ui.md` (removed: closed by this plan), `plans/README.md` (🔨), and the user's review
  section in `deferred/user-review-2026-11-22.md`. `ideas/` held no idea of its own for this plan (the opportunities
  table already links plan 035).
- Rebased onto plan 038's merge (050f2878): conflicts only in the dev-run action list, the bindings test, the sandbox
  crate (`case.rs`, `expected.rs`, `lib.rs`, `world.rs`) and the demo test harness, each resolved keeping both.
- Checks on the last commit: see the global readme's table. The sandbox walk and the look at the real Mac are the
  user's; their boxes stay unchecked.
