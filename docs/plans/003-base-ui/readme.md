# 003 — Base UI

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui, app/tauri, core, engine

## Goal

A premium dark interface people remember — "Graphite and Iris": layered graphite surfaces, a translucent native
sidebar, light from the accent and the danger levels, one hero visual per screen — with motion that makes a scan feel
alive and keeps the moment of deletion calm and exact. Under it, a design system that is easy to test and to extend:
tokens, primitives, components in every state in a gallery photographed by Playwright, the base screens on the `demo`
module, and lists that stay smooth on a scan of any size. The design: [`00-design.md`](./00-design.md).

Branch: `feat/003-base-ui`, squash-merged into `main` once when done ([`../../workflow.md`](../../workflow.md)).
Depends on plans [001](../001-repository-setup/readme.md) (Vitest, Playwright, CI) and
[002](../002-base-architecture/readme.md) (the IPC, the store, the dictionary, the `demo` module) — both done and
merged. Independent of plan [004](../004-fast-scan/readme.md), except that step 05/01 adds two fields to the core: if
004 runs at the same time, that step is merged into `main`'s line first and 004 rebases on it.

## Plans

| #   | Plan                                                      | Status   | Depends on              | What                                                                                       |
| --- | --------------------------------------------------------- | -------- | ----------------------- | ------------------------------------------------------------------------------------------ |
| 01  | [Foundations](./01-foundations/readme.md)                 | ✔️ done    | —                       | the baseline and its guards, tokens, base styles, motion system, icons, the gallery         |
| 02  | [Primitives](./02-primitives/readme.md)                   | ✔️ done    | 01                      | buttons, toggles, fields, indicators                                                        |
| 03  | [Data and feedback](./03-data-and-feedback/readme.md)     | ✔️ done    | 01, 02                  | sizes and numbers, the ring and the usage bar, rows and the virtual list, notices and toasts |
| 04  | [Shell and window](./04-shell/readme.md)                  | ✔️ done    | 01 (04/02–04: 02)       | the native window and translucency, layout, dialog and sheet, light and texture             |
| 05  | [Data flow](./05-data-flow/readme.md)                     | ✔️ done     | —                       | two core additions, totals instead of findings in the store, paged findings, selection, navigation |
| 06  | [Product components](./06-product-components/readme.md)   | ✔️ done    | 02, 03, 04, 05          | the scan hero, category cards, the finding list, the selection and confirmation, the result |
| 07  | [Base screens](./07-screens/readme.md)                    | ✔️ done    | 04, 05, 06              | the app frame, overview, module page, review, settings and permission, flows and screenshots |
| 08  | [Proof](./08-proof/readme.md)                             | ✔️ done    | 07                      | the performance measurement, the accessibility pass, the docs and the version               |

### Order and parallel work

```
01 ──┬── 02 ──┬── 03 ───────────┐
     │        └── 04/02–04/04 ──┤
     └── 04/01 ─────────────────┼── 06 ── 07/01 ── 07/02–05 ── 07/06 ── 08 ── conclusions
05 ─────────────────────────────┘
```

- **Wave 1, in parallel**: 01 (its steps in order; 01/04 may run beside 01/02–03) and 05 (its steps in order). 05
  touches no design file; 05/02 starts after 01/02 is merged back, because both touch `app/ui/src/app.tsx` and the two
  screenshots of plan 002.
- **Wave 2, in parallel** (after 01): 02 (02/02 and 02/04 first, then 02/01, then 02/03) and 04/01.
- **Wave 3, in parallel** (after the primitives each step uses): 03 and 04/02–04/04.
- **Wave 4** (after 02, 03, 04, 05): 06; its five steps write separate folders of `product/` and may run in parallel.
- **Wave 5**: 07/01, then 07/02–07/05 in parallel, then 07/06.
- **Wave 6**: 08/01 and 08/02 in parallel, then 08/03 and the conclusions.

Each parallel plan runs in its own worktree branched from `feat/003-base-ui` and is merged back when its plan (or
wave of steps) is done; two agents never share a worktree. Stories are found by `import.meta.glob`, so no plan edits a
shared gallery index.

Which files each plan touches, so parallel plans stay apart:

