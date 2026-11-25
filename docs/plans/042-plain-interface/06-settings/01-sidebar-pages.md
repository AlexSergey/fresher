# 042/06/01 — The sidebar and the pages

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

Settings splits into pages, picked from a second sidebar: General, Scan, Ignored, Permissions, About, then "What it
looks for", the module groups as a tree, each module a page. The design: [`../00-design.md`](../00-design.md),
section 10; agent's call 17. Mockup: [`settings.dc.html`](../../../mockups/0.1.0/settings.dc.html).

## Read first

`screens/settings/` (all: `settings-screen.tsx`, `settings-groups.ts`, `ignored-*`, `own-data-group.tsx`,
`file-notice.tsx`, `rescan-notice.tsx`), `screens/permission/permissions-group.tsx`, `product/{module-groups.ts,
settings-row/}`, `state/navigation.ts` (`{ kind: 'settings'; tab? }`), `screens/scan/scan-screen.tsx` ("Show in
Settings"), `e2e/visual.spec.ts` (settings), `e2e/flows.spec.ts` (settings flows).

## Files

- `state/navigation.ts` — `SettingsPage = { kind: 'general' | 'scan' | 'ignored' | 'permissions' | 'about' } | { kind:
  'module'; module: ModuleId }`; the screen `{ kind: 'settings'; page?: SettingsPage; focus?: SettingKey; tab?:
  IgnoredTab }` (`tab` opens Ignored on that tab; no page: General). Scan's "Show in Settings" opens Ignored on its tab
  as now.
- `screens/settings/settings-sidebar.tsx` (new) — a `nav` labelled `ui.screen.settings.sections` "Settings sections",
  220 px, border-right 1 px `--border-subtle` (= `--gray-800`), padding 24 px 12 px, column gap 2 px; a slot for the
  search (06/02; here an empty space of its height so the layout does not jump, or the field without behaviour);
  items `ui.screen.settings.group.{general,scan,ignored,permissions,about}` (13 px, `--text-secondary`, padding 5 px
  10 px, radius 6 px; hover `--surface-hover` and `--text-primary`; the current page `--surface-hover` +
  `--text-primary`, `aria-current="page"`); the label `ui.screen.settings.looks-for` "What it looks for" (11 px,
  uppercase, letter-spacing 0.6 px, `--text-tertiary`, padding 16 px 10 px 6 px); each module group with a module that
  has settings: a button `core.group.<group>.name` with a chevron (down when open, right when closed; `aria-expanded`),
  open when it holds the current page, toggled by a click; its modules (padding-left 26 px) by `<module>.enabled.label`.
- `screens/settings/settings-page.tsx` (new) — the page of the current section, main padding 28 px 40 px, gap 20 px:
  - a core page: `h1` (22 px, 600) of the section's name, then its part: General — the core's general rows, then
    Fresher's own data (`OwnDataGroup`); Scan — the scan rows; Ignored — `IgnoredGroup` (its tabs); Permissions —
    `PermissionsGroup`; About — `AboutGroup`.
  - a module's page: the group's name (12 px, `--text-tertiary`), the `h1` module name, its description (13 px,
    `--text-secondary`, `<module>.enabled.description`); its rows in one surface (background `--surface-raised`, border
    `--border-subtle`, radius 10 px), 1 px dividers inset 16 px, each row padding 16 px (label 14 px 500, description
    12 px `--text-tertiary`); the module's on/off switch (`<module>.enabled`) last.
  - the file notice and "Rescan to apply" on every page, at its top.
- `screens/settings/settings-screen.tsx` — the sidebar and the page side by side; the `Toolbar` keeps the screen's
  title; `settings-groups.ts` gives the page's rows (a module's in the module's order but `enabled` last).
- `app/ui/src/locales/en.json` — `ui.screen.settings.sections`, `ui.screen.settings.looks-for`; the group headings kept.
- `e2e/` — `visual.spec.ts` (settings of every kind becomes one baseline per page kind: General, a module's page, the
  Ignored page per tab, About), `flows.spec.ts` (changing a setting, the Ignored flows, reset), `keyboard.spec.ts`
  (the tree by keyboard), `axe`.

## Tests (negatives first)

`settings-screen.test.tsx`: negative "a module group with no settings is not in the tree"; "a closed group hides its
modules"; "the screen opens on General without a page"; positive "a click on a module opens its page with its switch
last"; "Ignored opens on the tab asked for"; "the current page is marked current"; "Fresher's own data is on General".
`navigation.test.ts`: positive "a settings screen keeps its page and focus".

## Checks

`npx vitest run src/screens/settings src/state`; test:cov:ui; lint:ts, lint:code, knip; `npx playwright test
e2e/visual.spec.ts e2e/flows.spec.ts e2e/keyboard.spec.ts e2e/axe.spec.ts --project webkit --project chromium` (look at
each settings baseline against the mockup).

Commit: `feat(ui): Settings with its own sidebar and pages`.

## Conclusions

- The Settings sidebar and pages: General (with Fresher's own data), Scan, Ignored, Permissions, About, the module
  tree, a module's page with its switch last ("Look for {module}", ruling C-7). The page's h1 names it; the sidebar's
  border runs full height (fix round, after deleting the stale baselines); a group holding the current page stays
  open.
