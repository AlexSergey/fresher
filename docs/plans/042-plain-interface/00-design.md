# 042 — A plain interface: design

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 ·
**Modules:** core, engine, modules, testing, sandbox, e2e, app, app/tauri, app/ui, scripts, docs

The design the plans argue from. Written from the approved draft of 2026-11-24 and a read of the code on `main`
(b12959dc). The visual spec is the approved mockups, [`docs/mockups/0.1.0/`](../../mockups/README.md): their markup
gives the sizes, colours and texts; the data in them is made up. Each step reads its code again before it changes it.

## Intent

The user's task (translated, from the manual test of 2026-11-24,
[`../../manual-testing/2026-11-24-feedback.md`](../../manual-testing/2026-11-24-feedback.md), items 1-27): "The UI
looks bad and I do not understand what is happening on the screens: it must be simplified. A more laconic,
understandable interface, with short clear descriptions and a clear separation of sections. A balance between how much
data a screen shows and how beautiful it is. For the first release, hide High entirely. Settings is overloaded: a
sidebar with its structure and a global search." Then: "I think the new UI suits us, start implementing; work
autonomously with subagents, take the recommended decisions."

## Goals

1. Scan answers "what can I remove, and how much" at a glance: a compact disk card with a coloured ring, a separator,
   then tiles grouped by what removing them costs.
2. Review shows what goes and what the user has checked, category by category, items apart; a "Look before removing"
   item goes only when the user ticked it by hand.
3. Every text a person reads is short and plain.
4. Settings has its own sidebar (a tree of sections) and a search that jumps to a setting; numbers are sliders.
5. No High finding is offered in the first release, and the core enforces it.

## Non-goals

