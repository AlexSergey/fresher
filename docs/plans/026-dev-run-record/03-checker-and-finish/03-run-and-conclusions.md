# 026/03/03 — The run by hand and the conclusions

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** docs

The user runs section 2.0 of [`docs/manual-testing.md`](../../../manual-testing.md) once on this Mac and hands over the
record; the agent checks it and writes the conclusions.

## By the user

- [ ] `npm run dev`: first the banner and one item's "a dry run" checked (otherwise stop and quit), then a scan of
  `~`, everything selected, Confirm to the Trash only (never "Delete permanently" or an emptied Trash on the real home;
  2.0, items 1 to 5): every item ends "a dry run"; nothing freed; two offered folders checked in Finder are still
  there; put back answers nothing went to the Trash.
- [ ] Quit; the record's path handed to the agent.
- [ ] `npm run dev:sandbox -- trial`, a scan, one cleanup to the sandbox's Trash, quit: a second record ending
  `_sandbox-trial.json`; the items moved for real (`ls sandbox/trial/home/.Trash`).
- [ ] With the agent: each difference of the report read and explained.

## By the agent

- `npm run dev-runs:check -- <the Mac record>` and `-- <the sandbox record>`; every difference explained: a bug
  (fixed in this branch with its test, or a known issue in `docs/known-issues/`), a rule of the checker corrected, or
  the disk changed after the scan (`changed-after-scan`).
- `grep -c '"would-' <record>` equals the number of path items the user selected and nothing in the record is
  `moved`, `emptied`, `deleted` or `tool-done` (the dry run held on the real Mac).
- The record's size and the events file's size after a full scan of the home (the risk of the plan's readme), and the
  scan's time with the recorder on against `npm exec -- tauri dev --config app/tauri/tauri.dev.conf.json` by hand (no variables, unrecorded) on the same home (the recorder must not slow the scan by
  more than 5 %, rule 4).

## Files

- `docs/plans/026-dev-run-record/readme.md`, `00-design.md` and every plan's readme — the conclusions: what was built,
  what changed from the design and why, the numbers (tests, coverage, the record's sizes, the scan's time with and
  without the recorder, the differences the checker found and what each was), the calls taken while building.
- `docs/known-issues/` — a file and a row for any defect found and lived with.
- `docs/plans/README.md` — the row ✔️ done once the user has looked at the result.

## Checks

- `npm run pre-push` on the branch's last commit; the user has looked at the result.

## Conclusions

Not written yet.
