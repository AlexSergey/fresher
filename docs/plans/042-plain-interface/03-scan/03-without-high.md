# 042/03/03 — What has no meaning without High

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

While the core withholds High, the sidebar has no Largest and its screen falls back to Scan; an app's page offers only
"Keep my settings" (the core sends only that choice). The e2e mock can state a withheld core. The design:
[`../00-design.md`](../00-design.md), section 1; agent's calls 5 and 18. Mockup:
[`scan.dc.html`](../../../mockups/0.1.0/scan.dc.html) (the sidebar: Scan, Apps; Review, Cleanups; Settings).

## Read first

01/02 (`AppInfo.high`, the mock's `high` option), `screens/frame-sidebar.tsx`, `state/navigation.ts`
(`screenToShow`), `screens/use-app-info.ts`, `screens/apps/app-screen.tsx` and `product/uninstall-choices/` (that the
choices are the core's, with no hard-coded Everything), `e2e/core-mock.ts`, `e2e/apps.spec.ts`, `e2e/support/
{largest-mock,apps-mock}.ts`.

## Files

- `screens/frame-sidebar.tsx` — the Largest item only when `useAppInfo()?.high === 'offered'` (hidden while the
  info is not known yet, so it never flashes).
- `state/navigation.ts` — `screenToShow` takes `high: HighLevel | null` and sends `largest` to Scan unless it is
  `'offered'`; its callers pass it.
- `screens/apps/` and `product/uninstall-choices/` — no change if the choices are the core's; otherwise make them so.
- `e2e/core-mock.ts` — a helper `withheld(script)`: the script's High findings, totals, picks, choices and left-outs
  removed and `high: 'withheld'` in the app info (what a withholding core produces from the same home).
- `e2e/flows.spec.ts` and `e2e/apps.spec.ts` — negative first: "the sidebar has no Largest while High is withheld"; "an
  app's page offers only Keep my settings while High is withheld"; "no High tile is on Scan while High is withheld";
  positive: "the sidebar has Largest when High is offered" (the existing default).
- `e2e/visual.spec.ts` — one baseline: the finished scan of the default script under `withheld` (what ships).

## Tests (negatives first)

`frame-sidebar` (in `app-frame.test.tsx` or its own): negative "no Largest while High is withheld", "no Largest before
the app's info"; positive "Largest when High is offered". `navigation.test.ts`: negative "largest falls back to Scan
while High is withheld"; positive "largest is shown when offered".

## Checks

`npx vitest run src/screens src/state`; lint:ts, lint:code, knip; `npx playwright test e2e/flows.spec.ts
e2e/apps.spec.ts e2e/visual.spec.ts --project webkit --project chromium` (look at the new baseline).

Commit: `feat(ui): hide Largest while High is withheld`.

## Conclusions

- The sidebar leaves out Largest while `AppInfo.high` is `withheld`, and until the app info is known. The e2e
  `withheld(script)` helper (`e2e/support/withheld.ts`) and the Apps mock's `withheld` world; its known limit (cleanup
  and Largest answers untouched) is in `docs/testing.md`.
