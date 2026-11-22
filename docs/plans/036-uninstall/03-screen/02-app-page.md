# 036/03/02 — The app's page and the uninstall

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** app/ui

An app's page: its sentence and size, every file it would lose item by item with the kind's danger and size, the kept
ones with their reasons, links to its cleanable categories ("Clean only its caches" is one click away), the App
Management notice, the two choices with their sentences (keep-data chosen), and **Uninstall…**, which asks the core for
the plan and opens the normal confirmation. Once confirmed, the progress and the result show on Review, as for every
cleanup. Not offered: the reason, and **Show in Finder** for an app that needs an administrator, or the manager's
command as selectable text. Then the UI's end-to-end scenarios. The design: [`../00-design.md`](../00-design.md),
sections 7, 8 and 12.

## Before you start

Read 03/01, `app/ui/src/screens/review/review-screen.tsx` (how the `confirming` state renders `ConfirmDialog`) and
`cleanup-panel.tsx`, `app/ui/src/state/cleanup.ts`, `state/cleanup-actions.ts` and `state/reduce.ts` (`cleanup-finished`
empties the selection), `app/ui/src/product/confirm-dialog/`, `app/ui/src/product/permission-card/` (a notice with
"Open System Settings": `openAccessSettings`), `app/ui/src/screens/category/category-screen.tsx` (a page that
navigates, the about block), `app/ui/e2e/core-mock.ts`, `flows.spec.ts`, `axe.spec.ts`, `visual.spec.ts` and
`docs/testing.md` ("UI e2e", visual baselines). 02/02's notice is in `AppDetail`.

## Files

- `app/ui/src/ipc/index.ts` — `prepareUninstall(app: AppId, choice: UninstallChoice): Promise<CleanupPlanView>`;
  `ipc/mock.ts` — its answer (a plan with one group of the choice's category, one item whose `more` lists the parts,
  Medium or High).
- `app/ui/src/state/cleanup.ts` and `cleanup-actions.ts` — `prepareUninstallPlan(store, app, choice)`: dispatches
  `cleanup-preparing` with `origin: 'uninstall'`, then `cleanup-prepared` (or the refusal) like `prepare`; the cleanup
  state keeps its `origin` (`'selection' | 'uninstall'`) through `confirming`, `running` and `finished`.
- `app/ui/src/state/reduce.ts` — `cleanup-finished` empties the selection only when the origin is `'selection'`
  (agent's call 14); a test pins both.
- `app/ui/src/screens/apps/app-screen.tsx` (new) — `{ kind: 'app', app }`: asks `getApp(app)` on mount and when a scan
  or a cleanup ends; a refused `unknown-app` (a new scan made the id stale) goes back to the list. Shows: the header
  (name, version, size, the core's summary), the notice (with **Open System Settings** when the capability is
  `Denied`, through the same action the permission card uses), "Clean only part of it" with a link per category
  (`navigate({ kind: 'category', category })`), the footprint list, the choices and **Uninstall…** when offered; the
  reason and **Show in Finder** (`revealApp`) for `needs-administrator`; the reason and the command (`AppDetail.command`,
  a selectable code line) for `installed-by`; the reason alone otherwise. While `confirming` with origin `'uninstall'`
  it renders `ConfirmDialog` exactly as Review does; on confirm it navigates to Review, where `CleanupPanel` shows the
  progress and the result. The core's `core.apps.other-names` line under the list.
- `app/ui/src/product/footprint-list/footprint-list.tsx` (new) — presentational: one row per `FootprintItem`, grouped
  by kind (`core.apps.part.<kind>`), with the danger badge (the existing danger badge component), the size and the
  path on demand; a kept row muted with its reason; per row, which choices take it ("Goes with both choices" / "Only
  with everything", `ui.product.footprint-list.goes.*`).
- `app/ui/src/product/uninstall-choices/uninstall-choices.tsx` (new) — the two `ChoiceView`s as radio buttons with
  their label, description and size (the core's texts), `keep-data` chosen when the page opens, and the button
  **Uninstall…** (`ui.product.uninstall-choices.button`), disabled while a plan is being prepared.
- `app/ui/src/screens/screen-stack.tsx` — renders `AppScreen` for `app`; `apps-screen.tsx` rows now open it.
- `app/ui/src/locales/en.json` — `ui.screen.app.back` "All apps", `ui.screen.app.part-of-it` "Clean only part of it",
  `ui.screen.app.show-in-finder` "Show in Finder", `ui.screen.app.command` "Run this in Terminal to uninstall it with
  its manager:", `ui.product.footprint-list.goes.both` "Goes with both choices", `ui.product.footprint-list.goes.
  everything` "Only with everything", `ui.product.uninstall-choices.button` "Uninstall…".
- `app/ui/src/gallery/` — the page's components in every state: offered with both choices, each not-offered reason,
  the notice not asked and denied, kept rows.
- `app/ui/e2e/core-mock.ts` — the apps commands and a scenario world: three apps (one offered with every kind of part,
  one installed by Homebrew, one needing an administrator), a prepared uninstall plan, a cleanup that moves it, its
  history entry with put back.
- `app/ui/e2e/flows.spec.ts` — `uninstall keeps the data by default`, `uninstall everything lists the files in the
  confirmation`, `a not-offered app shows its reason and no button`, `the filter keeps the unused apps`, `put back of an
  uninstall from Cleanups`. `axe.spec.ts` — the Apps list and an app's page. `visual.spec.ts` — `apps-list`,
  `app-page-offered`, `app-page-not-offered`, `apps-confirmation`; the baselines in `app/ui/e2e/screenshots/`, looked
  at, in chromium and WebKit.

## Produces

The screens 04/01's sandbox walk and 05/01's walks by hand go through.

## Checks

- Vitest, negatives first: `app-screen.test.tsx` — negative: `shows no button for an app that is not offered`, `goes
  back to the list when the id is stale`, `does not empty the selection after an uninstall`, `shows the refusal of a
  prepare`; positive: `chooses keep-data when it opens`, `asks for the chosen plan and opens the confirmation`, `goes to
  Review once confirmed`, `links each category of the app`, `shows the notice and its button when denied`, `shows
  Show in Finder for an app that needs an administrator`, `shows the manager's command`.
  `footprint-list.test.tsx`, `uninstall-choices.test.tsx`, `reduce.test.ts` (both origins).
- `npx vitest run app/ui/src/screens/apps app/ui/src/product/footprint-list app/ui/src/product/uninstall-choices
  app/ui/src/state`; `npm run lint`; `npm run test:e2e:ui -- --project chromium --project webkit` (or the workspace's
  script with both projects); every new baseline looked at; UI coverage at or above 90 % (`npm run test:cov` for the UI
  workspace).

## Conclusions

- Built: `AppScreen` and `use-app.ts`, `product/footprint-list`, `product/uninstall-choices` (two radios), the notice
  of App Management, the uninstall's state (`origin` `selection` | `uninstall`), `prepareUninstall`. Vitest 1410,
  coverage 97.3 %; Playwright apps, axe, keyboard, flows and visual 627 in both engines.
- The choices are buttons with `role=radio` (native radios failed the keyboard spec), with roving tabindex. The
  e2e flows are in `e2e/apps.spec.ts`, not `flows.spec.ts` (about 2400 lines).
- A review's fixes: a refused confirmation shows on Review for either origin; the state keeps the app's id so a late
  plan never opens on another page; the mock refuses as the core does; the baseline scrolls to the choices.
