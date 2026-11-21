# 027 — The scan screen: design

**Status:** 🔨 in progress (the user's decision of 2026-11-20) · **Version:** 0.1.0 · **Date:** 2026-11-20 ·
**Modules:** app/ui, app/tauri, core, sandbox, e2e, docs

Written on 2026-11-20 from the approved one-file draft ([`readme.md`](./readme.md), its eight agent's calls approved
as decisions) and from a read of the code on `feat/027-scan-screen` (from `main` 183e8d3). Where the draft was silent
or wrong about the code, the design decides and lists the choice under "Agent's calls".

## Global constraints

Every implementer and every reviewer of this plan checks each change against these. A change that breaks one is not
merged.

From the project ([`../../../CLAUDE.md`](../../../CLAUDE.md), [`../../rules.md`](../../rules.md)):

1. English only in every file, identifier, comment, commit and doc; no other product or project is named anywhere.
2. Never copy an implementation from elsewhere; design our own.
3. **No business logic in the UI**: sizes, danger levels, descriptions and every sentence a user reads about a disk,
   a category or a part come from the core (its dictionary, `core.*` keys, or its answers). The UI only sums the
   core's per-category sizes grouped by the core's levels (section 2.2) and renders.
4. Every call to the core goes through `app/ui/src/ipc/index.ts`; components never call `invoke`.
5. Components are small and presentational: the design layer (`app/ui/src/design/`) knows nothing of the core; the
   product layer (`app/ui/src/product/`) takes props; the screens (`app/ui/src/screens/`) wire state.
6. TypeScript strict, no `any`, `type` over `interface`, `readonly` props, discriminated unions, no needless
   generics. Prefer small focused files; reuse before adding; minimal diffs; no reformatting of unrelated code.
7. Rule 7 (simple interface): every screen answers one question; every number has a plain label; every choice the
   user is offered (a button, a menu item, "Select all") carries one or two plain sentences of what it does.
8. Nothing is deleted without confirmation: this plan changes no removal path. Selecting is not removing; Review and
   its confirmation stay the only way to a cleanup.
9. Accessibility: every control reachable and usable by keyboard, focus visible and never lost on a screen change,
   names that contain the visible text (WCAG 2.5.3), the axe and contrast suites green, reduced motion and increased
   contrast honoured.
10. Rust: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`; no `unwrap`/`expect`/`panic!`/
    `println!` outside tests; typed errors; `#[expect(…, reason = "…")]` only; enums over flags; `&Path`/`PathBuf`
    for paths, never string concatenation.
11. Tests: deterministic (no clock, no network, no real home); negative cases first, in their own block
    (`mod negative` / `describe('negative cases')`), then positive (`mod positive` / `describe('positive cases')`);
    descriptive names. Coverage stays at or above 90 % of lines in Rust and in the UI.
12. A changed component updates its gallery story and its committed baselines in the same step
    (`app/ui/e2e/screenshots/`), regenerated on this Mac with `npm run test:e2e:update --workspace app/ui -- <spec>`
    and looked at before the commit; a deleted component deletes its story and its baselines.
13. Every step leaves `npm run lint:ts`, `npm run knip`, the UI unit tests and the e2e specs it touches green, and
    ends with one commit (Conventional Commits, `feat(app): …`, `docs(docs): …`, no AI attribution, never
    `--no-verify`).
14. Rust in this worktree builds into the main checkout's target folder:
    `CARGO_TARGET_DIR=/Users/serhiialeksandrov/Desktop/Work/Projects/my/github/fresher/target`.
15. The core's queries behind the view by place and the by-meaning rows stay untouched (decision 6); only their UI
    goes.

## Intent

The user's feedback of the first session of runs by hand, 2026-11-20 (translated,
[`../../manual-testing/2026-11-20-feedback.md`](../../manual-testing/2026-11-20-feedback.md), items 1 to 18): the
sidebar is overloaded with modules and hides Settings in a short window; the overview's ring, the separate map block,
the two views and the rows under the cards are hard to read; the cards jump on hover and have no way to their detail
or to ignore them; Review should be split by danger zone; the selection bar belongs on the overview, which then
becomes "Scan"; after a sandbox cleanup it is not clear where the items went.

## Goals

1. One screen, **Scan**, answers "where did my space go and what can I remove": the home volume as one ring with a
   legend, the found categories in four danger zones below it, the selection bar to Review.
2. The sidebar holds Scan, Review, Cleanups and Settings, and scrolls when the window is short.
3. A category's **details** open from its card on their own screen; the module pages, the view by place, the view
   switch, the separate map block of the home volume and the rows under the cards leave the UI.
4. Review groups its rows by the same four zones.
5. In sandbox mode the banner and a finished cleanup to the Trash say where the sandbox's Trash is.

## Non-goals