| Plan | Owns (only this plan edits it during its wave)                                                                         | Shared files it touches                                                                                                                  |
| ---- | ---------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| 01   | `app/ui/src/design/{tokens,base,motion,icons}/`, `app/ui/src/gallery/`, `app/ui/gallery.html`, `app/ui/e2e/gallery.spec.ts`, `app/ui/e2e/support/`, `app/ui/src/test/css-guard.test.ts` | `app/ui/package.json`, `package-lock.json`, `app/ui/vite.config.ts`, `app/ui/playwright.config.ts`, `app/ui/tsconfig.json`, `eslint.config.ts`, `knip.json`, `app/ui/src/main.tsx`, `app/ui/src/app.tsx` (01/02: its classes move to a module file), `app/ui/src/styles.css` (removed), the screenshots of plan 002 |
| 02   | `app/ui/src/design/primitives/`                                                                                        | `app/ui/src/design/icons/icons.ts` (new names), the gallery's screenshots folder                                                         |
| 03   | `app/ui/src/design/{data,feedback}/`, `app/ui/src/design/effects/scan-sweep/`                                          | `app/ui/src/design/icons/icons.ts`, the gallery's screenshots folder                                                                     |
| 04   | `app/tauri/tauri.conf.json`, `app/tauri/capabilities/`, `app/ui/src/design/layout/`, `app/ui/src/design/effects/{ambient-light,spotlight}/`, `app/ui/src/design/base/window.css` | `app/tauri/Cargo.toml` (one feature), `Cargo.lock`, the capability test in `app/tauri/`, `app/ui/src/ipc/` (04/01: `inApp()`), `app/ui/src/main.tsx` (04/01: `data-window`), `app/ui/src/design/base/effects.css` (04/04), `app/ui/src/design/icons/icons.ts` |
| 05   | `app/ui/src/state/`, `app/ui/src/test/` (scan fixtures), `app/ui/e2e/core-mock.ts`                                      | `crates/core/` and `crates/engine/` (05/01 only), `app/ui/src/ipc/bindings.generated.ts`, `fixtures/golden/`, `app/ui/src/app.tsx` (05/02: the bare screen shows totals), `docs/known-issues/` |
| 06   | `app/ui/src/product/`                                                                                                   | `app/ui/src/locales/en.json` (keys `ui.<component>.*`), `app/ui/src/design/icons/icons.ts`                                               |
| 07   | `app/ui/src/screens/`, `app/ui/src/modules/`, `app/ui/e2e/*.spec.ts` of the screens, `app/ui/src/app.tsx`               | `app/ui/src/locales/en.json` (keys `ui.screen.*`), `app/ui/src/main.tsx`, `app/ui/src/state/` (07/05: settings and access events), `app/ui/e2e/core-mock.ts` (07/06), `docs/hacks/README.md` |
| 08   | `app/ui/e2e/perf.spec.ts`                                                                                               | `app/ui/package.json` (`test:perf`), `app/ui/playwright.config.ts`, `docs/architecture/README.md`, `docs/testing.md`, `docs/quality.md`, `docs/goal.md` (section 6), the version files |

`package-lock.json` and `Cargo.lock` are never merged by hand: on a conflict take either side, regenerate
(`npm install --package-lock-only`, `cargo metadata --format-version 1 > /dev/null`) and run the checks. Two plans
adding keys to `en.json` keep them in their own prefix, so a merge conflict there is a matter of order.

## Ideas this touches

