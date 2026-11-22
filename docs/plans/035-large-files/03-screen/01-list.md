# 035/03/01 — The entry, the Files list and the row

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** app/ui

The sidebar entry **Largest**, the screen's frame and its states, the Files list paged by the core, and the row
component. No selection yet (the row's checkbox is wired in 03/02); no filter yet. The design:
[`../00-design.md`](../00-design.md), sections 7 and 8.

Read first: `docs/architecture/README.md`, "UI" (the folders and their import boundaries, "State", "Navigation"),
`app/ui/src/ipc/index.ts` (every core call goes through it), `app/ui/src/ipc/bindings.generated.ts` (`LargestQuery`,
`LargestPage`, `LargestRow`, `Offer`, `ItemKind`, from 02/01), `app/ui/src/state/navigation.ts`,
`app/ui/src/screens/{frame-sidebar.tsx,app-frame.tsx,screen-stack.tsx}`, `app/ui/src/product/finding-row/` (the row
the app already has: its layout, size and text rendering, and its stories), `app/ui/src/i18n/` (`t`, `tText`,
`format-size.ts`), `app/ui/src/screens/permission/coverage-notices.tsx`, `app/ui/src/gallery/registry.ts`.

## Files

- `app/ui/src/ipc/index.ts` — `getLargest(query: LargestQuery): Promise<LargestPage>`, `getLargestPlace(folder:
  NodeId | null, offset: number, limit: number): Promise<OwnPlace>`, `pickOwnItem(node: NodeId): Promise<Finding>`,
  in `COMMANDS` and as the file's other calls are written; `getStoragePlace` stays absent.
- `app/ui/src/state/navigation.ts` — `| { readonly kind: 'largest' }` in `Screen`; `screen-stack.tsx` and
  `app-frame.tsx` — the new arm (label `ui.screen.frame.nav.largest`).
- `app/ui/src/screens/frame-sidebar.tsx` — a `NavItem` **Largest** right after Scan, in Scan's `NavSection`, with an
  icon from `design/icons/icons.ts` (add one allowed `lucide-react` icon such as `ArrowDownWideNarrow` to the
  allowlist if none fits; one icon, named in the file's list).
- `app/ui/src/product/own-item-row/own-item-row.tsx` (new), `.module.css`, `.stories.tsx`, `.test.tsx` — a
  presentational row, props only: `{ row: LargestRow | OwnRow; checked: boolean; onToggle?: () => void; onOpen?:
  () => void }` (a discriminated input, or two small components sharing one layout: the simpler). It shows the name, the
  kind (`core.largest.kind.<kind>`), the place (Files) or the explained part (Folders, "explained on Scan: {size}" when
  not zero), the size (`format-size`), "Changed {when}" from `node.modified` (the app's existing relative date helper
  if there is one, otherwise the date as the cleanups list writes it; no clock read in the component: the time to
  compare with comes by props), the `unpacked` hint (`core.largest.unpacked` with `folder`), and, when withheld, the
  reason's sentence (`core.largest.withheld.<reason>`) and **no checkbox**; a checkbox (`design/primitives/checkbox`)
  only when offered; an "Open" button only when `onOpen` is given. Accessible names: the checkbox is labelled by the
  row's name and size. Stories for every state: a file offered, a package, an archive with its hint, withheld (one per
  look), a folder that opens, checked.
- `app/ui/src/gallery/registry.ts` — the row's stories registered.
- `app/ui/src/screens/largest/largest-screen.tsx` (new), `.module.css`, `.test.tsx` — the screen:
  - the heading "Largest files and folders" and, from the answer, "{count} items, {size}";
  - the last scan's `CoverageNotices` above the list;
  - the states, from the store and the answer: no scan ("Scan first to see your largest files." with the Scan
    button the scan screen uses), a scan running ("Scanning. The list comes back when the scan ends."), a refusal (the
    app's error toast already shows the core's text: the screen shows its empty state), an empty list ("Nothing here is
    {size} or larger." with `minSize`), `truncated` ("Only the 10,000 largest are listed."), and the out-of-date note
    when `lastScan.changedSince` is set ("A cleanup changed the disk since this scan. Scan again for exact sizes.");
  - the Files rows, 100 a page, "Show more" asking the next offset; the list asked again when a scan ends and when
    `findings-removed` arrives (the store's existing signals: follow how `state/findings.ts` restarts its pages).
- `app/ui/src/screens/largest/use-largest.ts` (new) — the hook that asks `getLargest` and keeps the pages; its test
  with a fake IPC as the other hooks' tests do.
- `app/ui/src/locales/en.json` — `ui.screen.frame.nav.largest` "Largest", `ui.screen.largest.*` (heading, summary,
  states, "Show more", "Open", "Changed {when}", the truncated and out-of-date notes). The chrome-keys test passes.
- `app/ui/src/test/largest-scan.ts` (new) — a `LargestPage` the core could answer, with its `Finding`s for 03/02 and the
  e2e mock: a disk image, a virtual machine package, a video, an archive with `unpacked`, a package read in part
  (withheld `read-in-part`); texts read from `crates/core/locales/en.json` as the other test scans read theirs. Only
  states the core can produce together.

## Checks

Vitest, `describe('negative cases')` before `describe('positive cases')`:

- `own-item-row.test.tsx`: negative — `a withheld row has no checkbox and says why`, `a row with no time says nothing of
  when`, `a folder that does not open has no Open button`; positive — `an offered row shows its name, kind, place and
  size`, `an archive names the folder beside it`, `the checkbox is labelled by the row`.
- `largest-screen.test.tsx`: negative — `before a scan it asks to scan first`, `while a scan runs it says the list
  comes back`, `an empty answer says the minimum size`; positive — `the rows come largest first as the core sent them`,
  `show more asks the next page`, `the coverage notices show what was not seen`, `the truncated note shows`,
  `the out-of-date note shows after a cleanup`.
- `frame-sidebar` / `app-frame` tests: the entry is there, current on Largest, and opens it.
- `npx vitest run app/ui/src/product/own-item-row app/ui/src/screens/largest app/ui/src/screens`; `npm run lint:ts`
  (the boundaries: `product/` never imports `ipc/index` or `state/`; the type check); the gallery test.
- Commit: `feat(ui): the Largest screen's entry, Files list and row`.

## Conclusions

- The screen `screens/largest/` (the sidebar entry between Scan and Review, `Screen` `largest`), the component
  `product/own-item-row/`, the three IPC functions, and the Files list with "Show more". `getStoragePlace` and its mocks
  are gone from the UI.
- A row shows a checkbox only when offered and given `onToggle`; "Changed" uses the app's medium date; the empty,
  no-scan and refusal states are plain centred text.
- Review found a double-click on "Show more" landing a page twice: fixed with an in-flight guard and rows replaced by
  offset. A failed "Show more" keeps the list; the screen says "Reading the list." while loading.
- The new sidebar item moved 104 Playwright baselines (all looked at). Vitest 1 226 tests at this step.
