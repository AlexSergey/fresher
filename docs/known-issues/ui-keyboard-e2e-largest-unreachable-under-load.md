# The keyboard e2e test sees "Fresher is not answering" on Largest under load

**Status:** fixed (`test(ui)`: the test's script answers the Largest screen, and the walk waits for the scan's tiles) ·
**Updated:** 2026-11-26

## Symptom

`e2e/keyboard.spec.ts`, "a screen change never drops the focus", fails at times in WebKit: after the Largest button
the heading "Largest files and folders" is not found, and the page shows the frame's "Fresher is not answering" state.
It failed on CI twice in a row on 2026-11-26 (runs on 6489daad and 4e6b57f7). Locally it passes alone (20 of 20) and
fails under load: `--repeat-each 48 --workers 12` failed 12 of 48 on 4e6b57f7 and 27 of 48 with the UI code of
dd2a29e6, the last green CI run, so it was there before and that run passed by chance.

## Cause

Two timing-dependent paths of the test itself; the product was right on both, and the release app has no mock.

- The test ran the `cleaning` script, which has no Largest mock (`largest: true`), so `get_largest` had no answer: the
  mocked core rejected it with "The core has no answer", an `unreachable` failure, and the frame showed "not
  answering". Logged in the mock, every failing run had `get_largest` rejected with the Largest hook absent; in every
  passing run `get_largest` was never asked, and the Largest screen showed "Scan first to see your largest files."
- Whether it was asked depended on the overview. An overview answered while the scan's events still arrive is older
  than they are and is dropped (`state/reduce.ts`, `loadOverview`); `scan-finished` then marks the state stale, and the
  refresher asks again at most once per 250 ms (`state/refresh.ts`). So for up to 250 ms after the end of the scan the
  store had no last scan and no totals: "Scan again" showed, the tiles did not. Without load the test reached Largest
  inside that window ("Scan first", nothing asked); under load the refresh came first and `get_largest` rejected.
- The same window decided Back to Scan: the screen gives the focus to the Details button of the tile it came from
  (`screens/screen-stack.tsx`, `focusCard`), and to its heading only when the tile is not there. The test expected the
  heading, so with the fix of Largest alone it still failed 9 of 48 under load: the tiles had arrived, the Details
  button took the focus. When the tiles arrived between the test's focus of Details and its Enter, the button was
  replaced and the focus went to the heading (2 of 48).

## Fix

The test runs the `largest` script (the demo scan, with the Largest commands answered) and waits for the Example caches
tile (`[data-category="demo.caches"]`) before it opens the details: every screen then opens on the finished scan's
overview, Largest always asks for and gets its list, and Back to Scan is expected to focus the tile's Details button,
as the product does.

## What was tried

- The reproduction above with the mock's rejections and the page's commands, events and focus changes logged
  (temporary, removed): 13 of 48 failed before the fix (5 on Largest, 2 on Review, after the late rejection of
  `get_largest`, 6 on Back to Scan).
- The `largest` script alone: 9 of 48 failed, all on Back to Scan or the details' heading.
- Waiting for "Scan again" (`scanFinished`): 45 of 48 failed, since it shows before the tiles do; dropped.
- With the fix: 48 of 48, twice, with `--repeat-each 48 --workers 12`; the spec passes in Chromium and in WebKit.

## Options left

None.
