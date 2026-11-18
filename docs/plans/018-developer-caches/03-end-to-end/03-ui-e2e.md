# 018/03/03 — The UI e2e scenarios

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** app/ui

The module uses the generic module page; nothing of the interface is its own. The group Development exists (plan 015).

## Files

- `app/ui/src/test/dev-caches-scan.ts` (new) — a scan as the core streams it: the module `dev-caches` in the group
  Development, findings of npm (Low, 8.9 GB), Cargo's two folders, Gradle and two derived data projects (Safe, `app`
  `com.apple.dt.Xcode`), the left-out counts (one `app-open`, one `not-yours`), no setting but the switch, the map with
  Xcode's line; its texts read from `modules/dev-caches/locales/en.json`, shared by the unit tests and the e2e mock.
- `app/ui/e2e/core-mock.ts` — the dev-caches scan as one of the mock's scans.
- `app/ui/e2e/flows.spec.ts` — the six scenarios of the design's section 6, "UI e2e", negatives first:
  - negative: "the left-out lines of developer caches cannot be selected"; "an ignored cache category is in no total";
  - positive: "developer caches show under Development with their size"; "a developer cache category shows its danger
    and its five texts"; "npm's cache and a project go to the Trash and come back, the confirmation naming Low";
    "ignoring npm's cache keeps the other caches, and restoring brings it back"; "ignoring the developer caches module
    hides them all"; "Settings shows the developer caches switch alone"; "a project's build data counts in Xcode's
    line".
- `app/ui/e2e/visual.spec.ts`, `axe.spec.ts` — the module's page with its left-out lines.
- `app/ui/e2e/screenshots/` — the new images, looked at before committing.

## Produces

The interface's path for the module.

## Checks

- `npm run test:e2e:ui` passes in WebKit and Chromium; the new images reviewed.
- `npx vitest run src/test` for any unit test that reads the scan file.

## Conclusions

Built (943aabd): `app/ui/src/test/dev-caches-scan.ts` with the module's real texts from `modules/dev-caches/locales/en.json`,
registered in `core-mock.ts`; the scenarios of the design in `flows.spec.ts` (the left-out lines cannot be selected and
an ignored category is in no total, first; then the module under Development with its size, the five texts and danger,
select, trash and put back naming Low, ignore npm and restore, ignore the module, the Settings switch alone, Xcode's line
in the by-meaning view); the module's page in `visual.spec.ts` and `axe.spec.ts`.

Deviations and calls:

- Two scans, not one (F3): the main one has Xcode closed (npm, Cargo's two folders, Gradle and two projects offered, a
  `not-yours` left-out on a third project), the other has Xcode open (the projects left out `app-open`, a `not-yours`
  line on Gradle). A real scan cannot offer a project and leave another out `app-open` at once, so no one mock does.
- The ignore mock `app/ui/e2e/support/ignore-mock.ts` is extended to ignore categories (totals filtered, restore): it
  had none and the brief's category scenarios need it.
- The ignored-category negative asserts the totals only and the positive that the other categories stay (F18).

Tests: `tsc` and `eslint` clean; `vitest run src/test`: 15 passed; `npm run test:e2e` (gallery, WebKit, Chromium): 343
passed. Two new screenshots, `module-page-dev-caches-{webkit,chromium}.png` (dark; npm's cache expanded at Low, Gradle
with its `not-yours` line, Xcode's line "Its app is open..."), looked at: both engines show the same content, no
clipping. A category holding only left-outs shows "0 bytes, 0 items", the existing behaviour of the UI. Parked minors:
the negative "left-out lines cannot be selected" has a positive tail; the final `8.3 GB` assertion of "ignored category
in no total" is weak; the scan file's header mentions unit tests that do not exist.
