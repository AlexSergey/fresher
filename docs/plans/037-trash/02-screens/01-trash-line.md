# 037/02/01 — The Trash line and the result's invitation

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** app/ui

The scan screen shows, under the coverage notices, what waits in each Trash and how much of it Fresher's cleanups put
there, with **Open Trash**. The result of a cleanup to the Trash gets the same invitation under **Show in the Trash**.
No control empties the Trash (decision 1). The design: [`../00-design.md`](../00-design.md), section 5.

Read first: `CLAUDE.md` (TypeScript / React rules, test structure), `app/ui/src/screens/scan/scan-screen.tsx`,
`app/ui/src/product/result-summary/result-summary.tsx` and its test and stories, `app/ui/src/product/cleanup-row/`
(a small product component with stories and CSS module), `app/ui/src/design/primitives/button/button.tsx` (its
`description`), `app/ui/src/i18n/` (`t`, `formatSize`, the `date` argument), `app/ui/src/state/store.ts` and
`reduce.ts` (where `cleanup` and `overview.lastScan` live), `app/ui/src/state/cleanup-actions.ts` (`showInTrash`: how
a refusal reaches the frame's toast), `app/ui/src/ipc/mock.ts` (`mockCore`: a command without an answer fails).

## Files

- `app/ui/src/locales/en.json` — the texts, exactly:
  - `"ui.trash.waiting": "Waiting in the Trash: {size}."`
  - `"ui.trash.waiting-share": "Waiting in the Trash: {size}, {share} of it from cleanups here."`
  - `"ui.trash.waiting-all-ours": "Waiting in the Trash: {size}, all of it from cleanups here."`
  - `"ui.trash.share-only": "Waiting in the Trash from cleanups here: {share}. The rest of the Trash was not measured."`
  - `"ui.trash.gone-by": "Finder removes them by itself by {date}."`
  - `"ui.trash.invite": "Look at what is there and empty it yourself if you no longer need it."`
  - `"ui.trash.open": "Open Trash"`
  - `"ui.trash.label": "The Trash"` (the block's accessible name)
- `app/ui/src/product/trash-line/trash-line.tsx` (new) — `TrashLines` (presentational):
  `{ readonly lines: readonly TrashLine[]; readonly onOpen: (index: number) => void }`. Renders nothing for no line;
  otherwise a `<section aria-label={t('ui.trash.label')}>` with one block per line: the volume's name first only when
  there are two lines or more and the name is known; the sentence chosen as below; `goneBy` when given; then the
  invitation and a `Button` **Open Trash** with `description={t('ui.trash.invite')}` (the invitation is shown as the
  button's description, not a second paragraph, if the `Button` renders its description visibly; otherwise a
  paragraph plus `aria-describedby`). Sizes are `allocated` with their completeness (`formatSize`). The choice:
  - `waiting === null` → `ui.trash.share-only`;
  - `fromCleanups.allocated === 0` → `ui.trash.waiting`;
  - `fromCleanups.allocated >= waiting.allocated` → `ui.trash.waiting-all-ours`;
  - otherwise `ui.trash.waiting-share`.
  A short comment above the component says it is information only and that nothing here empties a Trash.
- `app/ui/src/product/trash-line/trash-line.module.css` (new) — quiet, secondary text in the scan screen's tokens; no
  new colour.
- `app/ui/src/product/trash-line/trash-line.stories.tsx` (new) — one line with a share; all ours; no share; not
  measured; with the date; two volumes (as the siblings' stories are written, so the gallery shows them).
- `app/ui/src/product/trash-line/trash-line.test.tsx` (new) — `describe('negative cases')`: renders nothing for no
  line; no volume name for a single line; no date without `goneBy`; **no button or text names emptying** (no element
  whose accessible name matches `/empty/i` but the invitation sentence itself, and no button but Open Trash);
  `describe('positive cases')`: each of the four sentences with its sizes; the date; the volume names with two lines;
  Open Trash calls `onOpen` with the line's index.
- `app/ui/src/screens/scan/use-trash.ts` (new) — `useTrash(): readonly TrashLine[]`: asks `getTrash()` when the scan
  screen mounts, when `overview.lastScan?.scan` changes, when the store's cleanup turns `finished` (its id changes),
  and on the window's `focus` event (removed on unmount). An answer that comes after a newer request is dropped. A
  failure answers no line and **dispatches nothing** (the line is information; the tests that mock no `getTrash`
  keep passing). `openTrash(index)` is exposed through a second export, `openTrashLine(store)(index)`, whose refusal
  goes to the frame's toast the way `showInTrash`'s does.
- `app/ui/src/screens/scan/use-trash.test.ts` (new) — negatives first: a failed answer gives no line and no
  `command-failed`; a late answer of an older request is dropped; then: asks on mount, after a new scan, after a
  finished cleanup and on focus; stops listening to focus on unmount.
- `app/ui/src/screens/scan/scan-screen.tsx` — `<TrashLines>` after `<CoverageNotices />`, not while scanning; the
  header comment's list names it.
- `app/ui/src/screens/scan/scan-screen.test.tsx` — the line shows with a mocked `getTrash` answer, not during a scan,
  and Open Trash calls `openTrash` with the index (negatives first).
- `app/ui/src/product/result-summary/result-summary.tsx` — under the actions, when `trash && counts.moved > 0 &&
  onShowInTrash !== undefined`, the invitation `t('ui.trash.invite')` as Show in the Trash's `description` (or a
  paragraph tied by `aria-describedby`, as above); nothing else changes. Its test: negatives first (no invitation after
  a permanent delete, none when nothing moved); then the invitation after a move. Its stories show it.
- `app/ui/src/ipc/mock.ts` — `getTrash` added to the answers a default mock helper gives, if one exists (as
  `sampleStorageAnswers` is spread), answering `{ trashes: [] }`.

## Checks

- `npx vitest run app/ui/src/product/trash-line app/ui/src/screens/scan app/ui/src/product/result-summary`.
- `npm run typecheck`, `npm run lint` (ESLint, knip, Stylelint, Prettier on the changed files).
- UI coverage stays at or above 90 % of lines (`npm run test:coverage` or the repository's script, read the total).
- The Playwright baselines are 03/03's; nothing in `app/ui/e2e/` changes here.

## Conclusions

Built: `product/trash-line/` (component, styles, stories, tests), `screens/scan/use-trash.ts` (mount, new scan,
finished cleanup and window focus refresh; a failed call hides the line), the line on the scan screen, the invitation
under Show in the Trash on the result, the `ui.trash.*` texts, and the baselines that moved (the result, the gallery,
the keyboard story), each looked at. `vitest`: 133 files, 1189 tests, all passed; `lint:ts`, `knip`, eslint and
prettier clean. A test fails on any button whose name matches "empty".

What changed: `Button` has no `description` prop, so the invitation is a paragraph tied by `aria-describedby`; no
mock default helper existed, so `mock.ts` is unchanged. Playwright full run at this step: 568 passed; the failures were
the expected baseline moves, a keyboard story that passes alone and the known chromium baseline (below).
UI coverage was not measured separately here; `npm run pre-push` runs it (see 04/01).

Left: a failed focus refresh hides the line for a moment (a flicker).
