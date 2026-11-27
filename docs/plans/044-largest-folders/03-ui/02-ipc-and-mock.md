# 044/03/02 — The IPC wrappers and the mock

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** app/ui, e2e

The UI's typed calls for Folders, and the e2e core mock and the unit tests' data that answer them. The design:
[`../00-design.md`](../00-design.md), section 9.

## Read first

02/04's commit (the bindings); `app/ui/src/ipc/{index.ts,index.test.ts,mock.ts}` (`COMMANDS`, `call`,
`getLargest`, `revealFinding`); `app/ui/e2e/support/largest-mock.ts` and `app/ui/e2e/core-mock.spec.ts` (:492);
`app/ui/src/test/largest-scan.ts`.

## Files

- `app/ui/src/ipc/index.ts` — `COMMANDS.getLargestFolders: 'get_largest_folders'`, `COMMANDS.revealNode:
  'reveal_node'`; `export const getLargestFolders = (query: FoldersQuery): Promise<FoldersPage> =>
  call('getLargestFolders', { query });` ("One page of the folders of the home left alone, largest first.") and
  `export const revealNode = (node: NodeId): Promise<void> => call('revealNode', { node });` ("Show in Finder of a
  Largest row: the UI names the node's id, the core holds its path.").
- `app/ui/src/test/largest-scan.ts` — `FOLDERS: FoldersPage`: five rows (`~/Code/old-app` a project of 14 months,
  `~/Archive` of 3 years, `~/Public/old-export`, `~/Renders`, `~/Work/datasets` a project), with `home`, `sizes`,
  `projects: 2`, `minSize` 1 GB, `days` 365.
- `app/ui/e2e/support/largest-mock.ts` — the script's `folders` (from `FOLDERS`); `get_largest_folders` answers them
  largest first, paged by `offset`/`limit`, `no-scan` before a scan; `reveal_node` answers `null` for a listed node,
  `stale-node` otherwise, and records the call (`revealed(page)` reads the list, as the mock's picks are read);
  `get_largest_place` stays (Browse).

## Interfaces produced

- `getLargestFolders`, `revealNode`; the mock's `folders` and `revealed`; `FOLDERS` for 03/03 and 03/04.

## Tests (negatives first)

- `app/ui/src/ipc/index.test.ts`, negative: `getLargestFolders passes a refusal on`; positive: `getLargestFolders
  sends the query`, `revealNode sends the node's id and nothing else`.
- `app/ui/e2e/core-mock.spec.ts`, negative: `the folders are refused before a scan`, `a reveal of an unlisted node is
  refused`; positive: `the folders come largest first, a page at a time`, `a reveal of a listed folder is recorded`.

## Checks

- `npx vitest run src/ipc/index.test.ts` (from `app/ui`)
- `npx playwright test --project chromium e2e/core-mock.spec.ts`
- `npm run lint:ts`, `npm run lint:code`, `npm run knip` (the new exports are used by the tests; 03/04 uses them in
  the screen)

Commit: `feat(ui): the calls for Largest's folders, and their mock`.
