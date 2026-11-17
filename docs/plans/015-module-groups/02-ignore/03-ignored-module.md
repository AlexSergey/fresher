# 015/02/03 — An ignored module in the sidebar and on its page

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** app/ui

## Files

- `app/ui/src/screens/use-ignored.ts` (new) — reads the two lists from the settings view the store holds and answers
  `isModuleIgnored(module)` (its own entry or its group's) and `isGroupIgnored(group)`; pure parts unit-tested.
- `app/ui/src/screens/frame-sidebar.tsx` — an ignored module's item dimmed with "Ignored" instead of its size; an
  ignored group's heading followed by "Ignored".
- `app/ui/src/screens/module/module-screen.tsx` — an ignored module's page shows the note "You ignore this module. Its
  findings are not shown or cleaned." with **Restore** (or "Restore the group" when its group is the reason); a
  visible module's page header gains **Ignore this module**, with the sentence of section 3 as its description.
- `app/ui/src/screens/overview/overview-screen.tsx` — "N ignored" sums the four counts; its link opens Settings on the
  first tab with an entry.
- `app/ui/src/state/navigation.ts` — the settings screen may be opened on a tab.
- `app/ui/src/locales/en.json`.

## Produces

The ignored module's look 02/04 checks.

## Checks

- Vitest, negatives first: a module that is not ignored shows its size and no note; an ignored module's page offers
  no selection. Positive: the sidebar's "Ignored", the page's note and Restore (module or group), the header's Ignore
  sends `{ kind: 'module', module }`, the overview's count and link.
- `npx vitest run src/screens/`.

## Conclusions

Built as written, with one change from the step: the core says which modules are hidden and by what
(`Overview.hiddenModules`), so the UI has no logic over the ignore lists (agent's calls below). The sidebar shows an
ignored module dimmed, under "<Group> · Ignored" when its group is ignored; the module page has the header with
"Ignore this module" and, when hidden, a note with Restore (or "Restore the group"), with no categories, left-outs or
selection. `Screen.settings.tab` lets the overview's "N ignored" link open the right tab. A review found the module
name twice on the page; the page header is now the only title. vitest 995 passed; Playwright passed after six
module-page screenshots were regenerated and looked at.
