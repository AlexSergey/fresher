# 003/07/02 — Overview

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `app/ui/src/screens/overview/overview-screen.tsx` — answers "Where did my space go?":
  - `Toolbar` with the title and **Scan** / **Stop** (`useScanActions`; **Scan again** after a scan);
  - `ScanHero` with the phase derived from the status and the last scan's outcome, the overview, the progress;
  - a `Notice` (warning) under the hero for a partial scan: some folders could not be read, so sizes are at least what
    is shown;
  - `PermissionCard` when `getAccessStatus()` says `denied` (never blocking, below the hero);
  - `CategoryGrid` of the overview's totals; opening a card goes to its module's page scrolled to that category;
    selecting a card selects the category;
  - before the first scan, only the hero (the Scan button in the ring) and one sentence of what a scan does and that
    it changes nothing on the disk.
- `app/ui/src/screens/overview/phase.ts` — `phaseOf(status, lastScan, failure)` → the hero's phase (pure, tested).
- `locales/en.json` — `ui.screen.overview.*`.

## Produces

The first screen of the app.

## Checks

- Negative: `before a scan there are no cards and no zero sizes`, `a partial scan shows the notice`, `a granted or
  unknown access shows no permission card`, `a failed scan shows the core's reason`, `phaseOf: a scan the events
  started is scanning even if an older overview says idle`.
- Positive: `Scan starts a scan and the hero shows scanning`, `cards appear as totals arrive, largest first`, `opening
  a card goes to its module's page at that category`, `selecting a card selects its category`.
- `npx vitest run src/screens/overview` passes.

## Conclusions

Done on 2026-11-16. The overview replaces its placeholder in `app/ui/src/screens/overview/`, with tests (negative
first); the frame's visual screenshots (start screen, finished scan; WebKit and Chromium) were updated and looked at.
No new dependency, token or icon; five keys under `ui.screen.overview.*`.

- `phase.ts`: `phaseOf(status, lastScan, outcome)` and `scanFailure(lastScan, outcome)`. The third argument is the
  store's `outcome` (the events' word on the followed scan, newer than the overview's last scan until it is asked for
  again), not a separate failure: a failed scan's reason is in its outcome. A refused command stays the frame's toast.
- `checked.ts`: a card's checkbox — checked for a whole selected category, mixed when only some of its findings are.
- `module-progress.ts`: the scan's chips are the modules `getModules` lists, each running until its `module-finished`.
- The permission card reads `useAccess()` of 07/05 (`screens/permission/use-access.ts`); a refused probe is the
  frame's toast like any refusal.

Decided while building (agent's calls, to check):

- **No duplicate buttons.** The toolbar shows **Scan again** only after a scan (finished, partial or failed). Before
  the first scan the ring's **Scan** is the only one, and during a scan the hero's progress holds **Stop**: the same
  action twice on one screen would be read twice by VoiceOver and is ambiguous for the e2e (`name: 'Scan'`).
- **The title** is an `h1` in the toolbar (no `PageHeader`): the hero is the page's top.
- **The permission card** shows whenever access is `denied`, before a scan too (what will be missing is worth knowing
  before scanning); `unknown` shows nothing, as the checks ask.
- **The partial notice** covers both causes (stopped, or folders that could not be read) and is not `live`: the hero
  already announces the outcome.
- **Cards** show only when the overview has totals and the scan did not fail; opening a card finds its module among
  `getModules`' categories.
- **Duplicate to fold**: `checked.ts` is the same rule as the private `checkedOf` of the module page
  (`screens/module/category-section.tsx`, built in parallel); one shared copy (in `state/selection.ts`) would serve
  both.
- **To watch**: a module switched off in the settings still gets a "scanning" chip during a scan, since the store
  does not keep `scan-started`'s module list; a store field for it would fix that in `state/`.
- `DEMO_DICTIONARY` gained `demo.caches.what` and `demo.downloads.what` (the texts the stories already used), so the
  screenshots' cards show text, not keys; `GRANTED` (an `AccessStatus`) is new there. Since the overview asks for
  the access status, the frame's and the app's unit tests and the smoke, visual and core-mock e2e answers gained
  `getAccessStatus` (without it the mocked core is unreachable).
