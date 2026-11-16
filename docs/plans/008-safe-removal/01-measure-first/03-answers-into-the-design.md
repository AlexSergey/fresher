# 008/01/03 — The answers into the design

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** docs

## Files

- `docs/plans/008-safe-removal/00-design.md` — the section "Measured (01)" filled with the answers of 01/01 and
  01/02 (counts, times, error codes; no path); each agent's call the answers touch gets its outcome ("confirmed" or
  "changed, because …"); a section of the design that changes is changed in the same commit.
- The step files of plans 03 and 04 that the answers change (an error code, a flag, a delay, a check that moves).

## Produces

A design that rests on measured behaviour before plans 03 and 04 start. If an answer breaks a premise — the Trash call
follows links, put back needs Full Disk Access, the open-files snapshot is too slow or prompts — the plan stops here
and the user is asked, with options and a recommendation, before any code.

## Checks

- Every question of 01/01 and 01/02 is answered in "Measured (01)".
- The calls touched are marked; nothing in the design still says "unverified" about a probed point.
- `npm run check:repo` passes.

## Conclusions

Done on 2026-11-16. "Measured (01)" answers every question of 01/01 and 01/02 or says why not and who runs it by
hand. Calls 17, 23 and 35 changed, 18, 19 and 24 confirmed; calls 37 (the landed path from the kept handle) and 38
(mapped files) are new and wait for the user. Changed with them: the design's sections 5, 6 and 8, and steps 03/01,
03/02, 04/02 and 04/05. Call 37 changes plan 02's `Trash` port (`move_to_trash` takes the held handle and returns
`Landed { at, reported }`); plan 02 owns that file, so the change is carried into it in the main session before the
core is frozen.
