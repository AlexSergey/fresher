# 043/02/02 — Largest from its own flag

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** app/ui, docs

The sidebar and the navigation show Largest from `flags.largest`, no longer from `flags.high`. The design:
[`../00-design.md`](../00-design.md), section 5; agent's call 4.

## Read first

02/01's commit; `app/ui/src/screens/frame-sidebar.tsx`, `app/ui/src/screens/app-frame.tsx` (`useScreen`),
`app/ui/src/state/navigation.ts` (`screenToShow`, `useScreen`) and their tests (`app-frame.test.tsx`,
`state/navigation.test.tsx`); `docs/architecture/README.md`, "The largest files" and "The High switch in the UI".

## Files

- `app/ui/src/screens/frame-sidebar.tsx` — `const largest = useAppInfo()?.flags.largest === 'shown';` and its
  comment: Largest only while the build shows it.
- `app/ui/src/state/navigation.ts` — `screenToShow(screen, categories, insights, largest: LargestScreen | null =
  null)`: Largest falls back to Scan unless `largest === 'shown'`; the doc comment follows; `useScreen` passes it on.
- `app/ui/src/screens/app-frame.tsx` — passes `info?.flags.largest ?? null`.
- `docs/architecture/README.md` — "The largest files": "Shown while the build's `largest` flag is `shown`" (the
  pick is still refused while High is withheld); "The High switch in the UI" no longer names Largest; a line under
  "The build's flags" (01/02) that the UI reads them only from `AppInfo.flags`.

## Interfaces produced

- `screenToShow`'s fourth argument is a `LargestScreen | null`.

## Tests (negatives first)

- `state/navigation.test.tsx`, negative cases: `sends Largest back to Scan while the build hides it`, `sends Largest
  back to Scan while the flags are not known`;
  positive: `keeps Largest while the build shows it`.
- `screens/app-frame.test.tsx`, negative: `the sidebar has no Largest while the build hides it` (was "while High is
  withheld"); positive: `the sidebar lists Largest while the build shows it` and `... with High withheld too`.

## Checks

- `npx vitest run src/state/navigation.test.tsx src/screens/app-frame.test.tsx` (from `app/ui`)
- `npm run lint:ts`, `npm run lint:code`
- `npx playwright test --project gallery --project webkit --project chromium e2e/flows.spec.ts e2e/visual.spec.ts
  e2e/apps.spec.ts e2e/core-mock.spec.ts` (from `app/ui`): every spec already names both flags (02/01), so no
  baseline changes; a spec that changes shows a flag 02/01 missed, fixed here.

Commit: `feat(ui): Largest follows its own build flag`.
