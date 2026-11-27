# 044/03/01 — The mockup

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** docs

The Largest screen with its three tabs, drawn on the design canvas before anything is built, and approved by the user.
The design: [`../00-design.md`](../00-design.md), section 8.

## Read first

[`../../../mockups/README.md`](../../../mockups/README.md) and the 0.1.0 mockups (`scan.dc.html` for the ring, the
separator and the tiles; `category-details.dc.html` for rows); the Largest screen as it is (`npm run dev`, or
`visual.spec.ts`'s `largest.png` and `largest-folders.png`); the user's taste: a coloured chart, a separator and tiles,
never a bare list, headings that say what to do.

## Files

- `docs/mockups/0.2.0/largest.dc.html` (new) — three frames, light and dark:
  1. **Files** as today (the filters, the rows with their ticks, the selection bar), only re-aligned with the second.
  2. **Folders**: the heading "Open these in the Finder and decide what to keep", the rule line, the bar of the home
     (five coloured segments and the rest, the home's free part neutral) with its caption, the separator, the three
     tiles (Folders, Space, Projects), then five rows (a project, a plain folder, one of years, one in a standard
     folder, one of the smallest size) each with its "Not changed for …" line and a "Show in Finder" button; "Show
     more"; and, small beside it, the empty state and the loading state.
  3. **Browse**: today's Folders view (the breadcrumb, the rows largest first, Open on a folder that opens) with no
     checkbox and no empty checkbox place, Show in Finder on every row, the note of a row that does not open, and the
     heading "Go into a folder, or show it in the Finder".
  The texts are the design's section 8, word for word; the data made up (names, paths, sizes).
- `docs/mockups/README.md` — a section "0.2.0" with the file's row, "approved on <date> for plan 044".

## Interfaces produced

- The approved layout 03/03 and 03/04 build from; what the mockup changes of the design's section 8 is written back
  into it in this step (the texts, the order of the parts).

## Tests

None (a drawing).

## Checks

- The user's approval, recorded (translated) in `docs/mockups/README.md` and in the global plan's "Decisions". 03/03 and 03/04
  do not start before it.
- `npm run check:readme`, `npx prettier --check docs/mockups/README.md`

Commit: `docs(docs): the mockup of Largest's three tabs`.
