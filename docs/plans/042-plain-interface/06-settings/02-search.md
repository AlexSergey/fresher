# 042/06/02 — The search

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

A search over every setting and section; its results replace the page while it holds text; the tree dims what does not
match and counts what does; Enter or a click opens the setting's page with the setting focused; Esc clears. The
design: [`../00-design.md`](../00-design.md), section 10; agent's call 17. Mockup:
[`settings-search.dc.html`](../../../mockups/0.1.0/settings-search.dc.html).

## Read first

06/01's commit, `design/primitives/search-field/`, `product/settings-row/texts.ts` (`settingText`, `limitsText`),
`app/ui/src/i18n/` (`tText`), the dictionary in the store.

## Files

- `screens/settings/settings-search.ts` (new, pure) — `searchSettings(query, entries)`: entries are every setting
  shown in Settings (key, its section, its group when a module's, its label and description from the dictionary) and
  every section (its name); a match is a case-insensitive substring of the label, the description or the section's
  name, trimmed; answers the hits in the tree's order, each with the matched ranges; an empty query answers none.
- `screens/settings/settings-search.tsx` (new) — in the sidebar's top slot: `SearchField` labelled
  `ui.screen.settings.search` "Search settings" (placeholder the same; border `--accent-fill-top` while it holds
  text), margin-bottom 14 px. While it holds text:
  - the page's place shows the results: `h1` `ui.screen.settings.matches` "{count} settings match “{query}”" (plural;
    `ui.screen.settings.no-match` "Nothing matches “{query}”" for none), `ui.screen.settings.matches-hint` "Pick one to
    go to it. Esc clears the search." (13 px, `--text-secondary`); one surface of hits, dividers 1 px; each hit a button
    (padding 12 px 14 px, column gap 3 px, hover `--surface-hover`): its place "Group › Module" or the section's name
    (12 px, `--text-tertiary`; `ui.screen.settings.place` "{group} › {module}"), its label (14 px, 500), and its
    description (12 px, `--text-secondary`) led by its current value for a number ("90 days. "), the matched text
    wrapped in `mark` (background `--menu-item-active`, colour `--text-on-fill`, radius 3 px, padding 0 2 px).
  - the tree: a section or module without a match in `--text-disabled`; a group with matches opens; a module with
    matches shows their count after its name in `--accent-text`.
  - Enter opens the first hit; a click opens its hit: the search clears, the page opens and the setting's control
    takes the focus (`focus` of the screen; the row scrolls into view). Esc in the field clears it.
- `app/ui/src/locales/en.json` — the texts above.
- `e2e/` — `flows.spec.ts`: negative "a search with no match says so and leaves the settings unchanged"; positive "a
  search lists every setting whose label or description matches, the match marked"; "a click opens the page with the
  setting focused"; "Esc clears the search"; `visual.spec.ts` the search's results (the mockup's "days"); `keyboard`,
  `axe`.

## Tests (negatives first)

`settings-search.test.ts`: negative "an empty or blank query matches nothing"; "a setting hidden from Settings is not
searched"; positive "matches a label, a description and a section's name, any case"; "hits come in the tree's order";
"ranges mark every match". `settings-search.test.tsx`: negative "Esc clears and shows the page again"; positive "the
tree counts matches and dims the rest"; "Enter opens the first hit with its setting focused".

## Checks

`npx vitest run src/screens/settings`; test:cov:ui; lint:ts, lint:code, knip; `npx playwright test e2e/flows.spec.ts
e2e/visual.spec.ts e2e/keyboard.spec.ts e2e/axe.spec.ts --project webkit --project chromium` (look at the baseline
against the mockup).

Commit: `feat(ui): the settings search`.

## Conclusions

- `settings-search.ts` and `settings-search-view.tsx`: labels, descriptions with their current value, and section
  names; marks, tree counts and dimming; Enter in the field or a click opens the page with the setting focused
  (`Screen.settings.focus`); Esc clears. Dimmed tree items use `--text-tertiary` to pass contrast (no axe skip).
