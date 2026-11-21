# 027/04/01 — The docs

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** docs

Every doc that names the overview, a module page, the view by place, the map block or the rows by meaning says what
the UI is now; the core's place queries without a UI get their deferred item. The design:
[`../00-design.md`](../00-design.md), sections 1 to 9 (agent's call 15).

## Files

- `docs/plans/012-storage-map/readme.md` — a dated note under the goal: "Since plan 027 (2026-11-20) the home
  volume's map is the Scan screen's ring and legend; the separate map block of the home volume, the view by place and
  the rows of the view by meaning are gone from the UI. Other volumes keep their map block. The core's queries stay;
  see `../../deferred/place-queries-without-ui.md`." Its steps are not rewritten.
- `docs/plans/015-module-groups/readme.md` — a dated note: the module pages and the sidebar's module groups are gone
  since plan 027; a module is ignored from a card's settings menu and restored in Settings → Ignored; a category's
  details replace its section of the module page.
- `docs/architecture/README.md` —
  - the UI tree (`screens/` lists `scan`, `category`, `review`, `cleanups`, `settings`; the `modules/` line goes);
  - the store bullet: the by-meaning groups of the home volume are asked for the ring's split by level only; the
    place pages and their cache are gone;
  - the overview bullet rewritten for Scan: the hero's ring of the home volume (parts, the cleanable part by level,
    Free as the gap, the legend with its "i", ring and legend highlight each other), the zones with "Select all",
    the selection bar, other volumes' map blocks;
  - the navigation bullet: the `Screen` union of the design, section 1; `screenToShow` by category;
  - the design layer's menu primitive (`design/primitives/menu/`) beside the tooltip;
  - the line about the module page dropping a left-out-only category speaks of the category's details;
  - the storage map's section keeps the core's By place (`get_storage_place`) and says the UI no longer shows it.
- `docs/testing.md` — every place that names the overview, a module page or the view by place: "Scan", "a category's
  details"; the `[case.map]` paragraph says the autotest opens the case's folder with the core's place query (not
  "as the overview opens a folder row"); the e2e list names the new cases (ring ↔ legend, the card's menu, a zone's
  Select all, the details, the short sidebar).
- `docs/manual-testing.md` — the items that open a module page, a module's sidebar entry, By meaning or By place
  (sections around "Ignored groups", "the map's lines (both views …)", "Ignore the module … from the module's page",
  "On the module's page: Keep settings …", "the module's page shows the notice that Downloads is not allowed",
  "On every module's page select everything offered", "By place: opening the home …") rewritten for Scan's zones,
  a card's Details and its settings menu, and the ring's legend; nothing else in the file changes.
- `docs/deferred/place-queries-without-ui.md` (new) — in the folder's format: **Agreed:** 2026-11-20, by the user's
  approval of plan 027's draft (decision 6) · **Due at:** the next plan that shows the disk by folder or by app.
  What is postponed: `get_storage_place` and `get_storage_roots` (the view by place) have no UI since plan 027;
  `get_storage_meaning` is used only for the ring's split by level, its apps, places and the rest unused; the engine
  and the core keep them, tested. What doing it needs: decide whether a folder view returns (in a category's details,
  or its own screen) or the queries are removed with their tests, bindings and the e2e mock's answers; and, if it
  returns, the two former deferred items folded here — a way up from a folder opened from the by-meaning view (the
  way from the volume's root kept in the navigation, or the folder's ancestors asked of the core) and a root's
  explained size in the roots list (a core change).
- `docs/deferred/README.md` — the new row; the rows of `way-up-from-meaning.md` and `roots-explained-line.md` removed
  and both files deleted (`git rm`).
- `docs/categories/README.md`, `docs/goal.md`, `docs/architecture/modules.md` and `docs/architecture/permissions.md`
  — only where they name the overview, a module page or the view by place (find them with the grep below).

## Produces

Docs that match the code; the deferred row.

## Checks

- `grep -rnE "module page|module's page|By place|by place|By meaning|by meaning|[Oo]verview" docs --include='*.md' | grep -v "^docs/plans/\|^docs/manual-testing/"`
  shows only lines that are true now (the core's `Overview` type and `get_overview`, the core's by-place query, other
  volumes' map blocks).
- `npx prettier --check` on the changed docs; every link resolves (`npm run lint:code` covers Markdown if configured,
  else each relative link opened by hand).
- `npm run pre-commit` passes on the commit (the repository guards).

## Conclusions

Not written yet.
