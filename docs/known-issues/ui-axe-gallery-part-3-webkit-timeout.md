# The accessibility gallery test times out at times in WebKit

**Status:** open · **Updated:** 2026-11-18

## Symptom

`npm run test:e2e:ui` at times fails one test, only in the `webkit` project: `e2e/axe.spec.ts`, "every story of the
gallery, part 3", on a 40-second timeout, under load (the whole e2e run at once). It passes alone, and it failed the
same without the changes of the step it was seen in (plan 017/03/03, 2026-11-18), so it was there before.

## Cause

Not known. A guess to check: one of the part's stories is slow to open or to check under load in WebKit, and a wait
for it runs out before the test's own time does.

## What was tried

Nothing yet.

## Options left

- Run the test alone in `webkit` with a trace, under load, to see which story and which wait time out.
- Share the stories among more parts, so each test checks fewer of them.