[Cleanup preview and safe removal](../../ideas/cleanup-preview-and-safe-removal.md) (the review screen,
`ConfirmDialog`, `ResultSummary`, nothing pre-selected above Safe), [user settings](../../ideas/user-settings.md)
(`SettingsRow`), the [ignore list](../../ideas/ignore-list.md) (a card's actions slot is where "Ignore" will go), the
[cleanup simulator](../../ideas/cleanup-simulator.md) (the selection bar's total is its first, simple form), the
[storage explainer](../../ideas/storage-explainer.md) (every arc of the ring is a named category, no "Other"), and
from [best practices](../../ideas/best-practices.md): honest sizes (`42 GB+`, `—`), the demo mode and automatic
screenshots (the gallery and the screens on the `demo` module).

## Acceptance

- `npm run pre-push` passes on the branch's last commit; UI coverage stays at or above 90 %.
- `npm run test:e2e` passes: every gallery story photographed in WebKit, every screen in WebKit and Chromium, the
  keyboard flows; every new screenshot was looked at.
- `npm run tauri dev` opens the app with the translucent sidebar and inset traffic lights; "Scan" on the `demo` module
  shows the sweep, the counting hero number, the cards gliding into order; the module page lists findings largest
  first; selecting findings shows the selection bar; the review screen lists them.
- The CSS guard, the contrast test and the import boundaries pass; `dist/` holds no gallery.
- The performance spec meets the budget on the reference Mac; the numbers are in the conclusions.
- With "Reduce motion" on, nothing translates, scales or loops; with VoiceOver, the scan's progress and end are
  announced and every control has a name.
- The UI store keeps no findings; [`ui-findings-kept-in-memory.md`](../../known-issues/ui-findings-kept-in-memory.md) is
  closed.

## Conclusions

Built on 2026-11-15 and 2026-11-16 in six waves of parallel worktrees, each plan merged back into `feat/003-base-ui`;
each plan's conclusions are in its readme, each step's in its file. The user looked at the result and approved the
interface on 2026-11-16; the plan is ✔️ done.

### What was built

- **The design system** in `app/ui/src/design/`: the tokens of "Graphite and Iris" (three tiers, the contrast of
  every pair tested), the cascade of layers, the motion system (`full | reduced | off`, five hooks of our own, no
  animation library), the icon allowlist (`lucide-react`, 21 icons), and 42 components: 15 primitives, 10 data
  components (sizes, numbers, the ring, the usage bar, rows, the virtual list), 5 feedback components, 9 of layout
  (the shell, the sidebar, the dialog and the sheet on one native `<dialog>`) and 3 effects (the ambient light, the
  spotlight, the scan sweep).
- **The native window** (04/01): an overlay title bar with inset traffic lights, the sidebar on the native material
  through a transparent window (`macOSPrivateApi`, accepted by the user for an app distributed outside the Mac App
  Store), exactly two new window permissions for the drag region.
- **The data flow** (05): two additive core fields (`Overview.found`, `FindingsQuery.order`); the store keeps the
  core's totals, never the findings; lists are paged (at most 10 pages of 100); selection and navigation stores. The
  known issue [`ui-findings-kept-in-memory.md`](../../known-issues/ui-findings-kept-in-memory.md) is fixed.
- **The product components** (06) in `product/`: the scan hero and progress, category cards and their grid, the
  module section, the finding row and list, the selection bar, the confirmation dialog, the result summary, the
  permission card and the settings row.
- **The app** (07): the frame and four screens on the `demo` module (overview, module page, review, settings with
  the permission card); "Move to Trash…" stays disabled until plan 008 (the hack "Review cannot remove yet").
- **The proof** (08): the performance measured, the accessibility pass (axe, keyboard, contrast, zoom), the docs
  (`architecture/` "UI", `testing.md`, `quality.md`, `goal.md` section 6) and the version **0.2.0**.
- **Tests**: unit tests negative first for every component, hook and store; 54 story files in the gallery and 290
  screenshots (every story in WebKit, every screen state in WebKit and Chromium, the focused looks, increased
  contrast, page zoom), each looked at; the e2e flows, keyboard, motion, axe, contrast and zoom specs in both engines.
  New dependencies: `lucide-react` 1.70.0 (ISC, shipped) and `@axe-core/playwright` 4.14.0 (MPL-2.0, dev only).

### Measured

On the reference Mac: MacBook Pro, Apple M3 Pro (`Mac15,7`), macOS 27.2.1; Playwright 1.65.1 with its WebKit 27.2
(60 Hz) and Chromium 159 (120 Hz), headed, 1100 × 720 at 2×; the median of three runs per scenario (08/01).

| Budget (the design)             | Result                                                                             |
| ------------------------------- | ---------------------------------------------------------------------------------- |
| p95 frame ≤ 17 ms (WebKit)      | 18 ms in every scenario, the idle page's own p95 (WebKit's clock); 22 ms on the    |
|                                 | overview during the scan under a load average of 11–17                             |
| no frame over 50 ms             | none, but one frame of 86 ms in one run of three of the overview scan under heavy  |
|                                 | load; 13 reruns of it (8 at a load of 3–5) had none; Chromium: no long task,       |
|                                 | longest frame 17 ms                                                                |
| about 60 list rows in the DOM   | 27 on the module page, 21 while 100,000 rows scroll in ten seconds                 |
| at most 1,000 findings held     | the JS heap grows from 2.7 to 4.9 MB over a 50,000-finding scan (Chromium, after a |
|                                 | full GC)                                                                           |
| bundle ≤ 200 kB JS, ≤ 40 kB CSS | 99.1 kB and 12.4 kB, gzipped                                                       |

`largest` order in the core: one page of 100 out of 100,000 findings in one category in 2.9–5.4 ms (05/01), so no
sorted index is needed for plan 004. `Fresher.app` 0.2.0 is 6.94 MiB, unsigned.
On the branch's last commit `npm run pre-push` passes (UI coverage 98.9 % lines, 92.5 % branches, 94.9 % functions,
97.9 % statements) and `npm run test:e2e` passes (157 Playwright tests and the core e2e).

