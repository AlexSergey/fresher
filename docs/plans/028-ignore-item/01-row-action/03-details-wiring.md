# 028/01/03 — The details wire it

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** app/ui, e2e

A category's details give every finding with a path the item "Ignore this item"; the row leaves, the focus goes to
its neighbour or to "Back to Scan", and Settings restores it. The e2e mock learns paths. The design:
[`../00-design.md`](../00-design.md), sections 3, 4 and 5 (agent's calls 5 to 10).

## Files

- `app/ui/src/screens/category/category-findings.tsx` — `menuOf` built once per render with `useIgnoreCommands()`:
  `finding.subject.kind === 'path'` gives one `MenuItem` (`key: 'ignore'`, `label: t('ui.finding-list.ignore')`,
  `description: t('ui.finding-list.ignore.description')`, `onSelect: () => ignore({ finding: finding.id, kind:
  'finding' })`), any other subject none (an empty list, so the row has no menu).
- `app/ui/src/screens/category/category-screen.tsx` — the Back button gets a `ref`; a layout effect gives it the
  focus when the findings' count turns from more than 0 to 0 while the focus is on the page (`document.activeElement`
  is `null` or `document.body`).
- `app/ui/src/locales/en.json` — `ui.finding-list.ignore` "Ignore this item", `ui.finding-list.ignore.description`
  "Hides it, and everything inside it, from the results and never removes it. You can restore it in Settings."
- `app/ui/src/screens/category/category-screen.test.tsx` — the cases below.
- `app/ui/e2e/support/ignore-mock.ts` — `paths` in its lists (and its session storage); `ignore` with
  `{ kind: 'finding' }` records the finding's path, `restore_ignored` with `{ kind: 'path' }` drops it, each sends
  `setting-changed` for `core.ignore.paths`; `get_settings` answers `core.ignore.paths`; the overview's
  `ignored.paths` counts them.
- `app/ui/e2e/core-mock.ts` — `get_findings`, with `ignoring` on, leaves out a finding whose path is ignored or inside
  one (its total too); the mock's comment says the category totals are not recomputed.
- `app/ui/e2e/core-mock.spec.ts` — negatives first: `a finding outside the ignored paths stays`; then `an ignored
  path leaves get_findings with what is inside it, and a restore brings it back`.
- `app/ui/e2e/flows.spec.ts` — the cases below.
- `app/ui/e2e/screenshots/visual.spec.ts/category-details*` and `zoom.spec.ts/category-details-{125,150}-webkit.png`
  — regenerated (the menu column); Review's and every other baseline unchanged.

## Produces

The Ignore of one item, end to end in the UI — for 02.

## Checks

- `category-screen.test.tsx`, negatives first:
  - `describe('negative cases')`: `it('a tool's object has no row menu')`; `it('a refused ignore keeps the row and
    shows the failure')`; `it('the focus stays where it is when the list empties without the focus in it')`.
  - `describe('positive cases')`: `it('Ignore this item asks the core to ignore that finding')`; `it('the menu item
    carries its sentence')`; `it('when the last row leaves, the focus goes to Back to Scan')`.
- `npm run test --workspace app/ui` and `npm run test:cov --workspace app/ui` (at or above 90 %); `npm run lint:ts`;
  `npm run knip`; `npx eslint .` in `app/ui`; `npx prettier --check` on the changed files.
- E2E, `flows.spec.ts`:
  - `describe('negative cases')`: `the row's menu closes on Escape and ignores nothing`; `Review's rows have no
    menu`.
  - `describe('positive cases')`: `Ignore this item takes the row out of the details and the focus to the next row`;
    `Settings lists the ignored item by its path and Restore brings its row back`; `ignoring one item by keyboard
    alone: Tab to the row's menu, ArrowDown, Enter`.
- `npm run test:e2e:update --workspace app/ui -- visual.spec.ts zoom.spec.ts` (only `category-details*` change; look
  at each PNG), then `npm run test:e2e --workspace app/ui`.

## Conclusions

Built in f929530d and 6e75ed1a: the details wire "Ignore this item" to `ipc.ignore`, the focus goes to the next row or
"Back to Scan", the e2e mock learned paths (nested ones too). The fix round removed a focus jump on scan start and
added a focus assertion. The last-row focus is unit-tested only.
