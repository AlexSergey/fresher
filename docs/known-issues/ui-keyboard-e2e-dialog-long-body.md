# The keyboard e2e test fails at times on a long dialog in Chromium

**Status:** open · **Updated:** 2026-11-17

## Symptom

`npm run test:e2e` at times fails one test, only in the `chromium` project: `e2e/keyboard.spec.ts`, "every story, part
8: Tab reaches each focus stop, its ring shows and a control answers its key", on the gallery story `Dialog/long-body`.
Tab does not reach the dialog's buttons. The same test passes in `webkit`, and in `chromium` it passes on other runs:
it failed during plan 011/02/03 and passed in the final run of plan 011.

## Cause

Not known. It is not caused by plan 011: during 011/02/03 it failed the same with the plan's new stories removed. A
guess to check: the long body scrolls inside the dialog, and the buttons below it are reached late or not at all
while the dialog is still settling.

## What was tried

Nothing yet.

## Options left

- Run the test alone in `chromium` with retries and a trace, to see where the focus goes after the body.
- Wait for the dialog to settle (its scroll and its opening motion) before the first Tab.
