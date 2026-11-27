# 043/02/01 — The e2e mock takes flags

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** e2e, app/ui, docs

The mocked core answers the app info's `flags` from the script, as the app answers its build's; `withheld` sets only
`high`; a spec of the 0.1.0 app names both flags. The UI is unchanged in this step, so no baseline moves. The design:
[`../00-design.md`](../00-design.md), section 6.

## Read first

01/03's commit; `app/ui/e2e/core-mock.ts` (`CoreScript.high`, `withHigh`, `mockCore`, the scripts' `appInfo`
answers); `app/ui/e2e/support/withheld.ts`; every caller of `withheld(` and of `high:` in a script (`rg "withheld\(|
high:" app/ui/e2e`); `app/ui/e2e/core-mock.spec.ts` (the mock's own tests); `docs/testing.md`, the e2e mock's section
and "The gallery".

## Files

- `app/ui/e2e/core-mock.ts` — `CoreScript.high?: HighLevel` becomes `flags?: Partial<BuildFlags>` (comment: the
  build's flags the app info answers; by default `{ high: 'offered', largest: 'shown' }`, what the core can produce);
  `withHigh` becomes `withFlags`, merging the script's flags into `appInfo.flags`.
- `app/ui/e2e/support/withheld.ts` — sets `flags: { ...script.flags, high: 'withheld' }`; its comment says it leaves
  `largest` as the script has it.
- The specs that walk or photograph the app as 0.1.0 shipped (today every `withheld(..)` caller that looks at the
  sidebar or the Largest screen: `visual.spec.ts`'s "a finished scan while High is withheld", the flows of
  `flows.spec.ts` that expect no Largest) state `flags: { high: 'withheld', largest: 'hidden' }`, through one helper
  `firstRelease(script)` in `support/withheld.ts` (High withheld and Largest hidden, as 0.1.0
  shipped). `apps.spec.ts`'s withheld cases look at no sidebar and keep `withheld(..)`.
- `app/ui/e2e/core-mock.spec.ts` — the mock's own cases for `flags`.
- No gallery story reads the app info (`rg useAppInfo app/ui/src`: screens only), so the gallery is unchanged.
- `docs/testing.md` — the e2e mock: a script names the build's flags with `flags`; `withheld` sets `high` only;
  `firstRelease` sets both as 0.1.0 shipped; the core's own tests and the sandbox name `HighLevel` explicitly (a case file's `high = "withheld"`).

## Interfaces produced

- `CoreScript.flags`; `withheld(script)` (High only); `firstRelease`. 02/02 relies on every sidebar spec naming
  `largest` through them.

## Tests (negatives first)

`app/ui/e2e/core-mock.spec.ts`:
- negative: `the app info keeps its flags when the script names none`; `withheld leaves largest as the script has it`.
- positive: `the app info answers the script's flags`; `firstRelease withholds High and hides Largest`.

## Checks

- `npm run lint:ts`, `npm run lint:code`
- `npx playwright test --project gallery --project webkit --project chromium e2e/core-mock.spec.ts e2e/flows.spec.ts
  e2e/visual.spec.ts e2e/apps.spec.ts` (from `app/ui`): passes with no baseline updated.

Commit: `test(ui): the e2e mock answers the build's flags`.
