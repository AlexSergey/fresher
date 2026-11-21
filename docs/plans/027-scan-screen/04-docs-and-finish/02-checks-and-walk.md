# 027/04/02 — Checks and the walk by hand

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** docs, sandbox, e2e

The whole branch checked, the conclusions written, and the user walks the new screens in a sandbox against the
feedback they gave. The design: [`../00-design.md`](../00-design.md), section 10.

## By the agent

- `npm run pre-push` on the branch's last commit (lint, crate and dependency guards, knip, machete, deny, the scripts'
  tests, Rust and UI coverage at or above 90 % of lines), with
  `CARGO_TARGET_DIR=/Users/serhiialeksandrov/Desktop/Work/Projects/my/github/fresher/target`.
- `npm run test:e2e --workspace app/ui` (gallery, webkit, chromium) and `npm run test:e2e:core` green; no baseline
  left that no test uses (`git status` clean after a run).
- `npm run test:perf --workspace app/ui` on `main` (183e8d3, in the main checkout) and on the branch, on this Mac: the
  Scan screen must not slow the first paint or a scan's updates by more than 5 % against the overview (rule 4); both
  numbers go in the conclusions.
- The conclusions of `docs/plans/027-scan-screen/readme.md`, `00-design.md` and each plan's readme: what was built,
  what changed from the design and why, the numbers (tests, coverage, perf), the calls taken while building, and the
  user's walk below.

## By the user (sandbox walk)

`npm run dev:sandbox -- trial` (and `npm run dev:sandbox -- app-leftovers` for the choices), then:

- [ ] Items 1, 2, 17: the sidebar holds Scan, Review, Cleanups and Settings; with the window at its smallest Settings
  is visible, and in a shorter frame (zoom in, `⌘+`) the sidebar scrolls to it.
- [ ] Items 3, 6, 13: the ring shows the disk with a gap labelled Free; the caption names the disk in a sentence; every
  legend row's "i" explains its part (Not measured lists its reasons; Free has the purgeable note when macOS reports
  one).
- [ ] Items 4, 5: hovering a segment marks its legend row and the reverse; no two segments share a colour.
- [ ] Items 7, 8, 11, 12, 14: the cards do not move on hover; they sit in Safe, Low, Medium, High, each with its
  sentence; Medium and High carry the warning icon; nothing is shown below the cards; there is no view switch.
- [ ] Items 9, 10: Details opens the category's own screen with every path and the four sentences; Back returns to
  Scan. The settings button's menu ignores the category, or its module, and Settings → Ignored restores it.
- [ ] Item 15: ticking a card or a zone's Select all shows the selection bar on Scan; Clear empties it; Review opens
  Review.
- [ ] Item 16: Review is split in the same zones with the same heads.
- [ ] Item 18: the banner names the sandbox's Trash; after a cleanup to the Trash the result names it again, and the
  items are in `sandbox/trial/home/.Trash`.

## Files

- `docs/plans/027-scan-screen/readme.md`, `00-design.md`, each plan's readme and each step — statuses and
  conclusions.
- `docs/manual-testing/2026-11-20-feedback.md` — unchanged (the raw feedback); the walk's result goes in the
  conclusions.
- `docs/known-issues/` — a file and a row for any defect found and lived with.
- `docs/plans/README.md` — the row ✔️ done once the user has looked at the result.

## Checks

- `npm run pre-push` and `npm run test:e2e --workspace app/ui` on the branch's last commit; the user has walked the
  list above and looked at the result.

## Conclusions

Checks done on 296f39fb: `npm run pre-push` green (Rust 97.48 %, UI 98.58 % of lines), Playwright 433 passed, e2e core
110 passed, `git status` clean, perf on `main` and on the branch within the 5 % budget (numbers in
[`../readme.md`](../readme.md)). The walk above is **pending**: its checkboxes stay open until the user has done it.
