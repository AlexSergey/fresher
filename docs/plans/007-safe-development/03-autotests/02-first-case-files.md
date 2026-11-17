# 007/03/02 — The first case files

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** fixtures, docs

## Files

- `fixtures/sandboxes/trial.toml` — the `trial` module. Negative: a look-alike `fresher-trial-cache2`, a
  `fresher-trial-cache` with no marker next to it, a `fresher-trial-cache.txt` file, a plain file next to a marker.
  Positive: a `fresher-trial-cache` next to a marker (`trial.caches`, Safe, `in-trash`, filled with data so a run
  measures it), a `fresher-trial-kept` next to a marker (`trial.kept`, Low, `emptied`).
- `fixtures/sandboxes/cleanup-guards.toml` — the cleaner's guards in a sandbox, on the `trial` module. Negative: trial
  folders behind a link that points out of the sandbox (never followed), a hard link from a negative's file into a
  positive's folder (the negative's name keeps its content). Positive: a trial cache inside the sandbox's
  `home/Documents` (an item inside a hard-blocked folder goes one by one: `in-trash`), and a trial cache next to a
  marker inside the sandbox's `home/.Trash`: never removed — whether the scan offers it (then `skipped` as protected)
  or not (then a negative) is what the code does today, written down when the file is made.
- `fixtures/golden/sandbox-trial.json`, `fixtures/golden/sandbox-cleanup-guards.json` — written with
  `UPDATE_GOLDEN=1`, reviewed.
- `docs/testing.md` — "Sandboxes": the case files, the generator, the harness, the commands.

## Produces

The case files that prove the harness and the disk parts of plan 008's run by hand.

## Checks

- `npm run sandbox:test` passes both; each fails when one expectation is changed on purpose (done once by hand,
  written in the conclusions).
- `npm run sandbox -- trial` and `npm run sandbox -- all` write their sandboxes; `EXPECTED.md` reads well.

## Conclusions

Two case files, `trial` and `cleanup-guards`, with golden reports; the harness hand-offs were removed (the early return and
the `expect(dead_code)`). Deviation: the Documents positive became a negative (the sandbox avoids its own Documents), and the
one-by-one positive uses `Library/Caches`. Found: the scan offers an item inside `.Trash`, which the cleanup leaves out as
protected (a known issue). Breaking an expectation made the autotest fail; reverted.
