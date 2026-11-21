# 028 — Ignore one item: design

**Status:** 🔨 in progress (the user's delegation of 2026-11-20) · **Version:** 0.1.0 · **Date:** 2026-11-21 ·
**Modules:** app/ui, e2e, docs

Written on 2026-11-21 from the deferred item and a read of the code on `feat/028-ignore-item` (plan 027's head,
951a8e79). Where the deferred note and the code disagree, the code is the truth and the choice is an agent's call.

## Global constraints

Plan 027's global constraints hold unchanged ([`../027-scan-screen/00-design.md`](../027-scan-screen/00-design.md),
"Global constraints", 1 to 14). In short: English only, no product named; no business logic in the UI; every core
call through `app/ui/src/ipc/index.ts`; the design layer knows nothing of the core; rule 7's plain sentence on every
choice; **no removal path changes**; every control reachable by keyboard and the focus never lost; negative tests
first; a changed component updates its story and its baselines, looked at before the commit; one commit per step,
never `--no-verify`. Rust builds, if any is needed, use
`CARGO_TARGET_DIR=/Users/serhiialeksandrov/Desktop/Work/Projects/my/github/fresher/target`.

## Intent

The deferred note `ignore-an-item-button`, removed by this plan (agreed 2026-11-16, plan 005,
agent's call 7): the Ignore button on one finding; the core does the rest already. The user's order of 2026-11-20
(translated): after plan 027, build "Ignore on a single item" autonomously, then plans 023 and 024; the user checks
everything together later.

## Goals

1. A finding row on a category's details offers "Ignore this item" with a sentence saying what it does.
2. It is reachable and usable by keyboard and by pointer, within the virtual list's keyboard model.
3. After the choice the row leaves at once, the focus goes to a sensible neighbour, the totals follow, and the item
   is listed in Settings → Ignored → Ignored items with Restore.

## Non-goals

- No core, engine, shell or cleaner change: `ignore`, `IgnoreTarget::Finding`, `core.ignore.paths`, the engine's
  hiding, the cleaner's protection and Restore exist and are tested (`crates/engine/src/ignore.rs`,
  `crates/e2e/tests/ignore.rs`).
