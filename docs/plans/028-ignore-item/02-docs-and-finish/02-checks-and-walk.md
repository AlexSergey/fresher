# 028/02/02 — Checks and the walk by hand

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** docs, sandbox, e2e

The whole branch checked, the conclusions written; the walk in a sandbox is the user's, left pending (the user checks
plans 027, 028, 023 and 024 together, the delegation of 2026-11-20).

## By the agent

- `npm run pre-push` on the branch's last commit (lint, guards, knip, machete, deny, the scripts' tests, Rust and UI
  coverage at or above 90 % of lines), with
  `CARGO_TARGET_DIR=/Users/serhiialeksandrov/Desktop/Work/Projects/my/github/fresher/target`.
- `npm run test:e2e --workspace app/ui` (gallery, webkit, chromium) and `npm run test:e2e:core` green; `git status`
  clean after a run (no baseline left that no test uses).
- `git diff 951a8e79 -- crates modules app/tauri` is empty: no core, shell or removal path changed.
- `npm run test:perf --workspace app/ui` on plan 027's head (951a8e79) and on the branch: the 100,000-row scroll and
  "a category's details after a scan" within rule 4's 5 %; both numbers in the conclusions.
- The conclusions of `docs/plans/028-ignore-item/readme.md`, `00-design.md` and each plan's readme and step: what was
  built, what changed from the design and why, the numbers, the calls taken while building.

## By the user (sandbox walk, pending)

`npm run dev:sandbox -- trial`, scan, then a category's Details:

- [ ] Every row has its settings button at its end; its menu says "Ignore this item" with its sentence.
- [ ] Choosing it takes the row out at once; the next row has the focus; the count on the screen follows.
- [ ] By keyboard alone: Tab to the list, the arrows to a row, Tab to its menu button, ArrowDown, Enter.
- [ ] Ignoring the last row of a category leaves the focus on "Back to Scan".
- [ ] Settings → Ignored → Ignored items lists the path; Restore brings the row back without a rescan.
- [ ] Review's rows have no menu; a cleanup prepared after the ignore does not list the item.

## Files

- `docs/plans/028-ignore-item/readme.md`, `00-design.md`, each plan's readme and each step — statuses and
  conclusions.
- `docs/known-issues/` — a file and a row for any defect found and lived with.
- `docs/plans/README.md` — the row 🔨 in progress while the walk is pending; ✔️ done once the user has looked.

## Checks

- `npm run pre-push` and `npm run test:e2e --workspace app/ui` on the branch's last commit; the user has walked the
  list above and looked at the result.

## Conclusions

The agent's checks are green (numbers in [`../readme.md`](../readme.md), "Conclusions"): pre-push, 449 Playwright
tests, the core e2e, an empty diff of `crates`, `modules` and `app/tauri` against 951a8e79, perf within rule 4's 5 %
(the WebKit jitter shows on the base too). The user's walk above is **pending**: its boxes stay unchecked.