- Any change to the core's logic, the engine, the scanner, the modules or the cleaner (decision 6). The core gains
  texts and one field of `Build` (agent's call 1), nothing else.
- Removing the core's place and meaning queries (`get_storage_place`, `get_storage_roots`, `get_storage_meaning`):
  they stay, the first two without a UI, recorded in `docs/deferred/` (section 9).
- A new module or category: no sandbox case file changes (the sandbox autotest is unaffected).
- The window's minimum size (decision 7).

## What already exists

| Need                         | What exists                                                                                                                                       | Plan     |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| Navigation                   | `state/navigation.ts`: `Screen` = `module` (with `category?`) · `cleanup` · `cleanups` · `overview` · `review` · `settings`; `OverviewView` (`meaning` or `PlaceAt`) kept for the session; `screenToShow` falls back to the overview for a module the core no longer lists | 003, 012 |
| The frame                    | `screens/app-frame.tsx` (`contentOf` per screen, `customViews` of `modules/registry.ts` for a module page), `screens/screen-stack.tsx` (`screenKey`), `screens/frame-sidebar.tsx` (Overview, one `NavSection` per module group with an item per module, Review with the count, Cleanups, Settings in the footer) | 003, 015 |
| The sidebar                  | `design/layout/sidebar/sidebar.tsx`: strip, indicator placed from the active item's `offsetTop`, sections, footer; `.sidebar { overflow: hidden }` — it never scrolls | 003      |
| The overview                 | `screens/overview/overview-screen.tsx`: toolbar (title, Scan again), `ScanHero`, intro, partial notice, `RescanNotice`, `CoverageNotices`, one `StorageMap` per volume (the second collapsible), the `SegmentedControl` view switch, `MeaningList` or `ByPlace`, the ignored line | 003, 012 |
| The hero                     | `product/scan-hero/scan-hero.tsx`: `StorageRing` of the found size by category (one arc per category, tone = its highest level), the centre `Found`, three `StatTile`s, a `UsageBar` with legend, `ScanProgress` | 003      |
| The ring                     | `design/data/storage-ring/` (`arcsOf` in `arcs.ts`: an arc and a `GAP` per segment, `MIN_SWEEP`; tones via `data-tone`); `design/data/segment.ts` (`Segment`, `Tone` = level · `plain` · `system` · `free` · `unknown`, `chartLabel`) | 003, 012 |
| The map's texts              | `product/storage-map/parts.ts`: `partName`, `partDescription`, `reasonText`, `explainedTone`, `segmentsOf`; `part-line.tsx` (a part's line, its details, the remainder's reasons, Free's purgeable note); core keys `core.storage.part.<kind>.{name,description}`, `core.storage.remainder.*`, `core.storage.reason.*`, `core.storage.purgeable`, `core.storage.changed`, `core.storage.kind.*` | 012      |
| The map                      | `Engine`'s `StorageMap { phase, volumes: VolumeMap[] }`, the home volume first; `VolumeMap { name, kind, capacity, parts: MapPart[], purgeable, roots, places, diskChanged }`; parts in `PartKind`'s order (explained, your-folders, not-scanned-yet, remainder, mac-os, swap-and-sleep, other-volume, disk-own, free) adding up to the capacity; Explained = the counted categories' bytes under the volume's roots (the findings so far during a scan) | 012      |
| By meaning                   | `useStorageMeaning(volume)` (`state/storage.ts`) → `MeaningGroup[]`: apps (with their categories' shares and data folders), categories no app names, places, parts; every byte of the roots in exactly one leaf; during a scan only the volume's parts | 012      |
| The cards                    | `product/category-card/` (whole card a button that opens the module page; hover lifts it 1 px; an `actions` slot unused), `product/category-grid/` (largest first, coalesced order while scanning, FLIP moves), `product/meaning-list/` (apps' rows, cards with the share outside apps, places, the rest) | 003, 012 |
| Totals and levels            | `Overview.totals: CategoryTotal { category, count, sizes, highestDanger }`, `Overview.found`; `ModuleInfo { id, group, categories: { descriptor: { id, danger, … }, availability }[], choices }`; `accessOfCategories`, `unavailableTotals` (`screens/permission/category-access.ts`) | 003, 006 |
| Ignoring                     | `ignore(target: IgnoreTarget)` with `category`, `finding`, `module`, `group` targets (`ipc/index.ts`); `useIgnoreCommands()` (`screens/settings/use-ignore-commands.ts`); the module page's "Ignore this module" and the category section's "Ignore {name}"; restoring in Settings → Ignored | 005, 015 |
| A category's detail          | `screens/module/category-section.tsx` (row with the category's checkbox, `About` disclosure with the four sentences `<category>.{what,made-by,if-removed,comes-back}`, Ignore, `LeftOutLines`, `CategoryFindings` with `FindingList` and reveal), `module-choices.tsx`, `unavailable-reason.tsx` | 003, 011 |
| Selection                    | `state/selection.ts`: actions `toggle-category`, `toggle-finding`, `choose { categories, within }`, `clear`; `checkedOf`; `summarize` (`state/selection-summary.ts`); `product/selection-bar/` on the module page only | 003      |
| Review                       | `screens/review/`: `reviewGroups(selection, totals)` → `ReviewGroup { category, count, highest, rows, sizes }` largest first; `ReviewGroupSection`; `CleanupPanel` → `ResultSummary` (`method === 'trash'` lines) | 008      |
| Danger texts                 | `core.danger.<level>.name` ("Safe", "Low", "Medium", "High"), `core.danger.<level>.meaning`; `levelName`, `levelMeaning` (`product/finding-row/texts.ts`); the level icons `ShieldCheck`, `Shield`, `TriangleAlert`, `OctagonAlert` (the card's `ICONS`) | 003      |
| The build                    | `fresher_core::Build::Sandbox { path, simulated }` (`crates/core/src/app_info.rs`), made by `app/tauri/src/sandbox.rs::build`; the mode's `SandboxTrash::{Sandbox, Real}` (`home/.Trash` or the system's); `BuildBanner` (`ui.build-banner.*`) | 007      |
| The core's key checks        | `crates/core/src/texts.rs`: `core_keys()` lists every key the core can emit; `no_key_in_the_file_is_unused`, `every_placeholder_matches_the_args_the_core_passes`, `the_other_texts_have_no_placeholders` | 002, 011 |
| The window                   | `minWidth` 900, `minHeight` 600 (`app/tauri/tauri.conf.json`, `tauri.dev.conf.json`)                                                              | 003      |

## Design

### 1. Navigation and the frame (plan 03)

```ts
// state/navigation.ts
export type Screen =
  | { readonly category: CategoryId; readonly kind: 'category' }
  | { readonly cleanup: CleanupId; readonly kind: 'cleanup' }
  | { readonly kind: 'cleanups' }
  | { readonly kind: 'review' }
  | { readonly kind: 'scan' }
  | { readonly kind: 'settings'; readonly tab?: IgnoredTab };
```

- `module` leaves; `category` arrives (03/01); `overview` is renamed `scan` (03/02). `OverviewView`, `PlaceAt`,
  `placeTop`, `getOverviewView`, `showOverviewView` and `useOverviewView` leave (02/03).
- `screenToShow(screen, categories: null | readonly CategoryId[])`: a `category` screen of a category no listed module
  has falls back to Scan. `useScreen(categories)`; the frame passes every descriptor id of `useModules()`.
- `screenKey`: `category:<id>`; `scan`, `review`, `cleanups`, `settings` by kind; `cleanup:<id>`.
- The frame's `contentOf`: `category` → `<CategoryScreen category module />` (the module found by
  `moduleOfCategory(modules, id)` in `screens/use-modules.ts`, the label the category's name); `null` while the
  modules are not known.
- `modules/registry.ts` (custom module views, none registered) leaves with the module pages; `app/ui/src/modules/`
  is deleted.
- **The sidebar** (`screens/frame-sidebar.tsx`): Scan (icon `LayoutGrid`, active on `scan` and on `category`), Review
  (with the selection's count, as today), Cleanups; Settings in the footer. No module sections; its `modules` prop
  leaves. `design/layout/sidebar/`: a scroll container `.scroll` (flex 1, `min-height: 0`, `overflow-y: auto`, its
  own position for the indicator) holds the indicator, the sections and the footer, the footer pushed to the bottom
  with `margin-top: auto`; the strip stays outside it. The indicator's `offsetTop` is then measured inside the
  scrolled content, so it stays right while scrolled. Settings is reachable at any height by scrolling.

### 2. The disk ring (plan 01 step 03, plan 02 step 01)

#### 2.1 What the ring draws

The hero's ring draws the home volume (`storageMap.volumes[0]`) from its parts, clockwise from the top:

1. **The cleanable part, split by danger level**: up to four segments, Safe, Low, Medium, High (decision 2), each in
   its level's colour; a level with nothing is not drawn.
2. **The volume's other parts**, in the core's order, each of zero size dropped: Your folders, Not scanned yet (during
   a scan), the remainder (Not measured / Outside the scanned folders), macOS, Swap and sleep image, Another volume,
   The disk's own space.
3. **Free**: the last part is not drawn. Its slot is the gap where the ring does not close, and the label "Free"
   (`core.storage.part.free.name`) sits at the middle of the gap, just outside the band (decision of the draft,
   item 3).

The ring's whole is the sum of its parts' allocated sizes (as the map block drew it, so a disk that changed during the
scan still fills the circle). Before a scan and after a failed one the ring is empty (the track only), as today; the
Scan button and the error keep their places.

#### 2.2 The split by level

`product/scan-hero/ring-parts.ts`:

```ts
// The cleanable bytes of the volume by danger level, from the core's answers only.
export const levelShares = (
  groups: null | readonly MeaningGroup[],
  totals: readonly CategoryTotal[],
): ReadonlyMap<DangerLevel, Sizes>;
```

- When the core's by-meaning groups of the volume hold category shares (`kind: 'category'`, and each app's
  `categories`), each share is added to the level of its category's total (`CategoryTotal.highestDanger`). These
  shares are exactly the volume's Explained bytes, every byte in one leaf (the engine's `meaning`). A share whose
  category the totals do not know (an ignored category, a hidden module) is not cleanable and is left out.
