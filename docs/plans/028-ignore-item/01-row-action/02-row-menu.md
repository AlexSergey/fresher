# 028/01/02 — The row's menu

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** app/ui, e2e

A finding row can end with a settings menu whose button is a tab stop only in the list's focused row, and the virtual
list leaves the keys of a control inside a row to that control. Nothing in the app passes a menu yet. The design:
[`../00-design.md`](../00-design.md), section 2 (agent's calls 1, 2, 3).

## Files

- `app/ui/src/design/primitives/icon-button/icon-button.tsx` — optional `tabIndex?: -1 | 0`, passed to the `button`.
- `app/ui/src/design/primitives/menu/menu.tsx` — `MenuProps.tabIndex?: -1 | 0` (default `0`, comment: "-1 keeps the
  trigger out of the tab order: a list's row that is not its tab stop"), passed to the trigger.
- `app/ui/src/design/data/virtual-list/virtual-list.tsx` — `onKeyDown`: the moving keys act only when the event's
  target is the row itself (`rowOf(slot) === event.target`), as Space does; the comment of the component says a key
  on a control inside a row is the control's.
- `app/ui/src/product/finding-row/finding-row.tsx` — `FindingRowProps.menu?: readonly MenuItem[] | undefined`; with
  items, after the size column, `<span className={styles.menu}><Menu icon={Settings} items={menu} label={t(
  'ui.finding-list.options', { title })} tabIndex={row.tabIndex ?? -1} /></span>`; the loading row keeps the
  column's room when its list has menus (`withMenu` passed by the list); the component's comment names the menu.
- `app/ui/src/product/finding-row/finding-row.module.css` — the `.menu` column (the small icon button's width) and,
  under `.rows[data-menu]`, the header's matching gap.
- `app/ui/src/product/finding-list/finding-list.tsx` — `FindingListProps.menuOf?: (finding: Finding) => readonly
  MenuItem[]`; each row gets `menu={finding === undefined ? undefined : menuOf?.(finding)}`; the frame carries
  `data-menu` when `menuOf` is given, so the header and the loading rows keep the column.
- `app/ui/src/product/finding-list/finding-list.module.css` — the header's empty menu column under `[data-menu]`.
- `app/ui/src/locales/en.json` — `ui.finding-list.options` "Options for {title}".
- `app/ui/src/product/finding-row/finding-row.stories.tsx` — new `with-menu` and `with-menu-open` (the menu's
  `defaultOpen`, for the gallery) stories with one item "Ignore this item" and its description.
- `app/ui/src/product/finding-list/finding-list.stories.tsx` — new `page-with-menu`.
- Tests: `menu.test.tsx`, `virtual-list.test.tsx`, `finding-row.test.tsx`, `finding-list.test.tsx` (the cases
  below).
- `app/ui/e2e/screenshots/gallery.spec.ts/FindingRow/` and `FindingList/` — the new stories' baselines; the existing
  ones must not change.

## Produces

`FindingRow.menu`, `FindingList.menuOf`, `Menu.tabIndex` — for 01/03.

## Checks

- `menu.test.tsx`: `describe('negative cases')` gains `it('a trigger with tabIndex -1 is out of the tab order but
  still opens on a click')`.
- `virtual-list.test.tsx`: `describe('negative cases')` gains `it('an arrow on a control inside a row does not move
  the list')`; `describe('positive cases')` keeps `it('moves with the arrows from the row')` green.
- `finding-row.test.tsx`, negatives first: `it('has no menu without items')`, `it('the menu button of a row that is
  not the tab stop is out of the tab order')`; then `it('the menu button of the focused row is reached with Tab and
  named after the finding')`, `it('choosing an item calls it once')`.
- `finding-list.test.tsx`, negatives first: `it('rows have no menu without menuOf')`; then `it('every loaded row
  gets the items of menuOf')`.
- `npm run test --workspace app/ui -- src/design src/product`; `npm run lint:ts`; `npm run knip`; `npx eslint` and
  `npx prettier --check` on the changed files.
- `npm run test:e2e:update --workspace app/ui -- gallery.spec.ts` (only the new `FindingRow/with-menu*` and
  `FindingList/page-with-menu` baselines appear; look at each PNG), then `npm run test:e2e --workspace app/ui --
  gallery.spec.ts keyboard.spec.ts axe.spec.ts`.

## Conclusions

Built in 64e19464: the finding row's settings menu (the card's menu and icon), a tab stop only in the focused row, with
the virtual list's keyboard model giving a control's keys to the control. Review: approved. After the menu closes the
focus is on its button (by design call 3).
