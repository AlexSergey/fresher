# 018/04/01 — The docs

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** docs

Every doc the plan's change affects, as built (the docs a step already updated are checked, not rewritten).

## Files

- `docs/architecture/modules.md` — `holds` and a held tag's `marker` (written by 01/01, checked); the module
  `dev-caches` as the example of a module with a data table of places and signs, a sign held by another match, and
  answers that wait for `finish`.
- `docs/architecture/README.md` — the crate list with `fresher-module-dev-caches`.
- `docs/testing.md` — `fill = "cachedir-tag"` (02/03) and the three case files (03/01), checked; the shared helpers of
  `crates/e2e/tests/support/` and `dev_caches.rs` in the e2e list.
- `docs/categories/dev-caches-*.md` — as built (rows, signs, texts); the catalogue's rows.
- `docs/ideas/developer-environment-cleaner.md` — its line on plan 018: what was built; what stays an idea (Docker,
  simulators and runtimes, device support, Archives, previews, VS Code, pnpm's store, Maven's repository, Cargo's git
  caches and index, derived data's module cache). `docs/ideas/README.md`: its row, if it names the plan's status.
- `docs/deferred/` — the shared helpers' item removed (03/02, checked).
- `docs/plans/018-developer-caches/` — every step's conclusions written as it was built (checked here); the design's
  tables brought up to what was built.

## Checks

- `npm run lint` (Markdown and links); every link of the changed docs resolves.

## Conclusions

Brought up to what was built: `docs/architecture/README.md` (the crate list names `dev-caches`);
`docs/architecture/modules.md` (`holds` and a held tag's `marker`, written by 01/01 and checked, and `dev-caches` as the
example of a module with a table of places and signs, a sign held by another match and answers that wait for
`finish`); `docs/testing.md` (`fill = "cachedir-tag"`, the case files and the e2e list, written by 02/03, 03/01 and
03/02 and checked); the nine `docs/categories/dev-caches-*.md` and the catalogue's rows (written by 02/01 and
checked against the table and the texts); `docs/ideas/developer-environment-cleaner.md` (plan 018 built; what stays an
idea); `docs/deferred/` (the shared helpers' item removed by 03/02, checked; a new item for the module harnesses'
shared code); `docs/known-issues/` (a new issue: the scanner does not count an unreadable folder inside a claimed match
as not read); the design's tables (the not-folders home, `decide`'s signature, flow 5); every step's conclusions
(written here from the implementers' reports and the review's rulings). 04/02's checklist gains the scan bench on a
quiet Mac. `docs/ideas/README.md` was checked: its row names no plan status, so it needs no change. `npm run lint` passes; every link of the changed docs resolves.