- Otherwise (before the core answers, during a scan, when the query failed) the totals themselves are summed by
  `highestDanger`: during a scan Explained is the findings so far, which are the totals.
- Sums follow the core's rule: bytes add up, the weakest completeness wins (the `plus` of `meaning-list.tsx` moves to
  `product/sizes.ts` as `addSizes`).

This is presentation of the core's per-category sizes and levels, the same grouping `summarize` already does for the
selection (agent's call 3). The core is not asked for a new query.

```ts
export type RingPart = {
  readonly about: RingAbout;
  readonly key: string; // `level-safe` … `level-high`, `<part kind>-<index>`
  readonly label: string; // core.danger.<level>.zone, or partName(kind)
  readonly sizes: Sizes;
  readonly tone: Tone;
};

// What the legend's "i" shows: a level's (Explained's description and the level's meaning) or a part's.
export type RingAbout = { readonly kind: 'level'; readonly level: DangerLevel } | { readonly kind: 'part'; readonly part: MapPart };

export const ringParts = (volume: VolumeMap, levels: ReadonlyMap<DangerLevel, Sizes>): readonly RingPart[];
export const ringSegments = (parts: readonly RingPart[]): readonly Segment[];
```

#### 2.3 Colours

Every segment has its own colour (draft item 5, decision 8). `design/data/segment.ts`:

```ts
export type Tone = 'disk-own' | 'free' | 'mac-os' | 'other-volume' | 'swap-and-sleep' | 'unknown' | 'your-folders' | Level;
```

`plain` becomes `your-folders`; `system` splits into `mac-os`, `swap-and-sleep`, `other-volume`, `disk-own`. New
semantic tokens in `design/tokens/semantic.css`, beside the level tokens: `--part-your-folders`, `--part-mac-os`,
`--part-swap-and-sleep`, `--part-other-volume`, `--part-disk-own` (`--tone-plain` and `--tone-system` are replaced by
them; `--tone-free-outline` and `--tone-unknown-stroke` stay). The neutral parts carry no light (goal, section 6): five
graphite shades and muted, low-chroma hues (slate, plum, teal, stone) distinguishable from each other and from the
four level colours, each at least 3:1 against `--elevation-1-surface` (non-text contrast); the contrast spec checks them.
`partTone` in `product/storage-map/parts.ts` maps each kind to its tone; the other volumes' map blocks
(`StorageMap`, `UsageBar`) take the same tones. Two other volumes share `other-volume` (agent's call 6).

#### 2.4 The legend and hover

`product/scan-hero/ring-legend.tsx`, beside the ring where the stat tiles and the usage bar were (decision 3):

- One row per ring part, Free included, in the ring's order: a swatch in the part's tone (Free an outline, the
  unknown hatched), the label, the size (`SizeText` with the honest size; while scanning an `AnimatedNumber` read as
  the honest size, as `PartLine` does), and an "i" `IconButton` (`Info` icon, label "What is {name}?").
- The "i" toggles the row's explanation below it (a disclosure: `aria-expanded`, `aria-controls`; agent's call 4):
  for a level, `core.storage.part.explained.description` then `core.danger.<level>.meaning`; for a part,
  `partDescription(kind)`, the remainder's reasons (the `Reason` list of `part-line.tsx`, moved to
  `product/storage-map/part-reasons.tsx` and used by both), and for Free the purgeable note
  (`core.storage.purgeable`).