- New categories or modules; changing a category's level (only its texts, its icon and its one line).
- A light theme, translations, onboarding (014).
- Removing High from the code: the categories, their tests and the High consent of 039 stay, switched off.
- A user setting for High (agent's call 7).
- A new scan algorithm, or any change to `crates/scanner`, `crates/macos`, `crates/system`, `crates/store`,
  `crates/rules`, `crates/cleaner`.

## What already exists (read on b12959dc)

| What | Where | Plan |
| --- | --- | --- |
| Danger levels | `DangerLevel` in `crates/core/src/danger.rs`; a finding may be raised above its category's level (`Finding::raise_danger`) | 002 |
| High categories | `app-leftovers.data`, `downloads.large-untouched`, `xcode.archives`, `demo.volumes`, `core.own-file` (Largest picks), `core.uninstall-everything`; Docker volumes are High but only described | 019-036 |
| What the user sees | `Results::shown` (`crates/engine/src/results.rs`): every page, finding, total, the found size and `explained_under` go through it; the ignore lists filter there | 015 |
| The map's shares | `Shares::of(state, ignored)` (`crates/engine/src/storage/shares.rs`): an ignored owner's bytes are "your folders" | 012 |
| Every prepare's tail | `rehearse_and_keep` (`crates/engine/src/cleanup/pending.rs`), reached by `prepare` and the uninstall's prepare | 036 |
| Largest picks | `Engine::pick_own_item` (`crates/engine/src/own.rs`), always High | 035 |
| Uninstall choices | `UninstallChoice::{KeepData, Everything}` (Everything is High); the app page lists `UninstallChoice::ALL` (`crates/engine/src/apps/page.rs`) | 036 |
| The engine's constructors | `Engine::new(registry, ports)`, `Engine::with_cleanup(registry, ports, cleanup)`: 14 call sites (`app/tauri/src/compose.rs::assemble`, `app/tauri/src/testing.rs`, `crates/engine/src/testing.rs`, two engine tests, nine in `crates/e2e/tests/`) | 002 |
| The app's info | `AppInfo { version, build }` (`crates/core/src/app_info.rs`), command `app_info` | 038 |
| The storage map | `VolumeMap` and nine `PartKind`s (`crates/core/src/storage/map.rs`), built by `volume_map` (`crates/engine/src/storage/volume.rs`); texts `core.storage.part.<kind>.{name,description}` | 012 |
| A category | `CategoryDescriptor { id, danger, needs }` (`crates/core/src/category.rs`), five texts (`CategoryText`), held by the module contract (`crates/testing/src/contract.rs`) | 002 |
| Zone texts | `core.danger.<level>.{name,zone,meaning}`; 039's `core.danger.part.{safe,danger}.{name,meaning}` | 027, 039 |
| Scan progress | `Event::ScanProgress { scan, entries, bytes }` (`crates/engine/src/session.rs`) | 004 |
| Scan screen | `screens/scan/` (`ScanZones`, `SelectAllQuestion`), `product/scan-hero/` (ring by level), `product/category-card/`, `product/storage-map/` (other volumes), `product/scan-progress/` (module chips) | 027 |
| Review | `screens/review/` (`reviewGroups`, `reviewZones`, `reviewParts`, `review-part.tsx`), `product/danger-zone/` (`ZONE_ORDER`, `PART_OF`, `LEVEL_ICONS`) | 027, 039 |
| Selection | `state/selection.ts` (categories whole minus exclusions, findings one by one), `state/selection-summary.ts`, `state/reduce.ts` | 003, 027 |
| Category details | `screens/category/` (`CategoryAbout`: four sentences in a list), `product/finding-row/` (title, path, level badge, size) | 027 |
| Settings | `screens/settings/settings-screen.tsx`: one long page of groups; `product/settings-row/` (`NumberField` for numbers) | 005 |
| Menus | `design/primitives/menu/` (hover = `--surface-hover`) | 027, 028 |
| Design tokens | `app/ui/src/design/tokens/{primitive,semantic,motion}.css`; `contrast.test.ts` | 003 |
| Icons | `lucide-react` through `design/icons/icons.ts` | 003 |
| The logo | `docs/mockups/0.1.0/logo.svg`, `assets/logo.png` (done 2026-11-24); `app/tauri/icons/` (old); `assets/social-preview.svg` embeds `logo.png` | 040 |
| Demo scenarios | `fixtures/demos/scenarios/{overview,cleanup}.toml`, targets traced to text keys (`crates/sandbox/src/scenario/traced.rs`); no GIF committed yet | 038 |

## Design

### 1. High off for the first release (plan 01, steps 01 and 02)

- `HighLevel::{Offered, Withheld}` (`crates/core/src/danger.rs`, kebab-case in JSON). The engine takes it as a
  required argument of both constructors: no default, so no test and no app can forget it. `Engine::high_level()`
  answers it.
- **Withheld** — a High finding never leaves the engine:
  - `Results::shown` leaves out a finding whose own level is High (raised findings too) or whose category is High: so
    every page, `finding`, total, the found size, `explained_under` and the app categories leave it out.
  - The shares count a High owner as not counted, as an ignored one: its bytes stay in the user's folders, so "Can be
    freed" never holds them (a High category's owners, and the paths of findings raised to High).
  - `modules()` leaves High categories out of each module's categories and out of its choices (a choice left empty is
    dropped); `ModuleFinished`'s left-outs of a High category are not published.
  - Refusals, new `CommandError::HighWithheld` (`core.error.high-withheld`): the tail of every prepare when an item is
    High (before the agreement and the settings file are checked), `pick_own_item`, and `prepare_uninstall` with
    `Everything`. The app page's choices leave out `Everything`.
- **Offered** — everything as today.
- The app sets `Withheld` in `assemble` for the normal engine and for sandbox mode, in development and release builds
  alike, so the user's walk sees what ships. The sandbox autotest runs `Offered` (the goldens of the High categories
  stay as they are), except a case file that says `high = "withheld"`: the new `fixtures/sandboxes/high-withheld.toml`
  proves nothing High is offered. A case file's `EXPECTED.md` names a High positive as "not shown in the app while High
  is withheld".