- No Ignore on Review's rows, the cleanups' rows or a card's left-out lines.
- No narrower selection rule (agent's call 5), no "Undo" toast, no right-click menu, no keyboard shortcut.
- The always-tabbable controls of a kept row (the removability word and "Show in Finder") are not reworked.

## What already exists

- **Core**: `ignore(target: IgnoreTarget)`; `{ kind: 'finding', finding }` puts the finding's path in
  `core.ignore.paths`; a finding without a path is refused (`CannotIgnore`), a finding of another scan too
  (`UnknownFinding`). The finding and everything inside its path leave every answer at once; `setting-changed`
  follows. The cleaner gets the ignored paths as protection.
- **UI**: `ipc.ignore` and `useIgnoreCommands()` (`screens/settings/use-ignore-commands.ts`, a refusal reaches the
  store as `command-failed`); Settings → Ignored → Ignored items lists the paths with Restore
  (`screens/settings/ignored-items.tsx`); Scan's "N things you ignored" counts paths.
- **The menu**: `design/primitives/menu/menu.tsx` (plan 027), used by the card with icon `Settings`, label
  `ui.category-card.options`, item "Ignore this category" and its description.
- **The list**: `product/finding-list/finding-list.tsx` renders `FindingRow` in `design/data/virtual-list/`. The
  list is one tab stop (roving `tabIndex` on rows); arrows, Home, End and Page keys move it; Space toggles only when
  the row itself has the focus. Contrary to the deferred note, a kept row's removability word and its "Show in
  Finder" button are tab stops of their own (`tabIndex` 0 in every rendered row). The arrow keys move the rows from
  any element inside a row, including those controls.
- **What an ignore does to the UI state today** (`state/reduce.ts`, `setting-changed`): a `core.ignore.*` change
  clears the **whole** selection (the core drops its pending plan) and marks the overview stale. The finding pages
  (`state/findings.ts`) follow an overview change with a throttled refresh of the pages on screen only (500 ms); the
  other cached pages keep their old rows, so after a row leaves they would show shifted rows until refreshed. A
  cleanup's removal resets and refreshes the pages at once instead.
- **The e2e mock** (`e2e/support/ignore-mock.ts`) answers `ignore` and `restore_ignored` for categories, modules and
  groups only; a finding target and a path entry change nothing.

## Design

### 1. The list follows an ignore at once (plan 01, step 01)

`CoreState` gains `ignoreChanges: number` (0 at start), incremented by the `setting-changed` case when the key starts
with `core.ignore.` (the same test that clears the selection). `findings.follow` treats a change of it as it treats
`removed`: `reset()` then `refresh()`, so every cached page is dropped and the rows on screen are asked for again at
once. A restore in Settings goes through the same path, so a category's details show the row again on return.

### 2. The row's menu (plan 01, step 02)

- `Menu` gains `tabIndex?: -1 | 0` (default 0), passed to its trigger (`IconButton` gains the same optional prop).
- `FindingRow` gains `menu?: readonly MenuItem[] | undefined`. With items, the row ends with a `Menu` (icon
  `Settings`, label `ui.finding-list.options` "Options for {title}", `tabIndex` equal to the row's own), after the
  size column, in a column of its own that `findingRowsClass` sizes; without it the row is as today (Review). The
  trigger is visible on every row, not on hover only.
- `FindingList` gains `menuOf?: (finding: Finding) => readonly MenuItem[]`; its header gets the empty column when
  `menuOf` is given.
- `VirtualList`: the arrow, Home, End and Page keys move the rows only when the row itself has the focus (as Space
  already does); a key on a control inside a row is that control's. So ArrowDown on the row's menu button opens the
  menu instead of moving the list.

Keyboard model: Tab enters the list on its focused row; Tab again reaches that row's controls (its menu button, and a
kept row's word and "Show in Finder"), then leaves the list; Shift+Tab goes back to the row; the arrows move between
rows from the row. The menu then works as the card's: Enter, Space or ArrowDown opens it, Escape gives the focus back.

### 3. The details wire it (plan 01, step 03)

`CategoryFindings` passes `menuOf`: for a finding whose `subject.kind` is `path` (the core refuses the others), one
item:

- label `ui.finding-list.ignore` "Ignore this item";
- description `ui.finding-list.ignore.description` "Hides it, and everything inside it, from the results and never
  removes it. You can restore it in Settings.";
- `onSelect`: `useIgnoreCommands().ignore({ finding: finding.id, kind: 'finding' })`.

What follows, with no new code beyond section 1 and the empty-list focus:

1. The menu closes and gives the focus to its trigger, then the core saves the path and sends `setting-changed`.
2. The selection is cleared (today's rule) and the overview is asked for again; the finding pages reset and refresh
   (section 1). The row leaves; the virtual list's existing rule gives the focus to the row now at its index (the
   next one, or the previous one when it was the last).
3. When the list empties, `CategoryScreen` gives the focus to its "Back to Scan" button if the focus fell to the page
   (the empty state or the left-out note replaces the list).
4. The item is in Settings → Ignored → Ignored items, by its path, with Restore; Scan's ignored line counts it.
5. A refusal (a read-only settings file, a scan that replaced the finding) shows as every command's failure; nothing
   leaves.

No removal path changes: ignoring hides; the cleaner already protects an ignored path.

### 4. Texts

New in `app/ui/src/locales/en.json`, beside the card's: `ui.finding-list.options` ("Options for {title}"),
`ui.finding-list.ignore` ("Ignore this item"), `ui.finding-list.ignore.description` (above).

### 5. The e2e mock

`e2e/support/ignore-mock.ts` keeps `paths` beside its lists: `ignore` with a finding target records the finding's
path (the mock knows its script's findings), `restore_ignored` with a path entry drops it, both send
`setting-changed` for `core.ignore.paths`, `get_settings` answers the list, and the overview's `ignored.paths` counts
it. `get_findings` leaves out a finding whose path is ignored or inside one. The overview's category totals are not
recomputed for an ignored item (a mock simplification, written in `docs/testing.md`).

## Testing

- Unit, negatives first: `reduce.test.ts` (a non-ignore setting leaves `ignoreChanges`; an ignore list bumps it),
  `findings.test.ts` (an overview change alone does not reset the pages; an ignore change resets and refreshes at
  once), `virtual-list.test.tsx` (an arrow on a control inside a row does not move the list; from the row it does),
  `menu.test.tsx` (`tabIndex` -1 keeps the trigger out of the tab order), `finding-row.test.tsx` (no menu without
  items; the menu button follows the row's tab stop), `finding-list.test.tsx`, `category-findings` through
  `category-screen.test.tsx` (no menu on a tool's object; choosing Ignore calls `ignore` with the finding; the empty
  list gives the focus to Back).
- E2E `flows.spec.ts` (WebKit and Chromium), negatives first: the row's menu closes on Escape and ignores nothing;
  Review's rows have no menu; then Ignore this item takes the row out and the focus to its neighbour, Settings lists
  the path and Restore brings it back; the whole path by keyboard alone. `keyboard.spec.ts` and `axe.spec.ts` cover
  the new stories.
- Baselines: the gallery's `FindingRow/*` and `FindingList/*` (with a menu), `visual.spec.ts` `category-details-*`,
  `zoom.spec.ts` `category-details-125` and `-150`; Review's baselines must not change.
- Perf (rule 4): `npm run test:perf --workspace app/ui`, the 100,000-row scroll and "a category's details after a
  scan" against plan 027's numbers.

## Docs to update (plan 02)

- `docs/deferred/ignore-an-item-button.md` deleted and its row removed from `docs/deferred/README.md`.
- `docs/ideas/ignore-list.md`: "Taken by plan 028" line; its two links to the deferred file point to this plan.
- `docs/architecture/README.md`: the category's details (the row's menu), the list's keyboard model, the finding
  pages' reset on an ignore change.
- `docs/testing.md`: the new e2e cases and the mock's paths.
- `docs/manual-testing.md`: section 1.1 gets the walk of ignoring one item and restoring it.
- Older plans that link the deferred file are history and stay as written (plan 027's precedent).

## Open questions

None.

## Decisions

The user's, of 2026-11-20 (translated): after plan 027, build "Ignore on a single item" autonomously, then 023, then
024; take the recommended decision on every question; the user checks everything together later. No version bump:
0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Each with what it costs if wrong.

1. **A per-row settings menu**, the card's `Menu` with its `Settings` icon, rather than a key on the row or a button
   shown on hover: one pattern for both places, room for later row actions ("Why this?"), visible to the pointer and
   to a screen reader. Cost: a column of 28 px in the details' rows.
2. **The menu button is a tab stop only in the focused row** (it takes the row's `tabIndex`): the list stays about one
   tab stop. Cost: the menu of another row is reached by moving to that row first.
3. **Keys on a control inside a row belong to the control** (the list moves only from the row): needed for ArrowDown
   to open the menu; it also stops the arrows moving rows from a kept row's word and "Show in Finder". Cost: from
   those controls, Shift+Tab first.
4. **An ignore change resets and refreshes the finding pages at once**, not on the throttled overview refresh: the
   row leaves without a 500 ms wait and no cached page keeps shifted rows. Cost: one page request per list on
   screen per ignore.
5. **The whole selection is still cleared on any ignore** (the existing rule; the core drops its pending plan, and
   the UI cannot tell which picked findings lie inside the ignored path without path logic of its own). The menu's
   sentence does not mention it, as the card's does not. Cost if wrong: the user loses other ticks on the same
   screen; narrowing it needs the core to say which findings left.
6. **A finding the app may not remove can be ignored too** (hiding is not removing, and it quiets a row the user
   cannot act on); **a tool's object has no menu**, since the core refuses it. Cost: none on safety.
7. **Only the category's details offer it**; Review lists what the user picked, and ignoring there would mix two
   jobs. Cost: an ignore from Review is a trip to the details.
8. **When the last row leaves, the focus goes to "Back to Scan"**, the screen's first control, instead of the page.
9. **The texts are UI keys** (`ui.finding-list.*`), as the card's menu texts are (`ui.category-card.*`). Cost: three
   keys move if they belong to the core.
10. **The e2e mock does not recompute category totals for an ignored item**; the specs assert the rows, Settings and
    Restore, and the core's own e2e (`crates/e2e/tests/ignore.rs`) proves the totals.

## Conclusions

The ten calls stand as written and are listed with what was built in [`readme.md`](./readme.md), "Conclusions". Two
details were added while building: the last-row focus is unit-tested only (call 10: the mock keeps the totals), and a row
with an empty menu keeps a placeholder of the menu column so the columns line up. The walk is pending.
