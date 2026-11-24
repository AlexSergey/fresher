# 042 — A plain interface

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-24 ·
**Modules:** core, engine, modules, testing, sandbox, e2e, app, app/tauri, app/ui, scripts, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved. Goes before 041 (the audit runs on the interface that ships).

## Intent

The user's task (translated, from the manual test of 2026-11-24,
[`../../manual-testing/2026-11-24-feedback.md`](../../manual-testing/2026-11-24-feedback.md), items 1-24): "The UI
looks bad and I do not understand what is happening on the screens: it must be simplified. A more laconic,
understandable interface, with short clear descriptions and a clear separation of sections. A balance between how much
data a screen shows and how beautiful it is. For the first release, hide High entirely. Settings is overloaded: a
sidebar with its structure and a global search."

The design was found on mockups (three rounds; the user approved the third):
[`docs/mockups/0.1.0/`](../../mockups/README.md). The screens are built from them.

## Goals

1. Scan answers "what can I remove, and how much" at a glance: a compact disk card with a coloured ring, a separator,
   then tiles grouped by what removing them costs.
2. Review shows what goes and what the user has checked, category by category, items apart.
3. Every text a person reads is short and plain: no "Swap and sleep image", no "Comes back by itself when needed".
4. Settings has its own sidebar (a tree of sections) and a search that jumps to a setting; numbers are sliders.
5. No High finding is offered in the first release, and the core enforces it.

## Non-goals

- New categories or modules; changing a category's level (only its texts).
- A light theme, translations, onboarding (014).
- Removing High from the code: the categories, their tests and the High consent of 039 stay, switched off.

## Design

### 1. High off for the first release (core)

- One switch in the engine, `HighLevel::{Offered, Withheld}`, set by the app at start (release: `Withheld`; tests set
  both). Withheld: High findings never leave the engine (not in the scan's report, the totals, the storage map's
  "can be freed"; their bytes stay in "Your files and apps"); `prepare_cleanup` refuses a High item (a new refusal,
  tested); the engine reports the switch so the UI hides what has no meaning without High.
- What disappears with it: app leftovers' data folders, big unopened downloads, Xcode archives, the screen Largest
  (every row is High) and the uninstall choice "Remove everything" (the app keeps "Keep my settings").
- The sandboxes and goldens of these categories run with `Offered`; one case file proves `Withheld` offers nothing.

### 2. Scan