- `AppInfo` gains `high: HighLevel`: the UI hides what has no meaning without High (the sidebar's Largest) and tests the
  rest (the app page's choices come from the core).

### 2. The disk's five parts (plan 01, step 03)

- A summary beside the detailed parts: `VolumeMap.summary: Vec<SummaryPart>`, always five, in this order, built in
  `VolumeMap::new` from its parts, adding up to the capacity:

  | `SummaryKind` | Name | From the parts | "i" (description) | Colour |
  | --- | --- | --- | --- | --- |
  | `can-be-freed` | Can be freed | `Explained` | What Fresher found that you can remove, sorted below by what removing it costs. | `#3dd68c` |
  | `files-and-apps` | Your files and apps | `YourFolders`, `NotScannedYet`, `Remainder` (both), `OtherVolume` | Your documents, apps and everything else Fresher does not offer to remove. | `#5b7cfa` |
  | `mac-os` | macOS | `MacOs` | The system itself, which macOS looks after; Fresher never touches it. | `#b18cf5` |
  | `system-space` | System space | `SwapAndSleep`, `DiskOwn` | Memory macOS keeps on disk, and space the disk keeps for itself. | `#f07ab0` |
  | `free` | Free | `Free` | Space you can still use for new files. | outline `#5a616d` |

- Texts `core.storage.summary.<kind>.{name,description}`. The detailed parts stay for the by-meaning groups and the
  place view.

### 3. Icons and the one line (plan 01, step 04)

- `CategoryIcon`, a fixed set of 14 keys (kebab-case): `cache`, `build`, `download`, `project`, `toolchain`, `log`,
  `temporary`, `app`, `model`, `editor`, `browser`, `device`, `archive`, `generic`. `CategoryDescriptor::with_icon`;
  `new` gives `generic`. The module contract refuses a module category with `generic`. The UI owns one shape per key
  (`lucide-react`).
- A sixth category text, `<category>.line`: what the tile says under the name, at most 50 characters, no full stop.
  The contract holds six texts.

### 4. The groups (plan 01, step 05)

The level names stay in the code; people read the group names, which reuse the zone keys with new texts (no full
stop: they are lines):

| Level | `core.danger.<level>.zone` | `core.danger.<level>.meaning` | Colour | Heading icon |
| --- | --- | --- | --- | --- |
| Safe | Safe to remove | Apps make these again on their own | `#3dd68c` | shield with a tick |
| Low | Usually safe | You may need to download them again | `#4cc3e6` | shield |
| Medium | Look before removing | May hold something you still want | `#f5b544` | eye |
| High | Only if you are sure | May hold the only copy of your data | `#ff6b70` | shield with a mark |

039's `core.danger.part.*` go with Review's two parts (plan 04).

### 5. The scan's progress (plan 01, step 06; plan 03, step 02)

`ScanProgress` gains `expected: Option<ByteSize>`, the size the scan will most likely walk: the last committed state's
roots' sizes when the roots are the same, else the used space of the roots' volumes, else none. The scanning card
fills the swoosh to `bytes / expected`, held at 99 % until the scan finishes; without an estimate it follows the share
of modules finished.

### 6. Design pieces (plan 02)

Tokens for every colour of the mockups (no raw colour in a component); the primary button's sheen and the menu's
accent hover (every baseline); the section separator, the tile and the category icon badge; the disk card with its ring
and legend; the swoosh progress; the slider.

### 7. Scan (plan 03)

- **Top bar**: "Scan", when it was scanned ("2 minutes ago"), Scan again; Stop while scanning.
- **The disk card**: the ring (Your files and apps, macOS, System space, Can be freed, with Free as the track), "3.8 GB
  can be freed" in its centre; the disk's name, "109 GB free of 494 GB", and the two-column legend with an "i" each.
  A part too small to see gets a minimum arc; hover links a legend row and its arc both ways. Other volumes: compact
  disk cards below it. Before the first scan the centre holds Scan.
- **While scanning**: the scanning card (the swoosh fills with the progress; "Scanning Macintosh HD", the percent,
  "Looking at <module>", the size found so far), then the tiles found so far without controls and one placeholder.
- **The separator** "What you can remove", then **the groups** in order, each a heading (icon, name, "· line"); Safe
  and Usually safe have a "Select all"; Look before removing has none.
- **Tiles**, two columns: the icon badge, name, line, checkbox; size, count, Details, the `⋯` menu; the border tinted
  by the group. The selection bar floats at the bottom. Left-out-only and described-only sections stay below,
  restyled.
- The sidebar leaves out Largest while High is withheld; the logo sits beside "Fresher" (plan 07).

### 8. Review (plan 04)

- **The tick model**: a "Look before removing" category (Medium, or High when offered) ticked on Scan comes to Review
  with nothing ticked (`Selection.toCheck`); each of its items goes only when the user ticks it, in Review or on its
  details. The tile's "all" checkbox in Review is the user's own act. Safe and Usually safe categories come ticked, as
  the user selected them. Nothing in the core changes: the selection the core receives names only what goes.
- **The summary card**: "2.6 GB goes to the Trash · 6 items", "You can put them back from Cleanups", a bar split by
  group, a chip per group with its size; the Look chip says how many items are not checked yet.
- **The separator** "What goes", **one tile per category**: the all-checkbox, its icon, name, "<group> · <line>" (Look:
  "Look before removing · Tick each one you have checked"), a pill "3 of 3 checked" (green when all, amber when not),
  the size; under it every item with its own checkbox, name and short path.
- **The bar**: count, size, Clear, Move to the Trash (inactive while nothing is ticked). The confirmation and the core's
  High consent stay as they are.

### 9. Category details and menus (plan 05)

- Back to Scan; the name, the group (a dot and its name), size and count; three answers side by side (What it is, with
  who makes it under it; If you remove it; Comes back?); each left-out line as an "i" bar; the module's choices; the
  items with checkboxes (the name first, the path small under it, no level badge); "Select all" only for Safe and
  Usually safe.
- Every row and tile menu: Show in Finder (rows), Ignore this item (rows), Ignore all <category>; the pointed
  item fills with the accent colour. No hint, and no "Stop looking for <module>" (user's decisions 2026-11-25: a
  module is turned off only in Settings).

### 10. Settings (plan 06)

- A second sidebar: the search, then General, Scan, Ignored, Permissions, About; then "What it looks for", the module
  groups as a tree (open/close), each module a page (its group, its name, its description, its settings, its on/off
  switch last). Fresher's own data sits on General.
- The search matches labels, descriptions and section names; results list "Group › Module", the setting's name and its
  description with the match marked; the tree dims sections without a match and counts matches; Enter or a click opens
  the page with the setting focused; Esc clears.
- A whole-number setting is a slider from its minimum to its maximum with a number field beside it; refusals stay.

### 11. The logo (plan 07)

The logo (variant A: graphite leaf, iris swoosh `#4553dc` to `#9aa6ff`, near-white star, a 1 px rim of the swoosh's
gradient at 50 % opacity) small beside "Fresher" in the sidebar; the app icons in `app/tauri/icons/` and the social
preview made from `docs/mockups/0.1.0/logo.svg`.

### 12. Wording (plan 08, the last)

Every text of the core, the modules and the UI rewritten to the writing guide, shorter: a category's name 2-4 words, its
line at most 50 characters, each answer one sentence (two only when the second says what you keep). The trial and demo
modules' items get readable names. `writing.md` gains the group names, the five part names, the line's length and a word
list of the replaced terms; `npm run check:texts` checks the line and the name's words, last, when every text passes.

### 13. Tests

- Rust: negatives first. The engine's withheld tests (`crates/engine/src/high_tests.rs`), the core e2e
  `crates/e2e/tests/high_withheld.rs`, the summary's tests, the contract's icon and line checks, the expected size.
- Sandboxes: every case file runs `Offered` but `high-withheld.toml`.
- UI: unit tests for every new piece (negatives first); the e2e mock gains `high` (default `offered`, so every existing
  script stays what the core can produce) and a `withheld` script helper; Playwright baselines for every screen a step
  changes, each looked at.

## Risks

- **A path that still offers High**: `shown`, the shares, the module info, the left-outs and three refusals are the
  sites; `high_tests.rs` asserts each, and the e2e test asserts them on files.
- **Screenshot churn**: the button sheen and the menu hover change most baselines once (02/01); later steps change only
  their screens.
- **The tick model**: a Look item must never go unticked. The reducer owns it, with unit tests; the e2e flows tick
  items in Review as a user would.
- **Texts change twice**: the drafts of 01/04 and 01/05, then the wording pass; the pass never changes a key, a
  placeholder or a level.

## Open questions

None. The draft's two questions are answered (agent's calls 7 and 8).

## Decisions

- 2026-11-24: direction A (a summary and a list); the third mockup approved (the user).
- 2026-11-24: High hidden for the first release, by a switch in the core, all High places go, Largest included (the
  user).
- 2026-11-24: the sidebar of the app stays (the user).
- 2026-11-24: logo variant A; the swoosh on the buttons and as the scan's progress; the logo small in the sidebar; no
  decorative swoosh at the bottom of Scan (the user).
- 2026-11-24: approved mockups live in `docs/mockups/<version>/` (the user).
- 2026-11-24 — the user delegated: build autonomously, the recommended decisions and agent's calls taken; what the
  user must look at goes to `docs/deferred/user-review-2026-11-24.md` (written by 09/01).

## Agent's calls (to check)

1. **The switch is set by the app, not a user setting**; development and release builds both withhold, so the walk
   sees what ships; tests and the sandbox autotest pass `Offered` explicitly (a required argument, no default).
2. **Group names** "Safe to remove", "Usually safe", "Look before removing" (High: "Only if you are sure"); the words
   Safe, Low, Medium leave the screens; the texts reuse `core.danger.<level>.zone` and `.meaning`.
3. **The five parts are merged in the core** as a summary beside the detailed parts (no business logic in the UI).
4. **"Look before removing" items are never ticked for the user**: a Look tile ticked on Scan sends nothing until its
   items are ticked; 039's two Review parts and its Scan question go; a group "Select all" only for Safe and Usually
   safe; Details has no "Select all" for a Look category. Cost: one more piece of selection state.
5. **Largest leaves the sidebar while High is withheld**; Apps stays with "Keep my settings" only.
6. **The plan goes before the audit (041).**
7. **High comes back in a later release** (dated 2026-11-24, the draft's question 1); no user setting now.
8. **Icons: a fixed set of 14 keys** (dated 2026-11-24, the draft's question 2); every category names one in its core
   data; the UI owns the shape of each key; no per-category drawing.
9. **The wording pass is the last implementation plan of 042** (dated 2026-11-24), so the user checks texts on the new
   screens.
10. **A sixth category text, `line`** (at most 50 characters, no full stop), written as a first draft with the icons
    (01/04) and rewritten by the pass. Cost: one more text per category, 50 of them.
11. **The remainder and other volumes count as "Your files and apps"**: most of what a scan cannot read is the user's;
    the permission notices still say what is missing. Cost: the reasons of the remainder leave the disk card.
12. **Withheld also hides** a High category's left-outs and its place in module info and choices, so no card, details
    page or "left out" line names something never offered.
13. **The progress estimate comes from the core** (`expected`), the percent held at 99 % until the end; the module
    chips go. Cost: an estimate can be off on a first scan of a full disk.
14. **Described-only sections (Docker) stay as they are**: described, never offered, at any level.
15. **Details shows who makes it under "What it is"** (rule 3), keeps a scrolling list instead of "N more", and drops
    its Scan button (one control per action: Scan lives on Scan).
16. **A menu item's description becomes its hint** — reversed by the user (2026-11-25): menu items carry no hint and no
    description.
17. **Settings**: Fresher's own data on General; a module's on/off switch last on its page; the search also matches
    section names.
18. **The e2e mock offers High by default** (every script stays what the core can produce); the new specs use a
    `withheld` helper.
19. **"2 minutes ago"** is formatted by the UI's i18n (`Intl.RelativeTimeFormat`) from the core's time.
20. **The bar and the sidebar badge** count what goes; with nothing ticked they count the categories to check.

## Conclusions

See [`readme.md`](./readme.md), "Conclusions": what was built, the numbers, what changed from this design and
the calls taken while building.
