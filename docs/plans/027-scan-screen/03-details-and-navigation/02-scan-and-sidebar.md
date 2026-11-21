# 027/03/02 — Scan and the sidebar

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** app/ui, e2e

The overview is renamed Scan, its screen, its sidebar item and its texts; the sidebar scrolls when the window is
short, so Settings is always reachable (feedback items 1 and 15). The design: [`../00-design.md`](../00-design.md),
section 1 (agent's call 12; decision 7).

## Files

- `git mv app/ui/src/screens/overview app/ui/src/screens/scan`; inside it `overview-screen.tsx` → `scan-screen.tsx`
  (`OverviewScreen` → `ScanScreen`), `overview-screen.module.css` → `scan-screen.module.css`,
  `overview-screen.test.tsx` → `scan-screen.test.tsx`, `overview-zones.tsx` → `scan-zones.tsx` (`OverviewZones` →
  `ScanZones`); `phase.ts`, `module-progress.ts`, `zones.ts` and their tests move unchanged. Imports updated.
- `app/ui/src/state/navigation.ts` — `{ kind: 'overview' }` → `{ kind: 'scan' }` (the start screen; the fallback of
  `screenToShow`); every `navigate({ kind: 'overview' })` in `app/ui/src` (Review's empty state, the result's Done,
  the details' Back) → `scan`.
- `app/ui/src/screens/screen-stack.tsx`, `app/ui/src/screens/app-frame.tsx` — `scan` in place of `overview`
  (label `ui.screen.frame.nav.scan`).
- `app/ui/src/screens/frame-sidebar.tsx` — the item Scan (`ui.screen.frame.nav.scan`, active on `scan` and
  `category`); its comment.
- `app/ui/src/design/layout/sidebar/sidebar.tsx` and `sidebar.module.css` — the scroll container of the design,
  section 1: `.scroll` holds the indicator, the sections and the footer; `.sidebar` keeps the strip and the scroll
  container; `.footer { margin-top: auto }`; the indicator's comment no longer says the sidebar does not scroll. A
  story `short` in `sidebar.stories.tsx` (a frame 360 px high with five sections) shows the scroll.
- `app/ui/src/locales/en.json` — renamed `ui.screen.overview.*` → `ui.screen.scan.*` (`title` "Scan", `scan-again`,
  `intro`, `partial.*`, `ignored.*`), `ui.screen.frame.nav.overview` → `ui.screen.frame.nav.scan` ("Scan");
  `ui.screen.category.back` "Back to Scan"; `ui.screen.review.empty.action` "Go to Scan" and
  `ui.screen.review.empty.text` "Tick what you want to remove on Scan, then check it here."
- `app/ui/src/i18n/chrome-keys.test.ts` (and any test listing keys) — the renamed keys.
- E2E: every spec and helper that names "Overview", the overview's heading or `data-screen="overview"` names Scan;
  `contrast-overview-*` and `visual.spec.ts`'s `overview` baselines renamed with the tests (`git mv` of the screenshots when
  the test's name changes, then regenerated); `layout.spec.ts` gains the short-window case.

## Produces

The Scan screen and the scrolling sidebar; no later step depends on more.

## Checks

- `sidebar` test (new `sidebar.test.tsx` if absent), negatives first: `it('places no indicator while no item is
  active')`; then `it('places the indicator from the active item inside the scroll container')`, `it('keeps the footer
  inside the scroll container, after the sections')`.
- `frame-sidebar` coverage through `app-frame.test.tsx`: negatives first (no module item, no group heading), then the
  items Scan, Review, Cleanups, Settings and Scan active on a category's details.
- `scan-screen.test.tsx`: the title is Scan; the rest as before the rename.
- `npm run test --workspace app/ui`; `npm run lint:ts`; `npm run knip`; `npx eslint .` in `app/ui`;
  `npx prettier --check` on the changed files; `grep -rn "overview" app/ui/src --include='*.ts*'` shows no screen or
  key named overview (the core's `Overview` type and `get_overview` stay).
- E2E: `layout.spec.ts`, negatives first `at the window's minimum, 900 × 600, no sidebar item is hidden`, positives `in a short frame the sidebar scrolls and Settings can be reached and opened by keyboard` (the viewport
  set to 900 × 420, Tab to Settings, Enter, the Settings screen shows). `npm run test:e2e:update --workspace app/ui --
  gallery.spec.ts visual.spec.ts contrast.spec.ts zoom.spec.ts` (look at each PNG), then
  `npm run test:e2e --workspace app/ui`.

## Conclusions

Not written yet.
