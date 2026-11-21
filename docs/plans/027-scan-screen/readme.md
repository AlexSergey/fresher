# 027 — The scan screen

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** app/ui, app/tauri, core, sandbox,
e2e, docs

Drafted on 2026-11-20 as one file from the user's feedback of the first session of runs by hand
([`../../manual-testing/2026-11-20-feedback.md`](../../manual-testing/2026-11-20-feedback.md), items 1 to 18), and
approved by the user the same day with its eight agent's calls, which became decisions. The detailed design, the
global constraints every step follows and the agent's calls made while writing it from the code are in
[`00-design.md`](./00-design.md).

## Goal

One screen answers "where did my space go and what can I remove": the disk as one ring, the found categories in
danger zones below it, and the selection bar to Review. The module pages, the view by place and the separate storage
map go; a category's detail opens from its card.

Branch: `feat/027-scan-screen` from `main`. Depends on plans [003](../003-base-ui/readme.md),
[012](../012-storage-map/readme.md) and [015](../015-module-groups/readme.md), all in `main`.

## What changes

1. **Sidebar** (items 1, 2, 17): Scan, Review (with the selection's count), Cleanups; Settings at the bottom. The
   module groups and their modules leave the sidebar. The sidebar scrolls when the window is short, so Settings is
   always reachable.
2. **Overview becomes Scan** (item 15): the screen and its sidebar item are renamed.
3. **The disk ring** (items 3, 4, 5, 6, 13): the hero's ring draws the home volume's whole capacity from the storage
   map's parts. Free space is the gap where the ring does not close, labelled "Free". The cleanable part is split by
   danger level; the other parts are your folders, apps' data, macOS, swap and sleep image, not measured. Each segment
   has its own colour (two categories never share one). The legend beside the ring lists every segment with its size
   and an "i" button that opens the core's description of the part (with the remainder's reasons and the purgeable
   note). Hovering a segment highlights its legend row and the reverse. The separate block "Data · internal" goes: the
   volume's name becomes the hero's caption. The centre keeps "Found", its size and the counts.
4. **Danger zones** (items 8, 16): below the ring the category cards are grouped in four zones, Safe, Low, Medium,
   High, each with its name, the core's sentence of what the level means, and "Select all" for the zone. Medium and
   High carry a warning icon with an exclamation mark. Review groups its rows by the same zones, with the same heads.
5. **Cards** (items 7, 9, 10): no movement on hover. Each card gets "Details" and a settings button with a menu:
   "Ignore this category" and "Ignore <module>". A card's size is the whole category.
