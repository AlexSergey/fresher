# 003/06/03 — The finding list

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `product/finding-row/` — `FindingRow`: a 52 px `ListRow`: `Checkbox`, the title (`tText(finding.title)`) over the
  subject (a `PathText` for a path; the tool and id for an external subject), the `DangerBadge` (small), the size
  (`SizeText`); the reason (`tText(finding.reason)`) in the row's tooltip and accessible description; selected rows
  tinted, a selected High row with the level's edge and glow. A loading variant (`finding` undefined) shows skeleton
  lines of the same height.
- `product/finding-list/` — `FindingList`: props `total`, `rowAt`, `onRangeChange` (from `useFindings`), `isSelected`,
  `onToggle`, `label`; a `VirtualList` of `FindingRow`s (row height 52), largest first as the source gives them.
  States: loading (the first page: 8 skeleton rows), the list, empty (`EmptyState`: "Nothing found here"), failed
  (`ErrorState` with the core's text and "Try again"). Rows of a newly loaded page fade in; rows that change place
  after a refresh glide (positions from the indices). A header row (not virtualised) names the columns for sighted
  users; the rows' accessible text reads title, level, size.
- `locales/en.json` — `ui.finding-list.*` (empty, column names, the row's accessible text pattern).

## Produces

The lists of the module page (07/03) and the review screen (07/04).

## Checks

- Negative: `a row whose page is loading is a skeleton, not an empty row`, `an empty list shows the empty state, not an
  empty frame`, `a refused page shows the core's text`, `100,000 findings render no more than about 60 rows`, `a
  finding's level is never colour alone`.
- Positive: `rows read title, level and size`, `Space toggles the focused row`, `the range sent for paging follows the
  scroll`, `a refreshed page moves a row to its new index with a glide`, `an external subject shows its tool and id`.
- Stories: finding row per level, long path, external subject, selected, selected High, focus, loading; list loading,
  a page of the demo findings, 100,000 generated findings scrolled to the middle, empty, failed; screenshots looked at.

## Conclusions

Done on 2026-11-16. `FindingRow` (`product/finding-row/`, with `texts.ts`: a level's word and a size as `SizeText`
takes it) and `FindingList` (`product/finding-list/`), each with tests (negative first), stories and WebKit gallery
screenshots, looked at. No new dependency, token or icon; 13 `ui.finding-list.*` keys.

Decided while building (agent's calls, to check):

- **One tab stop per row.** The row is the virtual list's roving tab stop and Space on it toggles the finding; its
  checkbox takes `tabIndex={-1}` (a new optional prop of `Checkbox`, only `-1`), so Tab never stops inside a row, and
  a click on the box still toggles it.
- **The row's accessible name and description.** `ListRow` passes on `aria-label` and `aria-describedby` (two
  optional props). The row's name is `ui.finding-list.row` ("{title}, {level}, {size}"), its description the reason,
  from a `hidden` element inside it. The reason's tooltip sits on the level's badge (pointer only): a `Tooltip` around
  the row would wrap the element the virtual list focuses and toggles.
- **The 52 px height** comes from `findingRowsClass` on the list's frame: it sets `--size-row` to the existing
  `--size-finding-row` and the widths of the level and size columns, which the header shares.
- **Props of the list**: `total` is `undefined` until the first page answers (8 skeleton rows, which also ask for the
  first page), `0` is the empty state. `failure` has the shape of `ipc/`'s `Failure` (product/ may not import it): a
  refusal shows the core's text, an unreachable core `ui.core.unreachable`. "Try again" calls `onRangeChange` with the
  last range, which makes the source ask for the pages again. `onToggle` and `isSelected` take the `Finding` (the
  store's `toggle-finding` needs its category, sizes and level). `useFindings` gives `total: 0` before the first page:
  the review and module screens (07) pass `undefined` until it has answered.
- **Sizes** are the bytes on disk (`allocated`); a partial size reads "at least …", an unknown one "Size not known".
- **Selected rows** keep the accent edge; only a selected High row lights the edge in High's colour (`edgeLevel`).
- A refusal replaces the rows with the error state, even when the cached rows could still be shown.
- Test helpers: `src/test/finding-fixtures.ts` (a finding of each level, a long path, an external subject, a partial
  size, the level words and sentences as the core has them) and `src/test/viewport.ts` (`fakeViewport`).
