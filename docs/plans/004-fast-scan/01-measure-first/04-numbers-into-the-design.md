# 004/01/04 — The numbers into the design

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-15 · **Modules:** docs

## Files

- `docs/plans/004-fast-scan/00-design.md` — a new section "Measured (01)" before "Open questions": the machine
  (model, cores, disk, macOS version), then the answers of steps 01–03 as tables of numbers. No name or path from a
  home folder.
- The same file: the targets of section 13 replaced by bars set from the numbers (a bar is the measured value plus a
  margin of 30 %, rounded); the placeholders are gone.
- The same file: each agent's call that 01 was to verify is confirmed or changed — the journal (call 1), the folding
  threshold (2), compression (8), the queue and thread count (13), the IO policy (15), `EPERM`/`EACCES` (16), the
  stand-in avoid list (17).
- `docs/plans/004-fast-scan/01-measure-first/readme.md` — the conclusions of this plan.

## Produces

The measured basis for plans 02–08: the attribute set and record layout (03/02), the journal's full-scan flags and
flush behaviour (03/04), the thread count and the queue (05/03), the folding threshold (04/03), the bars every later
step measures against.

## Checks

- Every placeholder of section 13 is replaced by a number with its source (which probe, which tree).
- If the journal does not record a file growing in place (01/02, question 1), the plan stops here: the design's
  section 8 is rewritten with the user before 02 starts (a rescan that misses growth would hide exactly the space the
  product exists to find).
- The commit is a docs commit on `feat/004-fast-scan` (`docs(docs): plan 004/01 measured numbers`); `NO_COMMIT/` is not in
  it.
