# 003/07/01 — The app frame

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `app/ui/src/screens/app-frame.tsx` — `AppFrame`: `AppShell` with the `Sidebar`, the current screen from
  `useScreen()`, the `Toaster` and the `Announcer`.
  - The sidebar: the app's name under the traffic lights; **Overview** (`LayoutGrid`); a "Modules" section with one
    `NavItem` per module of `getModules()` in registration order, each with its found size from the overview (live
    during a scan; `—` before a scan), named from the module's dictionary (`<module>.enabled.label`, the name its
    settings already use); **Review** (`ListChecks`) with the selection's count as a `Badge`; **Settings** in the
    footer.
  - Screen changes: the outgoing screen fades out over `--duration-fast` (`usePresence`), the incoming one fades in and
    rises `--distance-1` over `--duration-base`; the nav indicator slides; the focus moves to the new screen's heading;
    the scroll position is kept per screen.
  - A refused command anywhere becomes an error `Toast` with the core's text; `unreachable` shows the frame's
    `ErrorState` ("The app could not reach its core. Restart the app.").
- `app/ui/src/screens/use-scan-actions.ts` — `useScanActions()` → `{ scan, stop }` through `requestScan` and
  `cancelScan`, used by the toolbar of every screen.
- `app/ui/src/app.tsx` — renders `AppFrame` inside the store's context; the bare screen of plan 002 is gone (its
  version line moves to the settings screen's footer, 07/05).
- `app/ui/src/modules/registry.ts` — `customViews: Partial<Record<ModuleId, ComponentType<ModuleViewProps>>>`, empty:
  a module with a custom view would register it here; every module uses the generic page in this plan.
- `locales/en.json` — `ui.screen.frame.*` (nav labels, the modules section, the unreachable text moved from
  `ui.core.unreachable`).

## Produces

The frame the four screens plug into; `useScanActions` for their toolbars.

## Checks

- Negative: `a module the core does not list has no nav item`, `the review badge is absent with nothing selected`, `an
  unreachable core shows the error state, not a blank window`, `motion off changes screens without animation`.
- Positive: `the nav items follow getModules in order`, `a module's size updates when the overview does`, `choosing a
  nav item shows its screen and moves the focus to its heading`, `a refused scan shows a toast with the core's text`.
- `npx vitest run src/screens` passes; the smoke e2e of plan 002 is rewritten for the frame and passes.

## Conclusions

Done on 2026-11-16. The frame, its providers and `useScanActions` are in `app/ui/src/screens/`, with tests (negative
first); the smoke, visual and core-mock e2e specs of plan 002 are rewritten for the frame, and both screenshots
(WebKit, Chromium) were looked at. No new dependency or token; one new icon (`Package`, the modules' nav items).

**How 07/02–07/05 plug in** — each step replaces one placeholder file in its own folder and edits no frame file:

| Step  | Replace                                         | Export           | Props                                        |
| ----- | ----------------------------------------------- | ---------------- | -------------------------------------------- |
| 07/02 | `screens/overview/overview-screen.tsx`          | `OverviewScreen` | none                                         |
| 07/03 | `screens/module/module-screen.tsx`              | `ModuleScreen`   | `ModuleViewProps` (`modules/registry.ts`): `{ module: ModuleInfo; category?: CategoryId }` |
| 07/04 | `screens/review/review-screen.tsx`              | `ReviewScreen`   | none                                         |
| 07/05 | `screens/settings/settings-screen.tsx` (and its test) | `SettingsScreen` | none                                   |

- **What the frame gives a screen.** It renders the screen inside a `ScrollArea` (the scroll is kept per screen; a
  module's page keeps one position whatever category it opened at) inside a positioned layer: a screen renders its
  `Toolbar` first, then its content, and may render the `SelectionBar` anywhere — the layer is its containing block, so
  the bar stays at the bottom while the content scrolls. Screens render only once the core's dictionary is loaded, so
  `tText` is safe from their first render.
- **The heading.** Every screen has exactly one `h1` (a `PageHeader`, or an `h1` in the toolbar's title); after a
  screen change the frame focuses the first `h1` of the new screen (it adds `tabindex="-1"`).
- **What a screen reads** (all provided by `screens/app-providers.tsx`, `AppProviders`): `useCore` (store),
  `useNavigation().navigate(screen)` (go to `{ kind: 'module', module, category }` to open a module's page at a
  category, `{ kind: 'review' }`, …), `useFindings` (the paged findings), `useModules()` (`ModuleInfo[]` of
  `get_modules`, `null` until answered) and `moduleFound(module, overview)` (`screens/use-modules.ts`: a module's found
  size, `null` before any scan), `useToasts()` and `useAnnounce()`. `useScanActions()` → `{ scan, stop }` for toolbars.
- **Failures.** A refused command dispatched to the store (`command-failed`, as `useScanActions` and `requestScan` do)
  becomes an error toast with the core's text; a screen does not show store refusals itself. An `unreachable` failure
  replaces the screens with the frame's `ErrorState` ("Try again" reloads the window).
- **Tests of a screen** render `<AppProviders navigation={createNavigation()} store={createCoreStore()}>` around it, with
  `mockCore` answering `getDictionary`, `getModules` and `getOverview` (`DEMO_MODULES` is new in `src/test/demo-scan.ts`,
  and `DEMO_DICTIONARY` gained `demo.enabled.label`); passing a navigation lets a test read where a screen went. The e2e
  mock needs `getModules` in its answers, or the frame shows the unreachable state.

Decided while building (agent's calls, to check):

- **Placeholders.** The four screens exist as minimal placeholders (heading only; the overview has Scan / Stop, the
  settings the version line), so the frame's imports are final and 02–05 never touch `app-frame.tsx`. Placeholders use
  only keys that stay used after they are replaced (`ui.scan-hero.scan`, `ui.scan-progress.stop`, the nav labels).
- **The version line** is in the settings placeholder's header (`ui.app.version`, `getAppVersion`), so `getAppVersion`
  stays used; 07/05 moves it to the footer. Removed keys of the bare screen: `ui.app.connecting`, `ui.scan.*`,
  `ui.totals.*`.
- **`ui.core.unreachable` stays** where it is instead of moving under `ui.screen.frame.*`: `FindingList` (06/03) uses it
  too. The frame adds only the error state's title and its action.
- **`Screen` gained `category?`** on its module kind (`state/navigation.ts`), the overview's "open a card at its
  category"; the frame passes it to the module's view.
- **A module's size** in the sidebar sums the overview's totals of its categories with `summarize` (the core's rule:
  saturating sum, weakest completeness), a dash before any scan, live during one. The review badge is `summarize`'s
  count, absent at 0.
- **Screen changes** stack the leaving and the arriving screen in one grid cell: the leaving one is `inert`, hidden
  from assistive technology and fades out through `usePresence`; the arriving one fades in and rises by CSS. Motion off
  swaps at once; reduced motion only fades. The first screen neither animates nor takes the focus.
- The app's name is a plain line at the top of the sidebar's sections (`ui.app.title`).
