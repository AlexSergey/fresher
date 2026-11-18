# 017/03/03 — The UI e2e scenarios

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** app/ui

The module uses the generic module page; nothing of the interface is its own but the group's and the places' texts.

## Files

- `app/ui/src/test/temp-scan.ts` (new) — a scan as the core streams it: the module `temp` in the group System, a few
  offered temporary items, the left-out counts (thousands of `recent`, a few `system`, one `special`), the setting,
  the two place roots in the map; its texts read from `modules/temp/locales/en.json`, shared by the unit tests and the
  e2e mock.
- `app/ui/e2e/core-mock.ts` — the temp scan as one of the mock's scans.
- `app/ui/e2e/flows.spec.ts` — the six scenarios of the design's section 9, "UI e2e".
- `app/ui/e2e/visual.spec.ts`, `axe.spec.ts` — the module's page with its left-out lines, and Settings → Temporary
  files with the number field.
- `app/ui/e2e/screenshots/` — the new images, looked at before committing.

## Produces

The interface's path for the module.

## Checks

- `npm run test:e2e:ui` passes in WebKit and Chromium; the new images reviewed.
- Negative first in each scenario: the left-out lines are not selectable; a value of 6 days is refused with the
  setting's message and the saved value stays 7; an ignored module's findings are in no total.

## Conclusions

Built (5412e71): `app/ui/src/test/temp-scan.ts` (its texts read from the module's and the core's dictionaries), the temp
scan in `core-mock.ts` (whose scripts gained an optional `storage` for the map and its places), nine scenarios in
`flows.spec.ts` (three negative, six positive), two in `visual.spec.ts`, two in `axe.spec.ts`, and four screenshots (the
module's page and its setting, in WebKit and Chromium), each looked at.

The refusal of 6 days shows the UI's generic range text, "Enter a whole number from 7 to 30."; there is no core message
for the setting. That the saved value stays 7 is proven by no update being sent. The special left-out line shows "0
bytes". `npm run test:e2e:ui`: 320 passed, 1 failed, the accessibility gallery's part 3 in WebKit on a 40-second timeout
under load, which passes alone and without this step's changes (a flake that was there before).