### Agent's calls

- **The design's calls 1–32** (`00-design.md`): confirmed by the user with the plan's approval on 2026-11-15.
- **Accepted by the user (2026-11-16)**: the calls in the readmes of 03 (a row keeps `role="listitem"` without
  `aria-selected` or `aria-expanded`; `Segment` carries a formatted size; `Notice` takes an explicit `live`), 04 (the
  noise a normal blend at 3 %; the spotlight drawn at rest with motion off; a modal `<dialog>` does not loop the focus
  but never reaches the page behind) and 06 (the hero's number with a smaller unit; a failed scan's error beside the
  ring; the reason tooltip on the badge; the result's buttons do not wait for the freed size).
- **To check** (the user said "take the recommended" but has not seen them):
  - 08/01: each scenario's p95 is held to 17 ms or the idle page's p95 of the same run, whichever is larger, since
    WebKit's frame clock reads 18 ms on an empty page; the one 86 ms frame under heavy load is read as the machine's
    load, not a defect (13 quiet reruns clean), so nothing was changed and the spec keeps the bar.
  - 08/02: when the focused element leaves the screen, the focus falls back to the screen's heading, not to the
    control that took its place (one rule for every screen); the one axe exemption, a contrast finding on the text of
    a greyed-out row whose own control is disabled (WCAG 1.4.3 sets no contrast for inactive components).
- The smaller calls written in the steps of 01, 02, 05, 07 and 08 ("Decided while building") stand as built; each is
  in its step's file.

### Left for later

- To watch: while pages refresh at different times, one finding could briefly sit at two indices of a list (06/03).
- Tauri's page-zoom hotkeys need a webview permission and stay off until plan 005 (04/01).
- The confirmation dialog, wired to the core's cleanup plan, and the removal: plan 008.

### Checks by hand

The agent cannot look at the real window or drive VoiceOver; these are the user's:

1. **`npm run tauri dev`**: the translucent sidebar shows the desktop colours through it, the traffic lights sit
   inset, the window drags by its top strip and a double click zooms; an inactive window's sidebar dims; with "Reduce
   transparency" on, the sidebar turns opaque. **Scan** on the `demo` module shows the sweep, the counting hero
   number and the cards gliding into order; a card opens the module page, findings largest first; selecting findings
   shows the selection bar; the review screen lists them.
2. **VoiceOver**: the overview reads its heading, the hero's "Found" and its size, each card as "Open …" with what
   it is, its level and size; a scan announces its progress at most every 5 seconds and its end once; Scan pressed
   with the keyboard lands on the "Overview" heading; the module page's rows read title, level, size and their
   position; the confirmation dialog (gallery) reads the count, the size and the highest level before its buttons,
   Cancel focused first; a long dialog body is announced as a region named by the dialog's title.
3. **Reduce motion** on: nothing slides, scales or rotates; the scan shows its static state and text.
4. **Increase contrast** on, in the real window: the glows and the noise are gone, borders are stronger, and the
   translucent sidebar and the dialog's backdrop blur still read well (Playwright's WebKit draws no
   `backdrop-filter`).