- Hover linking: `StorageRing` takes `highlighted: null | string` and `onHighlight: (key: null | string) => void`.
  An arc's pointer enter or leave calls `onHighlight`; while a key is highlighted the other arcs dim (`data-dimmed`,
  opacity 0.35) and the legend row with that key is marked (`data-highlighted`, a soft
  background). A legend row's pointer enter or leave, and focus entering or leaving the row (its "i"), call the same.
  The Free gap highlights its label. Under reduced motion the dimming does not transition.
- The hero's caption above the ring and legend replaces the map block's title "Data · internal" (draft item 6):
  `ui.scan-hero.caption` "Space on {name}, {kind} disk" (`kind` from `core.storage.kind.<kind>`). The disk-changed
  note (`core.storage.changed`) shows under the legend when `diskChanged`.
- The ring's accessible name: `ui.scan-hero.ring` becomes "Space on {name}"; `chartLabel` adds every drawn segment and
  its size; the legend is the readable equivalent.

Other volumes keep their collapsed `StorageMap` block under the hero, unchanged but for the tones (decision 4).

### 3. Danger zones (plan 02 step 02, plan 03 step 03)

`product/danger-zone/danger-zone.tsx`:

```ts
export type DangerZoneProps = {
  readonly children: ReactNode;
  readonly level: DangerLevel;
  // Scan's "Select all" for the zone; absent on Review, where everything listed is already selected.
  readonly selectAll?: { readonly checked: 'mixed' | boolean; readonly onChange: () => void } | undefined;
};
```

