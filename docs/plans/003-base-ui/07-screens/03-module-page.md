# 003/07/03 — The module page

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `app/ui/src/screens/module/module-screen.tsx` — the generic page, rendered from the module's `ModuleInfo` alone, so
  a new module needs no UI code (unless it registers a custom view in `modules/registry.ts`):
  - `Toolbar` with the module's name and Scan / Stop;
  - `ModuleSection` with the module's found size and outcome;
  - one section per category of the descriptor that has findings, in size order: a `TreeRow` heading (name, level
    badge, size, count, the category's checkbox; expanded by default for the largest, collapsed for the rest) and,
    when expanded, a `FindingList` of that category (`useFindings({ module, category, order: 'largest' })`);
  - the category's four plain sentences (what it is, what made it, what happens if removed, whether it comes back) in a
    `<details>` under its heading — detail on demand, as rule 7 asks;
  - the `SelectionBar` at the bottom (`summarize`), **Review** goes to the review screen;
  - a screen opened at a category (from the overview) scrolls to it and expands it.
- `locales/en.json` — `ui.screen.module.*` (the four sentence labels, "no findings in this module").

## Produces

The page every module gets for free.

## Checks

- Negative: `a category without findings has no section`, `a collapsed category requests no page`, `nothing is
  selected when the page opens`, `a module that failed shows its reason and the findings it sent before`.
- Positive: `sections follow size order`, `expanding a category loads its first page`, `ticking rows updates the
  selection bar`, `ticking the category selects it whole`, `opened at a category, it is expanded and in view`.
- `npx vitest run src/screens/module` passes.

## Conclusions

Done on 2026-11-16. `ModuleScreen` (`screens/module/module-screen.tsx`) and one category's section
(`category-section.tsx`), with 14 tests (negative first); WebKit and Chromium screenshots of the page were taken on a
temporary port and looked at (not committed; 07/06 photographs every screen). No new dependency, token or icon; 14
`ui.screen.module.*` keys.

Decided while building (agent's calls, to check):

- **`useFindings` gives `total: undefined`** until the view's first page has answered (it gave 0), so a screen passes
  it to `FindingList` as it is; its test reads `unknown` instead of `0` before the answer. The only change outside the
  step's folder.
- **The heading**: the toolbar's title is the page's `h1` (the module's name); `ModuleSection` keeps its `h2` with the
  found size and the status.
- **Before any scan** the page shows a "Not scanned yet" empty state and no module section (there is no size to show);
  a scan that found nothing in the module shows the section with "No findings in this module".
- **The section's status** is the module's outcome in this scan; while scanning and not finished, "Scanning…"; with no
  outcome known (an overview of a scan from before the app started), "Done".
- **A category** is a `section` named by the category; its `TreeRow` sits in a one-row `role="list"` (a `ListRow` is a
  list item). The box is ticked when the category is selected whole, mixed when only some of its findings are (or it
  has exclusions); ticking it selects or drops it whole. Its level is the descriptor's.
- **The four sentences** (`<category>.what`, `.made-by`, `.if-removed`, `.comes-back`) sit in a `<details>` ("About
  this category") under the row, collapsed.
- **An expanded list** is a frame of up to 8 rows tall (`--rows`, from the overview's count), so the page does not
  jump while its pages load; it scrolls inside beyond 8. A collapsed category mounts no list, so it asks for no page.
- **Expanded by default**: the largest category and the one the page was opened at; the user's own choice per
  category wins after that. Opened again at another category (the same module's screen stays mounted), that one is
  expanded and scrolled to. The scroll is instant (`scrollIntoView({ block: 'start' })`, `scroll-margin-top` the
  toolbar's height), once per category it was opened at.
- **The selection bar** sums the whole selection (`summarize` over the overview's totals), as the review screen will
  show all of it; **Clear** empties it, **Review** goes to the review screen.
