# The keyboard e2e test sees "Fresher is not answering" on Largest under load

**Status:** open · **Updated:** 2026-11-26

## Symptom

`e2e/keyboard.spec.ts`, "a screen change never drops the focus", fails at times in WebKit: after the Largest button
the heading "Largest files and folders" is not found, and the page shows the frame's "Fresher is not answering" state.
It failed on CI twice in a row on 2026-11-26 (runs on 6489daad and 4e6b57f7). Locally it passes alone (20 of 20) and
fails under load: `--repeat-each 48 --workers 12` failed 12 of 48 on 4e6b57f7 and 27 of 48 with the UI code of
dd2a29e6, the last green CI run, so it was there before and that run passed by chance.

## Cause

Not found yet. The frame shows "not answering" when a command rejects with anything but a refusal
(`app/ui/src/ipc/index.ts`, `failureOf`): no timeout exists, so under load one of the mocked commands of the Largest
screen (`get_largest`, `get_largest_place`) seems to reject. The e2e mock answers them through a hook installed on the
page (`e2e/support/largest-mock.ts`, `LARGEST_HOOK`); the first suspect is that the hook is not in place yet, or
answers nothing, when the screen asks under load. The release app has no such mock.

## What was tried

The reproduction above; nothing changed yet.

## Options left

- Log the rejected command and its error in the mock to see which one fails.
- Make the test wait for the hook, or install the Largest mock before the page loads.
