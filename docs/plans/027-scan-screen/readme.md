# 027 — The scan screen

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** app/ui, app/tauri, core, sandbox,
e2e, docs

Drafted on 2026-11-20 as one file from the user's feedback of the first session of runs by hand
([`../../manual-testing/2026-11-20-feedback.md`](../../manual-testing/2026-11-20-feedback.md), items 1 to 18), and
approved by the user the same day with its eight agent's calls, which became decisions. The detailed design and
the steps come next.

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
