# 036/03/01 — The Apps list

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** app/ui

A sidebar entry **Apps** and its screen: the installed apps largest first, each a row with its name, its size, the
core's one sentence, a small breakdown on demand, and what keeps it from being offered when something does; a filter
"Not used for" (Any time, 3 months, 6 months, a year) that the core applies. Opening a row goes to the app's page
(03/02; until then a row opens nothing but is focusable). The design: [`../00-design.md`](../00-design.md), sections 11
and 12.

## Before you start

Read `docs/architecture/README.md` ("UI": folders and boundaries, state, navigation), `app/ui/src/screens/frame-sidebar.tsx`,
`app/ui/src/state/navigation.ts`, `app/ui/src/screens/screen-stack.tsx`, `app/ui/src/screens/category/` (a screen that
asks the core for a list on mount, its loading and refusal states), `app/ui/src/ipc/index.ts` and `ipc/mock.ts` (how a
command and its mock are added), `app/ui/src/product/finding-row/` and `product/sizes.ts` (rows and sizes),
`app/ui/src/design/icons/icons.ts` (the allowlist), `app/ui/src/locales/en.json` (chrome only), the gallery
(`app/ui/src/gallery/`) and `CLAUDE.md`'s TypeScript rules. Plan 035 added a **Largest** entry
to the same sidebar and a screen kind (`largest`) to the same union: keep its lines, add after them.

## Files

- `app/ui/src/ipc/index.ts` — `getApps(query: AppsQuery): Promise<AppsView>`, `getApp(app: AppId): Promise<AppDetail>`,
  `revealApp(app: AppId): Promise<void>` (typed from `bindings.generated.ts`); `ipc/mock.ts` — their mock answers
  (a mock states only what the core can answer together: a not-offered app carries its reason and text, sizes add up).
- `app/ui/src/state/navigation.ts` — `{ readonly kind: 'apps' }` and `{ readonly app: AppId; readonly kind: 'app' }`
  in `Screen` (the second used by 03/02); `screenToShow` unchanged.
- `app/ui/src/screens/frame-sidebar.tsx` — a `NavItem` **Apps** (`ui.screen.frame.nav.apps`, icon `AppWindow` added to
  `design/icons/icons.ts`) in the first section, after Largest (the order: Scan, Largest, Apps); active for `apps`
  and `app`.
- `app/ui/src/screens/screen-stack.tsx` — renders `AppsScreen` for `apps` (and a placeholder heading for `app` until
  03/02).
- `app/ui/src/screens/apps/apps-screen.tsx` (new) — asks `getApps({ unused })` on mount, again when the filter changes,
  when a scan ends and when a cleanup ends (the same signals the category screen listens to); states: loading ("Measuring
  your apps…", the core's measure may take seconds), refused (the core's refusal text), no scan (the Scan button's
  sentence, as the category screen shows it), empty (`ui.screen.apps.empty` when a filter keeps nothing), the list. The
  view's `notice` under the heading. The filter is the design system's segmented control if one exists, else `Tabs`
  (`design/layout/tabs/`); its value is `AppsQuery.unused`.
- `app/ui/src/product/app-row/app-row.tsx` (new) — presentational: name, version, size (`product/sizes.ts`), the
  summary text (`tText`), the not-offered text with a muted icon when not offered, a disclosure "What it takes" that
  lists `parts` by kind (`core.apps.part.<kind>` from the core) with sizes; a click or Enter opens the app's page
  (`onOpen`). No business logic: nothing is computed but formatting.
- `app/ui/src/locales/en.json` — `ui.screen.frame.nav.apps` "Apps", `ui.screen.apps.title` "Apps", `ui.screen.apps.
  measuring` "Measuring your apps…", `ui.screen.apps.empty` "No app fits this filter.", `ui.screen.apps.filter.label`
  "Not used for", `ui.screen.apps.filter.{any,three-months,six-months,a-year}` "Any time", "3 months", "6 months", "A
  year", `ui.product.app-row.parts` "What it takes".
- `app/ui/src/gallery/` — `AppRow` in every state (offered, each not-offered reason, no date, parts open).

## Produces

The list screen and `AppRow` for 03/02; the IPC functions for 03/02.

## Checks

Vitest, negatives first (`describe('negative cases')`, then `describe('positive cases')`):

- `apps-screen.test.tsx`: negative — `shows the refusal text when the core refuses`, `shows the empty sentence when the
  filter keeps nothing`, `shows nothing offered for a not-offered app's row`, `shows the measuring line while the core
  answers`; positive — `lists the apps in the core's order`, `asks again with the filter's value`, `asks again when a
  scan ends`, `shows the core's notice`.
- `app-row.test.tsx`: negative — `shows no breakdown until it is opened`, `shows the reason of a not-offered app`;
  positive — `shows the size and the core's sentence`, `opens the page on Enter and on a click`.
- `frame-sidebar` test: `marks Apps active on the apps and app screens`.
- `npx vitest run app/ui/src/screens/apps app/ui/src/product/app-row app/ui/src/screens/app-frame.test.tsx`;
  `npm run lint` (ESLint boundaries: `product/` imports no `state/` or `ipc/index`); `npm run typecheck` if present;
  knip (the new icon is used).

## Conclusions

- Built: `product/app-row`, `screens/apps/` (the list, the filter, `useApps`), the IPC calls, the screens `apps` and
  `app`, the sidebar's Apps after Largest, chrome keys. Vitest 1359; Playwright 706 with 114 baselines moved by the
  sidebar item (a sample looked at).
- The refusal of the list shows the core's text inline (no toast); three chrome keys were added (`no-scan`, `list`,
  `version`). A review's minors were done in 03/02.
