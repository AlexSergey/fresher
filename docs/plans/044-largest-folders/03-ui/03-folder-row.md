# 044/03/03 — `FolderRow`

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** app/ui

One folder left alone: what it is, where it is, how long it was left, and Show in Finder; nothing to tick. The
design: [`../00-design.md`](../00-design.md), section 8; the approved mockup (03/01).

## Read first

03/01's approved `docs/mockups/0.2.0/largest.dc.html` and 03/02's commit; `app/ui/src/product/own-item-row/`
(`ListRow`, `SizeText`, `honestSize`, its stories and test); `app/ui/src/design/{data/list-row,primitives/badge,
primitives/button}`; `app/ui/src/i18n/` (`t` with `count`, plurals as `.one`/`.other`).

## Files

- `app/ui/src/product/folder-row/folder-row.tsx` (new) — `export type FolderRowProps = Forced & Partial<VirtualRowProps>
  & { readonly onReveal: () => void; readonly row: FolderRow }` and `FolderRow`: a `ListRow` with the name, the line
  `{place}` and a "Project" `Badge` when `row.project`, the line from `row.leftFor` (`ui.folder-row.months` or
  `ui.folder-row.years`, `count` of kind `count`), the size (`SizeText` of `honestSize(row.node.sizes)`), and a
  secondary `Button` "Show in Finder" with the accessible name "Show in Finder: {name}". No checkbox, no empty checkbox
  place. A comment: the row is information; Fresher never removes it.
- `app/ui/src/product/folder-row/folder-row.module.css` (new) — the mockup's spacing, tokens only.
- `app/ui/src/product/folder-row/folder-row.stories.tsx` (new) — a plain folder, a project, one of years, a long
  path, in the gallery like `own-item-row.stories.tsx`.
- `app/ui/src/locales/en.json` — `ui.folder-row.months.one` "Not changed for {count} month. Open it in the Finder to
  decide.", `.months.other` "Not changed for {count} months. Open it in the Finder to decide.", `.years.one` "Not
  changed for {count} year. Open it in the Finder to decide.", `.years.other` "Not changed for {count} years. Open it
  in the Finder to decide.", `ui.folder-row.project` "Project", `ui.folder-row.reveal` "Show in Finder",
  `ui.folder-row.reveal-label` "Show in Finder: {name}".

## Interfaces produced

- `FolderRow` (`product/folder-row/folder-row`), for 03/04.

## Tests (negatives first)

`app/ui/src/product/folder-row/folder-row.test.tsx`:
- `describe('negative cases')`: `shows no checkbox`, `shows no project badge for a folder that is no project`.
- `describe('positive cases')`: `shows the name, the place and the size`, `says the months it was left alone`, `says
  the years it was left alone`, `shows the project badge`, `calls onReveal from Show in Finder, named after the
  folder`.

## Checks

- `npx vitest run src/product/folder-row` (from `app/ui`)
- `npx playwright test --project gallery e2e/gallery.spec.ts` (the stories render; new gallery baselines reviewed
  against the mockup before they are kept)
- `npm run lint:ts`, `npm run lint:code`, `npm run check:texts`

Commit: `feat(ui): the row of a folder left alone`.