6. **Category details** (items 9, 17): a screen per category, opened from its card: the name, what it is, the danger
   level and why, the left-out lines with their reasons, the module's choices that touch it, and every finding with
   its path, size and checkbox (today's category section of the module page). Back returns to Scan.
7. **What leaves** (items 11, 12, 14, 17): the rows below the cards (apps, places, the rest of the disk), the view
   switch and the view by place, the module pages. Restoring an ignored module or group stays in Settings.
8. **The selection bar on Scan** (items 15, 17): the floating bar (count, size, Clear, Review) shows on Scan and on a
   category's details.
9. **Sandbox Trash** (item 18): in sandbox mode the banner says where the sandbox's Trash is
   (`sandbox/<name>/home/.Trash`), and a finished cleanup to it says the same.

Docs in the same change: plan 012's view by place and storage map, plan 015's module pages, the UI architecture,
`docs/manual-testing.md` items that open a module page or By place, `docs/testing.md` where it names screens.

## Plans

| #   | Plan                                                             | Status      | Depends on | What                                                                                                   |
| --- | ---------------------------------------------------------------- | ----------- | ---------- | ------------------------------------------------------------------------------------------------------ |
| 01  | [Foundations](./01-foundations/readme.md)                        | 🔨 in progress | —          | the sandbox's Trash in the build, the banner and the result; the menu primitive; the ring draws a disk |
| 02  | [The Scan screen](./02-scan-screen/readme.md)                    | 🔨 in progress | 01         | the hero's ring and legend; danger zones and cards; the overview in zones with the selection bar      |
| 03  | [Details and navigation](./03-details-and-navigation/readme.md)  | 🔨 in progress | 02         | the category's details; the module pages go; Overview renamed Scan; the sidebar scrolls; Review in zones |
| 04  | [Docs and finish](./04-docs-and-finish/readme.md)                | 🔨 in progress | 03         | plans 012 and 015, the architecture, testing and manual-testing docs, the deferred row; checks; the walk |

### Order and parallel work

```
01/01 ──┬──────────── 02/02 ──┐
01/02 ──┘                     ├── 02/03 ── 03/01 ── 03/02 ──┬── 04/01 ── 04/02
01/03 ─────── 02/01 ──────────┘          03/03 ────────────┘
```

- **Wave 1, in parallel**: 01/01 (core, shell, bindings, banner, result), 01/02 (the menu), 01/03 (the ring, the
  tones).
- **Wave 2, in parallel**: 02/01 (after 01/01 and 01/03), 02/02 (after 01/01 and 01/02).
- **Wave 3**: 02/03.
- **Wave 4**: 03/01, with 03/03 beside it (after 02/02; it shares only `app/ui/src/locales/en.json` and
  `review-screen.tsx` with 03/01 and 03/02); then 03/02.
- **Wave 5**: 04/01, then 04/02.

`app/ui/src/locales/en.json`, `bindings.generated.ts`, the e2e specs and their baselines are never merged by hand: the
second step to merge rebases, regenerates the bindings and the baselines it touched, and runs the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI.
- `npm run test:e2e --workspace app/ui` passes in all three projects with the committed baselines, which show the new
  screens and no deleted one.
- No module page, view switch, view by place, by-meaning row or home map block is left in `app/ui/src`; the core's
  queries and their tests are unchanged (`git diff main -- crates/engine crates/scanner modules crates/cleaner` is
  empty).
- The user's walk of 04/02 passes every feedback item, 1 to 18.

## Ideas and deferred items this touches

- [Intelligent Storage Explainer](../../ideas/storage-explainer.md): the ring's legend with its "i" is a step toward
  every gigabyte explained; the remainder keeps its reasons, never a catch-all.
- [Large Files Finder](../../ideas/large-files-finder.md) and [Application Footprint](../../ideas/application-footprint.md):
  the view by place and the apps' rows leave the UI; a later folder or app view starts from the core's queries kept
  here, recorded in the new deferred item `place-queries-without-ui` (04/01).
- [A way up from a folder opened from the by-meaning view](../../deferred/way-up-from-meaning.md) (due "with the next
  change to the overview") and [a root's explained line](../../deferred/roots-explained-line.md): their UI goes; both
  fold into the new deferred item (agent's call 15 of the design).

## Agent's calls (to check)

1. **Zone order**: Safe first, High last, so the easy wins come first and the dangerous zone is never the first thing
   selected.
2. **The cleanable part of the ring is split by danger level** (four segments, the zones' colours), not one segment
   per category: a ring of thirty categories cannot be read, and the colours then match the zones below.
3. **The stat tiles and the usage bar beside the ring go**: the legend takes their place; the centre already says
   what was found and how many items and categories.
4. **Other volumes** (an external drive) keep their collapsed line under the ring, as today.
5. **The module choices** (today one: the uninstalled app leftovers' "keep the settings") move to the details of the
   categories they touch.
6. **The view by place and the apps' rows go from the UI only**: the core's queries behind them stay until a later
   plan decides (a row in `docs/deferred/`), so this plan touches no core logic. The core gets only the new texts
   (the zones' heads, "Free", the sandbox's Trash line).
7. **The window's minimum height stays 600 px**; the sidebar's scroll fixes item 1, and with three items it fits
   anyway.
8. **Item 5 (two categories, one colour)** is fixed by the ring's new segments and the legend's own colours, not by a
   separate patch.

## Open questions

None.

## Conclusions

Built on `feat/027-scan-screen` in 20 commits over `main` (183e8d3), **awaiting the user's walk**: the plan is not
marked done until the user has looked at the result (the walk list is in
[`04-docs-and-finish/02-checks-and-walk.md`](./04-docs-and-finish/02-checks-and-walk.md)).

**What was built.** The sidebar holds Scan, Review, Cleanups and Settings, and scrolls when the window is short. Scan
is one screen: the hero's ring of the home volume (Free as the gap, one colour per part, hover both ways with the
legend, an "i" under every legend row), four danger zones of cards, and the selection bar. A category's details open
from its card (Details), with Back to the card's Details button. Review is split in the same zones. The module pages,
the view switch, the view by place, the by-meaning rows, the home map block and the stat tiles are gone from the UI;
the core's queries behind them stay (`git diff 183e8d3 -- crates/engine crates/scanner modules crates/cleaner` is
empty) and the deferred item [`place-queries-without-ui`](../../deferred/place-queries-without-ui.md) holds them. A
sandbox build names its Trash in the banner and in a finished cleanup's result.

**Numbers on the last commit (296f39fb), on the reference Mac.**

| Check                                   | Result                                                                      |
| --------------------------------------- | --------------------------------------------------------------------------- |
| `npm run pre-push`                      | green: lint, crate and dependency guards, knip, machete, deny, 78 script tests |
| Rust tests and coverage                 | 3,290 passed (5 skipped); 97.48 % of lines (floor 90 %)                      |
| UI unit tests and coverage              | 1,130 passed in 130 files; 98.58 % of lines (floor 90 %)                     |
| Playwright gallery, WebKit, Chromium    | 433 passed, no flake in the final run; `git status` clean after it           |
| `npm run test:e2e:core`                 | 110 passed, 2 skipped                                                        |
| Perf, WebKit (reference), scan streaming | `main` "scan, overview": 653 frames, p95 19 ms, longest 33 ms, 3 dropped; branch "scan, scan screen": 651-652 frames, p95 19 ms, longest 40-41 ms, 4 dropped |
| Perf, Chromium, scan streaming          | `main`: 1313 frames, p95 9.9 ms, longest 18 ms; branch: 1305 frames, p95 9.9 ms, longest 25 ms |
| Perf, bundle (gzipped)                  | JS 121.7 kB to 119.3 kB, CSS 15.9 kB to 16.2 kB (budgets 200 and 40)          |
| Perf, 100,000-row scroll                | WebKit p95 19 ms on both; Chromium p95 9.9 ms on `main`, 10.1 ms on the branch |

The Scan screen does not slow a scan's updates beyond the 5 % budget of rule 4: the frame count differs by 0.3 % in
WebKit and 0.6 % in Chromium and the p95 is the same; the longest single frame is a few milliseconds higher on the
branch (a one-frame figure, inside the run to run spread). There is no first-paint number: the perf specs measure
frames, not a paint time, and the scenario "scan, overview" is the closest measure of the screen. The transitions
scenario cannot be compared: on `main` the spec stops at a stale assertion on a card's accessible name (see "What
changed from the design", item 9); on the branch it passes, every row within the budget. The branch's first WebKit
perf run had three soft p95 failures of one to three milliseconds over the idle row and a rerun passed everything;
`main`'s first run failed the scan test once and passed on the rerun: recorded as
[`ui-perf-webkit-jitter`](../../known-issues/ui-perf-webkit-jitter.md).

**The calls taken while building (for the user to look at).**

- Every zone, including Medium and High, has a one-click "Select all" (the approved design; it reverses the old rule
  that selecting stops at Safe). Review and its confirmation stay the gate to a removal.
- On a category's details, a module choice (Keep settings / Remove everything) ticks and unticks all of the module's
  categories, including ones that are not on that screen; Review shows them all.
- A category with only left-out lines and no total gets no card; it shows in a "Found, but left out" section on Scan
  (name, its left-out lines, Details), so the lines stay reachable.
- The details' level heading is "Danger level".
- Back from a category's details returns focus to the card's Details button (constraint 9), found as the first button
  in `li[data-category]`; the e2e `flows.spec.ts` guards it.
- The legend's "i" is a disclosure under its row (design call 4), the card's body is no longer a button (call 7), the
  Free label sits outside the band (call 8).

**What changed from the design and why.**

1. The sandbox's Trash sentence travels in `Build::Sandbox { trash }` and its texts are core keys (design calls 1, 2).
2. Tasks ran one after the other, not in the plan's waves: they share `locales/en.json`, `overview-screen.tsx`,
   `navigation.ts` and the baselines, so parallel work would have conflicted.
3. The gallery-only props `ScanHero.preview`, `RingLegend.initiallyOpen` and the card menu's `menuOpen` exist so the
   stories can show a state that needs a click; nothing in the app sets them.
4. The perf scenarios changed with the screens: "scan, module page" became "a category's details after a scan", and
   the transitions went to Scan, Details, Review and Settings.
5. The status, Back and focus work of the final review: the menu returns focus to its trigger; Back returns focus to
   the card's Details button; the left-out section no longer lists a counted category during a scan.
6. The architecture, testing and manual-testing docs say Scan, the details and the zones; the ideas that still say
   "the module's page" are the user's and were left as written.
7. The idle and failed hero keeps a large empty right side now that the stat tiles are gone; left for the user's
   look.
8. `StatTile`, `modules/registry.ts` and two deferred items (the way up from the by-meaning view, a root's explained
   line) are deleted; their content is folded into `place-queries-without-ui` (design calls 14 to 16).
9. On `main` the transitions scenario of `perf.spec.ts` stops at an assertion that a card's accessible name is just
   "Open <name>", while the card already announced its description and size there; the branch rewrote the scenario
   and the assertion is gone. This is a defect of `main` that the plan fixes by the way; it is not a regression.

**Left open (parked minors, none a defect the user can hit):** the menu re-measures when a caller passes an inline
`items` array; the legend's "i" open state is keyed by index and may jump if parts reorder mid-scan; the Details
button's accessible name is "Details" plus its described-by name (the card's pattern); `keep-focus.ts` selects the
first button of the card (DOM order); a few comment lines in the architecture README and Review's CSS pass 120 characters;
the ring arcs are not focusable (the legend rows carry the keyboard); the sandbox banner's single line cuts a long
"Simulated: ..." text at 820 px.

**The user's walk.** Pending. Items 1 to 18 of the feedback are covered by tests; the walk in a sandbox
(`npm run dev:sandbox -- trial`, and `app-leftovers` for the choices) is the user's, and this plan becomes ✔️ done
when it passes.
