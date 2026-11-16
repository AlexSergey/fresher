# 003/03/03 — Rows and the virtual list

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `design/data/list-row/` — `ListRow`: a 44 px row with slots `leading` (a checkbox or icon), `content` (title and
  subtitle), `trailing` (badge, size); states hover (background to `--surface-hover`, `--duration-fast`), selected
  (Iris tint and a 2 px leading edge, or the level's edge when `edgeLevel` is given), focus (the focus ring inside the
  row), disabled. `role="listitem"`, `aria-selected` when selectable.
- `design/data/tree-row/` — `TreeRow`: a `ListRow` with a disclosure (`ChevronRight` rotated by `transform` when
  expanded, `--duration-fast`), `level` for the indent, `aria-expanded`; children are rendered by the caller.
- `design/data/virtual-list/` — `VirtualList<T>`:
  - props: `count`, `rowHeight` (fixed), `getKey(index)`, `renderRow(index, item | undefined)` (undefined while a page
    loads: the caller renders a skeleton row), `onRangeChange(first, last)` for paging, `label`;
  - renders only the rows in view plus an overscan of 6 above and below, each absolutely placed by
    `transform: translateY(index × rowHeight)`, inside a spacer of `count × rowHeight`; scroll handled in one
    `requestAnimationFrame` per frame, the viewport measured by `ResizeObserver`;
  - semantics: `role="list"`, rows `aria-setsize={count}` and `aria-posinset={index + 1}`;
  - keyboard: a roving focus — Up and Down, Home and End, Page Up and Page Down move the focused row and scroll it into
    view; Space calls `onToggle(index)`; the focused row survives a re-render by its key;
  - moves: when a key's index changes, the row glides from its old position (`useFlipList` with positions computed from
    the indices, no measuring); rows new in view enter with the stagger (fade and `--distance-1`), at most 8 at once.

## Produces

`ListRow`, `TreeRow` and `VirtualList` for the finding list (06/03), the review screen and the confirmation's list.

## Checks

- Negative: `renders no row for a count of 0`, `renders no more than the visible rows and the overscan for 100,000`,
  `does not move focus past the last row`, `a disabled row does not toggle`, `a row whose page is not loaded gets
  undefined`.
- Positive: `scrolling shows the rows at that offset`, `reports the visible range for paging`, `rows carry setsize and
  posinset`, `arrow keys move the focus and scroll it into view`, `Space toggles the focused row`, `a reordered row
  plays a translation from its old index`, `motion off moves rows without animation`.
- Stories: list row rest, hover, selected, selected with a High edge, focus, disabled; tree row collapsed, expanded,
  two levels; virtual list 0, 10 and 100,000 rows, scrolled to the middle; screenshots looked at.

## Conclusions

Done on 2026-11-16. `ListRow`, `TreeRow` and `VirtualList` in `design/data/`, with `virtual-list/range.ts` (which rows
are in view, where a key moves the focus, how far to scroll). 34 unit tests (every check above, plus Space on a control
inside a row, keys on an empty list, a scroll pending at unmount, a focused key that left the view, the entrance of
new rows); 13 gallery stories photographed in WebKit and looked at. No icon was added (`ChevronRight` was there).

Decided while building:

- **No `aria-selected` or `aria-expanded` on the row.** ARIA does not allow them on `listitem` (the lint refuses them,
  and VoiceOver would ignore them). A selectable row's state is read from its checkbox (the leading slot), the
  disclosure of a `TreeRow` is a real `<button aria-expanded>` named by `toggleLabel`; the row carries
  `data-selected` and `data-disabled` for its look. `role="listitem"`, `aria-setsize` and `aria-posinset` stay.
- **`renderRow(index, item, row)`**: a third argument, the props the row's root spreads (`aria-setsize`,
  `aria-posinset`, `tabIndex` of the one tab stop); `ListRow` takes them. The item comes from `getItem(index)`.
- **A disabled row** is marked `data-disabled` by `ListRow`; the list does not call `onToggle` for it. Space toggles
  only when the row itself has the focus: a control inside a row handles its own Space.
- **`onRangeChange` reports the rendered range** (the rows in view and the overscan), so the overscan's pages load too.
- **The selected tint** is the accent glow (Iris at 35 %) at 40 % opacity over the row, 14 % Iris like the levels'
  tints: no new token. The focus ring is inset (`inset 0 0 0 2px` accent), so the viewport never clips it.
- **Each row sits in two layers**: a slot placed by `translateY(index × rowHeight)` and a motion layer inside it, which
  glides (`useFlipList` with positions from the indices) without fighting the slot's transform, and enters.
- **Entering rows**: only rows new in view in an update that did not scroll (new data, the first measured render)
  enter, with `--i` up to `--stagger-steps`: the stagger is capped at 8 steps; rows past the eighth start with the
  eighth. Rows scrolled into view, and every row under motion `off`, do not animate. The entrance is a CSS keyframe,
  so reduced motion fades only.
- **Size text in the stories is plain text**: `SizeText` (03/01) is built in parallel; the trailing slot takes any node.
