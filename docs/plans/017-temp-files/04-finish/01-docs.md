# 017/04/01 — The docs

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** docs

Every doc the plan's change affects, as built (the docs a step already updated are checked, not rewritten).

## Files

- `docs/architecture/modules.md` — `ModuleGroup::System`; the two known folders and that a known folder outside the
  scan roots is walked as a place (only for the interests that name it); `Match.changed`, `accessed`, `inside` and
  `EntryKind::Other` in "What a module sees of the walk"; the module `temp` as the example of a module that judges by
  age and refuses by kind and owner.
- `docs/architecture/README.md` and `removal.md` — the places in the walk's roots and in the cleaner's check 2; check 7
  refusing `Other`.
- `docs/restrictions/deletion-safety.md` — "Unknown is not junk": the one exception of decision 6 (a top-level item of
  the user's own temporary folders, claimed by the place's contract, only after every row of the module's decision);
  "Never follow symlinks out, never touch protected paths": the new protected places and the two whole ones.
- `docs/testing.md` — the stated facts and the lens (if 01/05 left anything), the `temp.toml` case file in the list of
  case files.
- `docs/categories/temp-old-files.md` — as built; the catalogue row.
- `docs/known-issues/fixed/core-fifo-read-as-file.md` — fixed (01/02), checked.
- `docs/deferred/sandbox-world.md` — the stated file facts built here; open files, tools, access and the steps between
  scans still deferred; its README row.
- `docs/ideas/system-junk-cleaner.md` — its link to this plan (written with the plan) checked; crash reports, caches
  and logs stay there.
- `docs/plans/017-temp-files/` — every step's conclusions written as it was built (checked here).

## Checks

- `npm run lint` (Markdown and links); every link of the changed docs resolves.

## Conclusions

Docs as built: `docs/architecture/modules.md` (the group System, the two known folders walked as places, `Other` in a
match's kind, `temp` as the example of a module that judges by age and refuses by kind and owner; the new `Match` fields
were written by 01/03 and 02/02), `docs/architecture/README.md` (the crate list; the places and sandbox mode were
written by 01/04 and 01/05), `docs/architecture/removal.md` (check 2's roots with the places, check 7 refusing `Other`,
the guard's open that never waits on a pipe), `docs/restrictions/deletion-safety.md` (the one exception to "Unknown is
not junk", decision 6; the new protected places and the two whole ones), `docs/testing.md` (both sandbox reaches, the
`temp.toml` case file, `places.rs` and `temp.rs`), `docs/categories/temp-old-files.md` (rows 2 and 7 as built, the age,
names as bytes, the restore, the place's contract), `docs/deferred/sandbox-world.md` and its row (the stated file facts
taken), a deferred item for the e2e homes' shared helpers and a known issue for the rescan property test's access time,
each with its row. Checked unchanged: the FIFO known issue (fixed in 01/02), the system junk idea's link, the category's
catalogue row. In the plan: the design's rows 2 and 7 and section 9's flows 4 and 5, 03/02's check command, and the
conclusions of every step and plan.
