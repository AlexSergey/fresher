# 016/03/03 — The UI e2e scenarios

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** app/ui

## Files

- `app/ui/src/test/jetbrains-scan.ts` (new) — a scan as the core streams it: the module `jetbrains` in the group
  `development` with its three categories, findings of WebStorm 2024.3 in each (with `app`), the `module-finished`
  event with left-outs (`current` ×3, `running` ×2), and the texts read from `modules/jetbrains/locales/en.json`, so a
  changed text changes the tests. Shared by unit tests and the e2e mock.
- `app/ui/e2e/core-mock.ts` — a `jetbrains` script beside the demo's (the scan, the storage meaning with WebStorm's
  line).
- `app/ui/e2e/flows.spec.ts` — the seven scenarios of the design, section 8, "UI e2e": the sidebar's Development and
  the module page's levels and texts; the left-out lines; select → Review → confirm to the Trash → progress → result
  → Cleanups → put back; the confirmation with Safe only; ignore the module from its page and restore it in Settings;
  ignore the group Development and restore it; the WebStorm line in the by-meaning view.
- `app/ui/e2e/visual.spec.ts`, `axe.spec.ts` — the module page with left-out lines; the confirmation with Medium;
  `app/ui/e2e/screenshots/` updated and reviewed.
- `app/ui/src/screens/module/module-screen.test.tsx` — the generic page on `jetbrains-scan.ts`: three sections, the
  left-out lines, the Medium badge.

## Produces

The module's interface path, end to end.

## Checks

- `npm run test:e2e:ui`; each scenario fails once when its assertion is flipped on purpose (written in the
  conclusions); `npx vitest run src/screens/module/`.

## Conclusions

Built: `app/ui/src/test/jetbrains-scan.ts`, `SCRIPTS.jetbrains` in `core-mock.ts`, nine scenarios in
`flows.spec.ts` (levels and texts, left-out lines, the open IDE, the removed IDE's category, review to Trash to put back
at Medium, a Safe-only confirmation, ignoring the module and the group, the days setting, the by-meaning WebStorm line),
two visual shots (webkit and chromium), axe on the page and the Medium confirmation. `npm run test:e2e:update`: 295
passed. Not done: flipping each assertion once to see it fail (see 04/02).