- **Top bar**: the screen's name, when it was scanned, Scan again.
- **The disk card** (compact, the current card's gradient): a ring on the left, "3.8 GB can be freed" in its centre;
  on the right the disk's name, "109 GB free of 494 GB", and a two-column legend of five parts, each a distinct hue
  (no grey on grey): Can be freed (green, highlighted), Your files and apps, macOS, System space, Free. The core merges
  its storage parts into these five (System space = swap and sleep files + the disk's own space; Your files and apps =
  your folders + outside the scanned folders) and gives each a one-sentence "i". A part too small to see gets a
  minimum arc. Hover links a legend row and its arc both ways (027).
- **A separator**: "What you can remove", a line fading out.
- **Groups**, one per level, each a heading with its icon and one plain line:
  - Safe → **Safe to remove** — "Apps make these again on their own."
  - Low → **Usually safe** — "You may need to download them again."
  - Medium → **Look before removing** — "May hold something you still want."
- **Tiles**, two columns: an icon in a tinted square (the category's icon, from the core), name, one line, the size
  large, the item count, Details, the `⋯` menu, the checkbox; the border tinted by the group. The selection bar floats
  at the bottom as now. Left-out-only and described-only categories keep their own section below, restyled.

### 3. Review

- **The summary card**: "2.6 GB goes to the Trash · 6 items", "You can put them back from Cleanups", a bar split by
  level, a chip per level with its size; the Medium chip says how many items are not checked yet.
- **A separator**: "What goes".
- **One tile per category**: the checkbox for all, its icon, name, its group and one line, a pill "3 of 3 checked"
  (green when all, amber when not), the size; under it every item with its own checkbox, name and short path.
- **Look before removing** items are never ticked for the user: each must be ticked by hand. This replaces 039's two
  parts ("Safe to remove" / "Danger zone") and its Scan question for Medium "Select all"; the core's High consent
  stays, unused while High is withheld.
- The bar at the bottom: count, size, Clear, Move to the Trash.

### 4. Category details and menus

- Details: back to Scan, name, its group, size and count; three answer blocks side by side: What it is / If you remove
  it / Comes back?; the left-out line; the items with checkboxes, the app's name first and the path small under it.
- Every row and tile menu: the pointed item fills with the accent colour (as a macOS menu); Show in Finder, Ignore this
  item, Ignore the category, Stop looking for the module.

### 5. Settings

- A second sidebar inside the screen: a search field, then General, Scan, Ignored, Permissions, About, then "What it
  looks for": the module groups as a tree (open/close), each module a page.
- The search matches labels, descriptions and section names; results list "Group › Module", the setting's name and its
  description with the match marked; the tree dims sections without a match and counts matches; Enter or a click opens
  the page with the setting focused; Esc clears.
- A whole-number setting is a slider from its minimum to its maximum with a number field beside it (the core's
  bounds); refusal texts stay.

### 6. Wording

- Every text of the core, the modules and the UI is rewritten to the writing guide, shorter: a category's name 2-4
  words, its one line under 50 characters, each answer one sentence. The level names stay in the code; people read the
  group names above. Storage parts get plain names. The trial and demo modules' items get readable names.
- `writing.md` gains the group names, the five storage part names, a "one line" length and a word list for the
  replaced terms; `npm run check:texts` checks the new lengths.

### 7. The logo and its swoosh

- The logo is redrawn as a vector in the app's colours (variant A of 2026-11-24): the leaf in graphite, the swoosh in
  the iris gradient (`#4553dc` to `#9aa6ff`), the star in near-white, and a 1 px rim of the swoosh's gradient at 50 % opacity
  so the leaf does not melt into a dark background. It replaces `assets/logo.png`, the app icons in
  `app/tauri/icons/` (`.icns` included) and the social preview.
- The swoosh becomes the app's motif: a sheen on the primary buttons (text contrast kept) and the scan's progress, which fills a swoosh-shaped line in the disk card while the scan runs.
- The logo sits small beside "Fresher" at the top of the sidebar.

## Parts (planned when approved)

1. Core and engine: the High switch and its refusal; the five storage parts; a category icon key; the group texts.
2. UI design pieces: the disk card and ring, the separator, the tile, the menu hover, the slider.
3. Scan.
4. Review.
5. Category details.
6. Settings: sidebar, tree, search.
7. Wording pass (core, modules, UI) and `writing.md`.
8. The logo: the icons and the social preview (the vector source `docs/mockups/0.1.0/logo.svg` and the README's
   `assets/logo.png` were done on 2026-11-24).
9. Docs, Playwright baselines, demo recordings if a GIF changes, sandboxes, the walk.

## Open questions

1. When does High come back: a later release, or a setting "Show items that may hold the only copy" (off by default)?
2. Icons per category: one generic icon per group, or one per category (about 40 to draw)?

## Decisions

- 2026-11-24: direction A (a summary and a list) chosen; after two rounds the third mockup approved: the disk card
  with a ring, a separator, tiles grouped by level (Scan), a summary card and per-category tiles with items (Review);
  Settings with its own sidebar and search, sliders, the details screen, the menu hover (the user).
- 2026-11-24: High hidden for the first release, by a switch in the core, all five High places go, Largest included
  (the user).
- 2026-11-24: the sidebar of the app stays (the user: "neat and extensible").
- 2026-11-24: logo variant A (graphite leaf, iris swoosh); the swoosh on the buttons and as the scan's progress; the
  logo small in the sidebar beside the name; no decorative swoosh at the bottom of Scan (the user).
- 2026-11-24: approved mockups live in the repository, `docs/mockups/<version>/` (the user).

## Agent's calls (to check)

1. The switch is set at start by the app, not a user setting; the release build always withholds High.
2. Group names: "Safe to remove", "Usually safe", "Look before removing"; the words Safe/Low/Medium leave the screens.
3. The five storage parts are merged in the core, not the UI (no business logic in the UI).
4. "Look before removing" items are never pre-ticked in Review; 039's two Review parts and its Scan question go.
5. Largest leaves the sidebar while High is withheld; Apps stays with "Keep my settings" only.
6. The plan goes before the audit (041).
