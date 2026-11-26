# The keyboard e2e test fails at times on a long dialog in Chromium

**Status:** fixed (the gallery tests wait two frames before they look) · **Updated:** 2026-11-19

## Symptom

`npm run test:e2e` at times fails one test, only in the `chromium` project: `e2e/keyboard.spec.ts`, "every story, part
8: Tab reaches each focus stop, its ring shows and a control answers its key", on the gallery story `Dialog/long-body`.
Tab does not reach the dialog's buttons. The same test passes in `webkit`, and in `chromium` it passes on other runs:
it failed during plan 011/02/03 and passed in the final run of plan 011.

## Cause

A race in the test, not in the dialog. The dialog's body measures whether it overflows before the dialog is opened
(`showModal()` runs in the panel's layout effect, after the body's), so it becomes a focusable region only a frame
later, when its resize observer reports. A test that collects the focus stops, or runs the accessibility checker, in
between sees a scrolling body that cannot take the focus: Tab then stops on the body the test did not expect, and the
checker reports `scrollable-region-focusable` (seen on CI, Chromium, 2026-11-19). It was not caused by plan 011.

## What was tried

The gallery parts of `keyboard.spec.ts` and `axe.spec.ts` wait two frames (`nextFrames`, `e2e/support/frames.ts`)
after the story is visible, so the body's focusable state is settled before they look.

## Options left

- Run the test alone in `chromium` with retries and a trace, to see where the focus goes after the body.
- Wait for the dialog to settle (its scroll and its opening motion) before the first Tab.