- A `<section aria-labelledby>` with a head: for Medium and High the level's warning icon (`TriangleAlert`,
  `OctagonAlert`: both carry an exclamation mark) in the level's colour, `aria-hidden`; the zone's name as an `h2`
  (`core.danger.<level>.zone`); the core's sentence of what the level means (`core.danger.<level>.meaning`); and, on
  Scan, a `Checkbox` "Select all" (`ui.danger-zone.select-all`) with the description
  `ui.danger-zone.select-all.description` ("Ticks every category in this zone. Nothing is removed until you confirm
  it in Review."), described by the zone's name.
- A thin rule in the level's colour separates the zones; Safe first, High last (decision 1). An empty zone is not
  shown.
- `product/danger-zone/zones.ts`: `ZONE_ORDER: readonly DangerLevel[] = ['safe', 'low', 'medium', 'high']` and the
  level icons (`LEVEL_ICONS`, moved from the card, which imports them from here).

On **Scan** (`screens/scan/zones.ts`):

```ts
export type Zone = { readonly level: DangerLevel; readonly totals: readonly CategoryTotal[] };
// The categories by their highest level, in ZONE_ORDER, empty zones left out; each zone's totals in their order.
export const zonesOf = (totals: readonly CategoryTotal[]): readonly Zone[];
// A zone's "Select all": checked when every selectable category of it is selected whole, mixed when some are (or
// some of their findings), unchecked otherwise; `null` when nothing in it can be selected.
export const zoneSelectAll = (
  selection: Selection,
  zone: Zone,
  selectable: (category: CategoryId) => boolean,
): null | { readonly categories: readonly CategoryId[]; readonly checked: 'mixed' | boolean };
```

A selectable category has findings (`count > 0`) and is not unavailable. Turning "Select all" on dispatches
`{ type: 'choose', categories, within: categories }`; turning it off (from checked) dispatches
`{ type: 'choose', categories: [], within: categories }`; from mixed it turns on. Each zone holds one `CategoryGrid`
labelled by the zone's name.

On **Review** (`screens/review/review-groups.ts`): `reviewZones(groups)` → `{ level, groups }[]` by each group's
`highest`, in `ZONE_ORDER`, empty zones left out, each zone's groups in their largest-first order. Each zone is a
`DangerZone` without `selectAll`; the groups' sections become `h3`.

### 4. Cards and the menu (plan 01 step 02, plan 02 step 02)

**The menu primitive**, `design/primitives/menu/` (agent's call 5): a menu button.

```ts
export type MenuItem = {
  readonly description?: string;
  readonly key: string;
  readonly label: string;
  readonly onSelect: () => void;
};

export type MenuProps = {
  readonly icon: IconShape;
  readonly items: readonly MenuItem[];
  // The trigger's accessible name.
  readonly label: string;
};
```

- The trigger is an `IconButton` (ghost, small) with `aria-haspopup="menu"`, `aria-expanded`, `aria-controls`
  (`IconButton` gains optional pass-through of these and a `ref`). Click, Enter, Space or ArrowDown opens on the first
  item; ArrowUp opens on the last.
- The popup is `role="menu"` in a portal, `position: fixed`, placed by `placeMenu` (`menu/place.ts`): under the
  trigger, its right edge on the trigger's right edge, above it when there is no room below, always inside the
  window (8 px margin). Items are `role="menuitem"` with roving `tabIndex`: ArrowDown/ArrowUp move and wrap, Home/End
  jump; Enter or Space selects. Each item shows its label and, under it, its description in secondary text (the
  item's accessible description).
- Selecting an item closes the menu, gives the focus back to the trigger, then calls `onSelect`. Escape closes and
  gives the focus back; Tab closes and lets the focus move on; a pointer down outside closes without moving the
  focus. It fades in over `--duration-fast`; under reduced motion it just appears. Its layer is the tooltip's.

**The card** (`product/category-card/`):

- No movement on hover or press (draft item 7): the `translateY(-1px)` and the press scale go; hover brightens the
  border only (its width never changes) and the spotlight stays.
- The card is no longer one button: its body (icon, name as an `h3`, what it is, the level, size and count, the
  share bar) is static; the checkbox stays in its corner. A row of actions at its bottom right: **Details** (`Button`,
  small, ghost, `ui.category-card.details` "Details", described by the card's name) and the **settings** menu
  (`Menu`, icon `Settings`, label `ui.category-card.options` "Options for {name}") with:
  - "Ignore this category" (`ui.category-card.ignore-category`), description
    `ui.category-card.ignore-category.description` ("Hides it from the results and never removes it. You can restore
    it in Settings.");
  - "Ignore {module}" (`ui.category-card.ignore-module`, the module's name `<module>.enabled.label`), description
    `core.ignore.modules.description`; absent when the category's module is not known.
- The size is the whole category's (`CategoryTotal.sizes`, draft item 7); `underApps` and `ui.category-card.under-apps`
  leave.
- An unavailable card keeps Details (it explains why) and the menu, and has no checkbox.
- Props: `access`, `menu: readonly MenuItem[]`, `onDetails`, `onToggle`, `selected`, `share`, `total`.
  `CategoryGrid` takes `label`, `onDetails(category)`, `onIgnore(target: IgnoreTarget)` and
  `moduleOf(category): null | { readonly id: ModuleId; readonly name: string }` and builds each card's menu.

Ignoring from the menu calls `useIgnoreCommands().ignore(…)`; the core's `setting-changed` refreshes the overview as
it does today, so the card leaves its zone. Restoring stays in Settings (draft item 7).

### 5. The category details screen (plan 03 step 01)

`screens/category/category-screen.tsx` (`CategoryScreen { category: CategoryId; module: ModuleInfo }`), built from
today's category section and module page (draft item 6, decision 5):

1. Toolbar: "Back to Scan" (`ui.screen.category.back`, ghost, icon `ChevronLeft` added to `icons.ts`); Scan or Stop as
   on the module page today.
2. `PageHeader`: the category's name; subtitle `<category>.what`.
3. The level: a large `DangerBadge` of the category's level (`total.highestDanger`, else `descriptor.danger`) with the
   core's meaning as its description, and the zone's name.
4. When unavailable, `UnavailableReason`.
5. What it is, shown, not behind a disclosure: the four sentences (`CategoryAbout`, `screens/category/category-
   about.tsx`, the `About` of `category-section.tsx` without its `details`).
6. The left-out lines of this category (`LeftOutLines`, from the module's outcome in the store).
7. The module's choices that touch this category (`ModuleChoices` gains a `category` prop: only choices whose
   `categories` include it; nothing when none).
8. The findings: a head with the category's checkbox (`checkedOf`, `toggle-category`, label "Select all {name}"),
   the count and the size; then `CategoryFindings` (moved from `category-section.tsx` to
   `screens/category/category-findings.tsx`) in a frame of up to 12 rows. With nothing found, an empty state
   (`ui.screen.category.empty.*`).
9. The selection bar.

`screens/module/` is deleted: `module-screen.tsx`, its CSS and test, `category-section.tsx`; `module-choices.tsx`,
its CSS and test, and `unavailable-reason.tsx` move to `screens/category/`. `product/module-section/` is deleted.
`screens/use-ignored.ts` and `moduleFound` of `screens/use-modules.ts` leave if nothing else uses them (knip says).

### 6. The selection bar (plan 02 step 03, plan 03 step 01)

`SelectionBar` (count, size, highest level, Clear, Review) renders on Scan and on a category's details, as on the
module page today: Clear dispatches `clear`, Review navigates to Review. Both screens' content keeps room at the bottom
for it (`calc(var(--space-12) * 2)`).

### 7. The sandbox's Trash (plan 01 step 01)

```rust
// crates/core/src/app_info.rs
pub enum Build {
    Release,
    Development,
    /// … `trash` says where its cleanups move items, as the sentence the banner and a cleanup's result show.
    Sandbox { path: PathBuf, simulated: Vec<Simulated>, trash: Text },
}

/// Where a sandbox's cleanups move items.
pub enum SandboxTrashAt<'a> {
    /// The sandbox's own Trash folder.
    Sandbox(&'a Path),
    /// The Trash of this Mac (`--real-trash`).
    System,
}

impl SandboxTrashAt<'_> {
    /// `core.build.trash.sandbox` with `{path}`, or `core.build.trash.system`.
    pub fn text(&self) -> Text;
}
```

`app/tauri/src/sandbox.rs::build` takes the mode's `SandboxTrash` and the sandbox's Trash folder (the same path the
sandbox's Trash port is given, from the one place that builds it). The bindings are regenerated. The banner shows
`Sandbox · {path} · <the trash sentence>` (and the simulated parts as today); `ResultSummary` gains
`trashNote?: string`, shown under its lines when the method is the Trash; `CleanupPanel` passes it from `useAppInfo()`
in sandbox mode.

### 8. Texts

New in `crates/core/locales/en.json` (decision 6: the core gets only texts):

| Key                                | English                                                                 |
| ---------------------------------- | ----------------------------------------------------------------------- |
| `core.danger.safe.zone`            | Safe to remove                                                          |
| `core.danger.low.zone`             | Low risk                                                                |
| `core.danger.medium.zone`          | Medium risk                                                             |
| `core.danger.high.zone`            | High risk                                                               |
| `core.build.trash.sandbox`         | Cleanups move items to the sandbox's Trash: {path}                      |
| `core.build.trash.system`          | Cleanups move the sandbox's items to the Trash of this Mac.             |

Changed: `core.storage.part.your-folders.description` → "Files Fresher looked at but does not recognise. Fresher
never suggests removing them." (its "open a folder" pointed at the view by place). "Free" exists
(`core.storage.part.free.name`). `texts.rs`'s `core_keys()` lists the zone keys beside the danger names and the two
trash keys from `SandboxTrashAt`'s samples; `texts_with_args()` gains the sandbox sentence.

New and renamed `ui.*` keys (in `app/ui/src/locales/en.json`) are listed in each step; keys no longer used leave in
the step that stops using them (`ui.meaning-list.*`, `ui.place-list.*`, `ui.screen.overview.view.*`,
`ui.scan-hero.stat.*`, `ui.scan-hero.usage`, `ui.screen.module.*` that the details screen does not reuse,
`ui.screen.frame.group-ignored`, `ui.screen.frame.module-ignored`, `ui.category-card.open`,
`ui.category-card.under-apps`).

### 9. What leaves the UI

| What                                    | Files                                                                                                    | Step  |
| --------------------------------------- | -------------------------------------------------------------------------------------------------------- | ----- |
| The stat tiles and the hero's usage bar | `design/data/stat-tile/` (component, CSS, story, baselines); the hero's `UsageBar` use                    | 02/01 |
| The home volume's map block             | the first `StorageMap` on the overview (the component stays for other volumes)                           | 02/01 |
| The view switch and the view by place   | the `SegmentedControl` use on the overview; `screens/overview/by-place.tsx`; `product/place-list/` (all); `useStoragePlace`, `useStorageRoots`, `createPlaceSource` of `state/storage.ts` and their tests; `OverviewView`, `PlaceAt` | 02/03 |
| The rows below the cards                | `product/meaning-list/` (all: list, app row, part and place rows, story data, stories, baselines)          | 02/03 |
| The module pages                        | `screens/module/module-screen.tsx` (+ CSS, test), `category-section.tsx`, `product/module-section/`, `modules/registry.ts`; the sidebar's module sections | 03/01 |

The core's queries stay (decision 6). `docs/deferred/place-queries-without-ui.md` (new, 04/01) records that
`get_storage_place` and `get_storage_roots` have no UI and that `get_storage_meaning` is used only for the ring's
split, due at the next plan that shows the disk by folder; it absorbs
[`way-up-from-meaning.md`](../../deferred/way-up-from-meaning.md) and
[`roots-explained-line.md`](../../deferred/roots-explained-line.md), whose UI is gone (their rows and files leave).

### 10. Tests

- **Unit (vitest)**, negatives first in `describe('negative cases')`: the menu's keyboard and focus contract;
  `placeMenu`; `arcsOf`'s undrawn Free slot; the ring's highlight and gap label; `levelShares`, `ringParts`;
  the legend's rows, disclosure and highlight; `zonesOf`, `zoneSelectAll`; `DangerZone`; the card (no button body,
  Details, menu items, whole size); `reviewZones`; `screenToShow` with categories; the sidebar's items; the
  category screen; the banner's and the result's trash line; the core-keys test of `storage-map` for the changed
  description and the zone keys.
- **Rust**: `app_info.rs` (a sandbox build serialises its trash sentence; release and development carry none),
  `texts.rs` (the keys and placeholders), `app/tauri/src/sandbox.rs` tests (the sandbox's own Trash gives its path,
  `--real-trash` the system's sentence), `commands.rs`'s `app_info` test, the bindings test.
- **E2E (Playwright)**: every spec that opens a module page, the view by place or the overview is rewritten in the
  step that removes or renames what it used (`flows`, `axe`, `visual`, `keyboard`, `contrast`, `motion`, `zoom`,
  `layout`, `perf` where they name it); the gallery baselines of every changed story are regenerated; new e2e cases:
  ring ↔ legend hover, the card's menu by keyboard, "Select all" of a zone, the details screen and back, the
  selection bar on Scan, the sidebar reachable in a short window.

### 11. Where the code lives

| Plan                         | Files                                                                                                     |
| ---------------------------- | --------------------------------------------------------------------------------------------------------- |
| 01 Foundations               | `crates/core/src/{app_info,texts,lib}.rs`, `crates/core/locales/en.json`, `app/tauri/src/{sandbox,compose,commands}.rs` (+ `sandbox/tests`), `app/ui/src/ipc/bindings.generated.ts`, `app/ui/src/ipc/mock.ts`, `app/ui/e2e/core-mock.ts`, `app/ui/src/product/{build-banner,result-summary}/`, `app/ui/src/screens/review/cleanup-panel.tsx`, `app/ui/src/design/primitives/{menu,icon-button}/`, `app/ui/src/design/data/{segment.ts,storage-ring/,usage-bar/}`, `app/ui/src/design/tokens/semantic.css`, `app/ui/src/product/storage-map/parts.ts` |
| 02 The Scan screen           | `app/ui/src/product/{scan-hero,danger-zone,category-card,category-grid,storage-map}/`, `app/ui/src/product/sizes.ts`, `app/ui/src/design/data/stat-tile/` (deleted), `app/ui/src/screens/overview/`, `app/ui/src/state/{navigation,storage}.ts`, `app/ui/src/product/{meaning-list,place-list}/` (deleted) |
| 03 Details and navigation    | `app/ui/src/screens/{category,scan,review}/`, `app/ui/src/screens/{app-frame,frame-sidebar,screen-stack}.tsx`, `app/ui/src/screens/use-modules.ts`, `app/ui/src/design/layout/sidebar/`, `app/ui/src/state/navigation.ts`, `app/ui/src/screens/module/`, `app/ui/src/product/module-section/`, `app/ui/src/modules/` (deleted) |
| 04 Docs and finish           | `docs/plans/{012-storage-map,015-module-groups,027-scan-screen}/`, `docs/architecture/README.md`, `docs/manual-testing.md`, `docs/testing.md`, `docs/deferred/`, `docs/plans/README.md` |

`app/ui/src/locales/en.json`, the e2e specs and their baselines are touched by most steps: a second step to merge
rebases on the first.

## Interfaces with other plans

- Plan 012 (the storage map): its map block of the home volume, its view by place and its by-meaning rows leave the
  UI; the map's parts become the ring and its legend; the core's queries stay (deferred row).
- Plan 015 (module groups): the module pages and the sidebar's groups leave; ignoring a module moves to the card's
  menu, restoring stays in Settings.
- Plan 007 (sandbox mode): `Build::Sandbox` gains `trash`; the banner says where the Trash is.
- Plan 026 (the development run record): the record's session carries the build as serialised, so it gains the
  trash sentence; nothing else changes.
- Plan 014 (onboarding, draft): its first-scan screen will start from Scan, not the overview.

## Open questions

None.

## Decisions

The user's, of 2026-11-20 (translated): the feedback items 1 to 18 as the draft lists them, and the draft approved
with its eight agent's calls, which became these decisions:

1. **Zone order**: Safe first, High last.
2. **The cleanable part of the ring is split by danger level** (four segments in the zones' colours), not one segment
   per category.
3. **The stat tiles and the usage bar beside the ring go**; the legend takes their place.
4. **Other volumes keep their collapsed line under the ring.**
5. **The module choices move to the details of the categories they touch.**
6. **The view by place and the apps' rows go from the UI only**; the core's queries stay (a row in `docs/deferred/`);
   the core gets only new texts.
7. **The window's minimum height stays 600 px**; the sidebar scrolls.
8. **Two categories sharing a colour is fixed by the ring's new segments and the legend's own colours.**

No version bump: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Made while writing this design from the code; each with what it costs if wrong.

1. **The sandbox's Trash sentence travels in `Build::Sandbox { trash: Text }`** (the draft said the core gets only
   texts): the UI cannot know whether `--real-trash` was given, and building `<path>/home/.Trash` in the UI would be
   a path made by concatenation. One field of `Build`, filled by the shell. Cost if wrong: one field to drop.
2. **The sandbox line's texts are core keys** (`core.build.trash.*`), as the draft says, while the rest of the banner
   stays `ui.*`. Cost if wrong: two keys move.
3. **The ring's split by level is summed in the UI from the core's category shares of the volume and the totals'
   levels**, falling back to the totals during a scan; no new core query. A category the totals do not know is left
   out of the ring (it is not cleanable). Cost if wrong: a core field `levels` on the map's Explained part later.
4. **The legend's "i" is a disclosure under its row, not a floating popover**: no positioning, the reasons list fits,
   and it reads in order. Cost if wrong: a popover primitive later.
5. **A menu primitive in the design layer** (`design/primitives/menu/`), the app's first; the card's settings button
   uses it. Cost if wrong: none; it is reusable.
6. **Two other volumes on one disk share the `other-volume` colour**: they are rare and each is named in the legend.
   Cost if wrong: an indexed shade.
7. **The card's body is not a button any more**: Details is its one way to the details (one control per action);
   the checkbox stays in its corner. Cost if wrong: a click on the body could open Details too.
8. **The Free gap's label sits outside the band at the gap's middle**, the legend's Free row carrying the size.
   Cost if wrong: a styling change.
9. **The zone names are new texts** ("Safe to remove", "Low risk", "Medium risk", "High risk") rather than the bare
   level names: "Low" alone does not read as a heading. The legend's level rows use the same names. Cost if wrong:
   four strings.
10. **Review's zones have no "Select all"**: everything listed there is selected; Clear stays in its toolbar. Cost
    if wrong: one prop.
11. **The details screen shows the four sentences open** and its findings in a frame of up to 12 rows (the module
    page showed 8): the screen is about this one category. Cost if wrong: two constants.
12. **The rename of Overview to Scan is its own step after the details screen** (03/02), so the steps before it
    touch fewer files at once. Cost if wrong: none.
13. **The hero's caption is a sentence**, "Space on {name}, {kind} disk", not "Data · internal". Cost if wrong: one
    string.
14. **`modules/registry.ts` (custom module views) is deleted** with the module pages: nothing registers one, and a
    module's own view would now be a category's details. Cost if wrong: a file to bring back.
15. **The two deferred items about the by-place view are folded into the new deferred item** and their files
    removed: their UI is gone. Cost if wrong: two files to restore from history.
16. **`StatTile` is deleted from the design layer**, `UsageBar` stays (other volumes' map blocks use it). Cost if
    wrong: a component to bring back.

## Conclusions

Built; the numbers and every deviation from this design are in [`readme.md`](./readme.md), "Conclusions". Of the
sixteen agent's calls, 1 to 8, 10 to 14 and 16 stand as written; call 9 (the zone names as new texts) stands with the
details' heading "Danger level"; call 15 stands. Two things differ: Review keeps no "Select all" (call 10) while every
zone on Scan, Medium and High included, has one; and the details' module choice ticks the module's other
categories too. Awaiting the user's walk.
