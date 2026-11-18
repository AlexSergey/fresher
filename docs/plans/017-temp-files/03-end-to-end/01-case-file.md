# 017/03/01 — The sandbox case file

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
fixtures, app

Rule 8 of [`../../../rules.md`](../../../rules.md): the module ships its case file, negatives first, with its golden
report, its left-out expectations and its fixed clock (the world's `now`).

## Files

- `fixtures/sandboxes/temp.toml` (new) — `feature = "temp"`, `module = "temp"`; the eighteen negative cases, then the
  six positive ones, of the design's section 9, "Sandbox": files under `private/var/folders/xx/sandbox/T/`,
  `private/tmp/`, `private/var/folders/yy/other/T/` and `home/`; stated facts (`changed`, `accessed`, `owner`, `flags`,
  `kind`) where the disk cannot make them; the running and installed apps (`[[case.running]]` for
  `com.example.Editor`, `[[case.app]]` for `com.example.Viewer`); `[[case.left-out]]` for every left-out the design
  lists (category `temp.old-files`, the reason, the items); `fill = "data"` on the positives' files, so the map's
  numbers are blocks; a `[case.map]` on the first positive (`explained`) and the fresh file (`your-folders`). Its
  paths are its own across every case file.
- `fixtures/golden/sandbox-temp.json` (new) — `UPDATE_GOLDEN=1`, reviewed: the findings with their app, the left-out
  counts, the map with the two place roots, the cleanup's outcome, the tree after.
- `app/tauri/tests/sandboxes.rs` — `temp` in `case_files!`.

What the code does for each case is written down when the file is made (as plans 007, 012 and 016 did); a case that
differs from the design is fixed in the module, not in the case file.

## Produces

The module's sandbox: its autotest and the run by hand (04/02).

## Checks

- `npm run sandbox:test -- temp` passes; it fails, once each by hand, when a positive's danger, a left-out's `items`,
  a stated `changed` (30 days to 1) and a negative's file (moved to an offered name) are changed on purpose (written in
  the conclusions).
- `npm run sandbox -- temp` writes `sandbox/temp/` with its places, `world.toml` with the stated entries and the apps,
  and an `EXPECTED.md` that names every stated fact; `npm run sandbox -- all` writes it beside the other case files
  without a clash.
- `npm run lint`.

## Conclusions

Built (e159561): `fixtures/sandboxes/temp.toml` with 20 negatives, then 6 positives, `fixtures/golden/sandbox-temp.json`
and `temp` in `case_files!`. Two negatives beyond the design, from 02/02's rulings: a hidden Editor file (`app-open`)
and a folder holding `.#notes.txt` (`lock`). The left-outs sum to app-open 2, lock 4, locked 1, recent 6, special 3,
system 5; the cleanup moves six items, the link as itself, its target unchanged. `World::read` now accepts a stated
entry that is a link itself (a path through a link is still refused). No module change was needed.

Read the run by hand with this in mind: in the autotest the clock stands at the world's `now`, which is before the
files' real status change, so the cases "a fresh temporary file is kept" and "an unpacked folder with old modification
times is kept" reach `recent` through the future-time path, as `T/future` does, not through the days. The core e2e's
flow 2 proves the unpacked folder on real times. Not expressible in TOML: names that are not UTF-8 (the module's unit
tests cover them). The map's remainder names two unreadable folders that need Full Disk Access (`home/Documents`, where
the link's target is, and `home/Library/Autosave Information`), as `cleanup-guards.toml` does.

The checks by hand that a changed expectation fails, each made, run and reverted:

- the first positive's `danger` from safe to low: "offered as Safe, not as Low";
- the locks' `items` from 3 to 2: "expected 3 items, got 4" (the file's sum);
- the first positive's stated `changed` from 30 days to 1: "`temp.left-out.recent`: expected 6 items, got 7";
- a negative's file moved from `C/` to `T/cache.db`: "offered: private/var/folders/xx/sandbox/T/cache.db".

`npm run sandbox:test` passed 37; `npm run sandbox -- temp` wrote 26 cases, `world.toml` with 51 entries and an
`EXPECTED.md` naming every stated fact; `all` wrote 70 cases without a clash.
