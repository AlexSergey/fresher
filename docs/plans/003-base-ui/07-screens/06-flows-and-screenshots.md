# 003/07/06 — Flows and screenshots

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `app/ui/e2e/core-mock.ts` — scripts for the flows: the demo scan, a large generated scan (`large-scan.ts`), a
  partial scan, a failed module, a refused scan, settings with each kind, access denied; scan events streamed with
  small delays so the live states can be paused on.
- `app/ui/e2e/smoke.spec.ts` — rewritten for the frame: the app opens on the overview; the core unreachable shows its
  error state.
- `app/ui/e2e/flows.spec.ts` — the user's paths, by keyboard as much as by pointer:
  - scan → cards appear largest first → open a card → the module page at that category → tick findings → the
    selection bar → Review → the review screen lists them → the remove button is disabled with its reason;
  - a second Scan during a scan shows the refusal toast; Stop ends the scan as partial with the notice;
  - settings: switch the demo module off, the event updates the row;
  - the whole path without a pointer: Tab, arrows, Space, Enter, Escape.
- `app/ui/e2e/visual.spec.ts` — screenshots of every screen and state in WebKit and Chromium, with motion off:
  overview before a scan, scanning (paused mid-stream), finished, partial, failed, access denied; module page with a
  category expanded and rows selected; review with a selection and empty; settings normal and read-only.
- `app/ui/e2e/motion.spec.ts` — under `prefers-reduced-motion: reduce` (Playwright's emulation, motion not forced
  off): the scan shows no rotating sweep and no breathing (`getAnimations()` on the hero is empty or paused), a screen
  change only fades; under full motion a scan does run the sweep (the reverse check).

## Produces

The proof that the screens work together, and the screenshots the README and plan 008 start from.

## Checks

- Negative (`test.describe('negative cases')`): the refused scan, the unreachable core, the disabled remove button,
  reduced motion.
- Positive: the scan-to-review path, the keyboard-only path, the settings path, every screenshot.
- `npm run test:e2e:ui` passes; every new screenshot is looked at in both engines before it is committed.

## Conclusions

Done on 2026-11-16. Two new specs (flows, motion), the smoke spec on the shared scripts, and 11 states of the screens
photographed in WebKit and Chromium (22 images, 18 new; the start screen and the finished scan of 07/01 are
unchanged), each looked at; `npm run test:e2e` and
`npm run pre-push` pass. No new dependency, token, icon or text of the app.

- **The mock** (`e2e/core-mock.ts`) gained what the live states need: a scan **held** after `hold` events until
  `releaseScan(page)` or Stop, `get_overview` answering `scanning` meanwhile; `cancel_scan` ends a held scan with the
  script's `stop` events and overview; a `start_scan` while a held scan runs is refused as busy (the engine's rule);
  an answered `update_setting` sends its `setting-changed` event; `get_findings` pages only the findings streamed so
  far; every channel message has the next index (a second scan no longer restarts at 0). `SCRIPTS` holds the ten
  scripts the specs play (demo, live, large, partial, failed, refused, unannounced, settings, read-only, denied); their
  fixtures (`DEMO_LIVE_SCAN`, `DEMO_STOPPED`, `DEMO_PARTIAL`, `DEMO_FAILED`, `SETTINGS_OF_EVERY_KIND`, `DENIED`,
  `SCRIPTS_DICTIONARY` with the core's own texts) are in `src/test/demo-scan.ts`.
- **Flows** (`flows.spec.ts`): negative — a second Scan before the scan's first event shows the core's busy text; the
  review screen's **Move to Trash…** is disabled with its reason and opens no dialog. Positive — scan → cards largest
  first → a card opens the module page at its category, expanded and in view → rows ticked → the selection bar →
  Review lists them, removal disabled; Stop ends a held scan as partial with the notice and the first batch's cards; a
  thousand-finding category pages as the keyboard moves to its end; the demo module's switch turned off is updated by
  the core's event; the whole path by Tab, Enter, the arrows, Space and Escape. The smoke spec plays the same scripts.
- **Motion** (`motion.spec.ts`, `prefers-reduced-motion` emulated, motion not forced off): reduced motion runs no
  endless animation during a scan (the sweep is drawn, standing still; no breathing) and a screen arrives with opacity
  0 and no rise; full motion runs the sweep and the breathing and the screen rises 4 px.
- **Screenshots** (`visual.spec.ts`): overview before a scan, scanning (held mid-stream), finished, partial, failed,
  Full Disk Access denied; the module page with a category expanded and rows selected; review with a selection and
  empty; settings of every kind and read-only. Each waits for the core's last overview (the module's size in the
  sidebar), so the refresh after a scan cannot land between the wait and the picture.

**Bug found and fixed**: a keyboard move onto a row whose page was still loading (End in a long list) lost the focus
when the page arrived: the skeleton row was replaced by the loaded one under a new key, and the focus fell to the page.
`VirtualList` now remembers that a row of it had the focus and gives it to the row now at that index when the focused
one is removed (a leave to another element or to nothing is still a leave); two unit tests, and the large-list flow,
prove it in both engines.

Decided while building (agent's calls, to check):

- **A hold point, not delays**: the live states are paused by holding the stream after a set number of events, not by
  timed delays, so the scanning picture and the Stop flow are the same on every run.
- **The second Scan**: no screen offers Scan during a scan (07/02), so the refusal is reached the only way a user can
  reach it — a second click before the scan's first event arrives (the `unannounced` script).
- **The failed script** fails the only module and so the scan, with the module's reason, to show both the module's
  failure and the overview's failed state.
- **Escape** in the keyboard path is pressed on the review screen and changes nothing there (no dialog is mounted
  until plan 008).
- Seen, not changed: a failed scan shows both the toolbar's **Scan again** and the hero's **Try again**; the sidebar
  keeps the failed scan's found size. During a scan a module switched off still shows a "scanning" chip (07/02's note):
  the step does not ask for it.
