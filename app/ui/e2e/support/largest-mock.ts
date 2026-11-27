import type { Page } from '@playwright/test';

import type {
  CleanupPlanView,
  Finding,
  FindingId,
  FoldersPage,
  FoldersQuery,
  ItemKind,
  KindCount,
  LargestPage,
  LargestQuery,
  LargestRow,
  NodeId,
  NodeInfo,
  Offer,
  OwnPlace,
  OwnRow,
  PlaceCount,
  Refusal,
  Sizes,
  Unchanged,
} from '../../src/ipc/bindings.generated';

import { COMMANDS } from '../../src/ipc';
import { LARGEST_ROWS, largestPageOf, ownFindingOf, FOLDERS as SCRIPTED_FOLDERS } from '../../src/test/largest-scan';

// The Largest screen's commands of the mocked core: the Files list filtered as the core filters it, the home's folders
// and one folder's page, and the pick of a row. It states only what the core can produce together: a pick only of a row
// it offers, always the same finding for the same row, never pre-selected; the picks a cleanup moved leave both lists.

const STATE = 1;

const MB = 1_000_000;
const GB = 1_000 * MB;

// The scan's time (1 January 2025): `unchanged` counts back from it, never from the clock.
const TAKEN = 1_735_689_600;

const DAY = 86_400;

const DAYS: Readonly<Record<Unchanged, number>> = { any: 0, 'half-year': 182, month: 30, 'two-years': 730, year: 365 };

// The kinds in the core's order (the enum's), which the kind filter lists.
const KINDS: readonly ItemKind[] = [
  'disk-image',
  'archive',
  'installer',
  'video',
  'audio',
  'image',
  'virtual-machine',
  'media-library',
  'project',
  'memory-dump',
  'document',
  'device-backup',
  'file',
  'folder',
];

const HOME_INDEX = 1;

const sized = (bytes: number, completeness: Sizes['completeness'] = 'complete'): Sizes => ({
  allocated: bytes,
  completeness,
  logical: bytes,
  reclaimable: bytes,
});

const folderOf = (
  index: number,
  name: string,
  bytes: number,
  completeness: Sizes['completeness'] = 'complete',
): NodeInfo => ({
  files: 0,
  folders: 0,
  id: { index, state: STATE },
  kind: 'folder',
  modified: null,
  name,
  notScanned: null,
  owner: null,
  sizes: sized(bytes, completeness),
});

const HOME: NodeInfo = folderOf(HOME_INDEX, 'someone', 0);

type Folder = {
  readonly node: NodeInfo;
  readonly offer: Offer;
  readonly opens: boolean;
};

// The home's folders as the folders view lists them, largest first: Library is withheld and does not open, the
// standard folders open but cannot be chosen, Parallels is offered.
const FOLDERS: readonly Folder[] = [
  { node: folderOf(11, 'Library', 22 * GB), offer: { kind: 'withheld', reason: 'library' }, opens: false },
  { node: folderOf(12, 'Downloads', 12 * GB), offer: { kind: 'offered' }, opens: true },
  { node: folderOf(13, 'Parallels', 9 * GB), offer: { kind: 'offered' }, opens: true },
  {
    node: folderOf(14, 'Pictures', 4_800 * MB, 'partial'),
    offer: { kind: 'withheld', reason: 'standard-folder' },
    opens: true,
  },
  { node: folderOf(15, 'Movies', 2_400 * MB), offer: { kind: 'withheld', reason: 'standard-folder' }, opens: true },
  { node: folderOf(16, 'Documents', 1_100 * MB), offer: { kind: 'withheld', reason: 'standard-folder' }, opens: true },
];

// The folder a row of the Files list is in: `~/Movies` is Movies.
const topOf = ({ place }: LargestRow): Folder | undefined => FOLDERS.find(({ node }) => place === `~/${node.name}`);

// The finding ids the user picked by node index, and the nodes a cleanup moved to the Trash.
export type Picks = {
  readonly gone: ReadonlySet<number>;
  readonly ids: ReadonlyMap<number, FindingId>;
};

const NO_PICKS: Picks = { gone: new Set(), ids: new Map() };

// The finding id the mock gives the pick of a node (the same one every time).
export const ownIdOf = (node: NodeId): FindingId => 9_000 + node.index;

const keyOf = (node: NodeId): number => node.index;

const withPick = (row: LargestRow, { ids }: Picks): LargestRow => ({
  ...row,
  picked: ids.get(keyOf(row.node.id)) ?? null,
});

const unchangedBefore = (unchanged: Unchanged): null | number =>
  unchanged === 'any' ? null : TAKEN - DAYS[unchanged] * DAY;

// What `get_largest` answers: the rows under the three filters, the kinds under the other filters, the folders under
// the other filters (the most items first, then by name), as the core counts them.
export const largestAnswerOf = (query: LargestQuery, picks: Picks = NO_PICKS): LargestPage => {
  const before = unchangedBefore(query.unchanged);
  const kept = LARGEST_ROWS.filter(({ node }) => !picks.gone.has(keyOf(node.id)));
  const old = ({ node }: LargestRow): boolean => before === null || (node.modified !== null && node.modified < before);
  const inKind = ({ kind }: LargestRow): boolean => query.kind === null || kind === query.kind;
  const inPlace = (row: LargestRow): boolean => {
    const top = topOf(row);

    return query.place === null || (top !== undefined && keyOf(top.node.id) === keyOf(query.place));
  };
  const rows = kept.filter((row) => old(row) && inKind(row) && inPlace(row));
  const kinds: KindCount[] = KINDS.flatMap((kind) => {
    const count = kept.filter((row) => old(row) && inPlace(row) && row.kind === kind).length;

    return count === 0 ? [] : [{ count, kind }];
  });
  const places: PlaceCount[] = FOLDERS.flatMap(({ node }) => {
    const count = kept.filter((row) => old(row) && inKind(row) && topOf(row)?.node.id.index === node.id.index).length;

    return count === 0 ? [] : [{ count, folder: node }];
  }).sort((a, b) => b.count - a.count || a.folder.name.localeCompare(b.folder.name));

  return {
    ...largestPageOf(
      query,
      rows.map((row) => withPick(row, picks)),
    ),
    kinds,
    places,
  };
};

const folderRowOf = ({ node, offer, opens }: Folder): OwnRow => ({
  explained: sized(0),
  kind: 'folder',
  node,
  offer,
  opens,
});

const fileRowOf = (row: LargestRow): OwnRow => ({
  explained: sized(0),
  kind: row.kind,
  node: row.node,
  offer: row.offer,
  opens: false,
});

const pageOf = (path: readonly NodeInfo[], rows: readonly OwnRow[], offset: number, limit: number): OwnPlace => ({
  offset,
  path: [...path],
  rows: rows.slice(offset, offset + limit),
  total: rows.length,
});

// What `get_largest_place` answers: the home's folders, or the children of a folder that opens. A node the state does
// not have is stale; a folder that does not open is refused as the core refuses it.
export const placeAnswerOf = (
  folder: NodeId | null,
  offset: number,
  limit: number,
  picks: Picks = NO_PICKS,
): 'not-own-item' | 'stale-node' | OwnPlace => {
  if (folder === null || keyOf(folder) === HOME_INDEX) {
    const rows = FOLDERS.filter(({ node }) => !picks.gone.has(keyOf(node.id))).map(folderRowOf);

    return pageOf([HOME], rows, offset, limit);
  }
  const opened = FOLDERS.find(({ node }) => keyOf(node.id) === keyOf(folder));

  if (opened === undefined || folder.state !== STATE) {
    return 'stale-node';
  }
  if (!opened.opens) {
    return 'not-own-item';
  }
  const rows = LARGEST_ROWS.filter((row) => topOf(row) === opened && !picks.gone.has(keyOf(row.node.id))).map(
    fileRowOf,
  );

  return pageOf([HOME, opened.node], rows, offset, limit);
};

// The finding the core makes of a node the user ticks, or `null` when it does not offer it: only a row of the Files
// list that is offered; a plain folder of Browse is refused.
export const pickAnswerOf = (node: NodeId, picks: Picks = NO_PICKS): Finding | null => {
  const id = ownIdOf(node);
  const row = LARGEST_ROWS.find((one) => keyOf(one.node.id) === keyOf(node));

  if (picks.gone.has(keyOf(node)) || node.state !== STATE) {
    return null;
  }

  return row?.offer.kind === 'offered' ? ownFindingOf(row, id) : null;
};

// The core's most rows a page holds.
const MAX_PAGE = 100;

// What `get_largest_folders` answers: the folders left alone, largest first, one page of them.
const foldersAnswerOf = (folders: FoldersPage, { limit, offset }: FoldersQuery): FoldersPage => {
  const rows = [...folders.rows].sort((a, b) => b.node.sizes.allocated - a.node.sizes.allocated);

  return { ...folders, offset, rows: rows.slice(offset, offset + Math.min(limit, MAX_PAGE)), total: rows.length };
};

// Whether `reveal_node` knows the node: a folder of the list or a node of the tree, of the state the scan has.
const revealsNode = (folders: FoldersPage, node: NodeId): boolean =>
  node.state === STATE &&
  (folders.rows.some((row) => keyOf(row.node.id) === keyOf(node)) ||
    FOLDERS.some((one) => keyOf(one.node.id) === keyOf(node)) ||
    LARGEST_ROWS.some((one) => keyOf(one.node.id) === keyOf(node)));

const NOT_OWN_ITEM: Refusal = {
  error: { code: 'not-own-item' },
  text: { args: {}, key: 'core.error.not-own-item' },
};

const STALE_NODE: Refusal = { error: { code: 'stale-node' }, text: { args: {}, key: 'core.error.stale-node' } };

const NO_SCAN: Refusal = { error: { code: 'no-scan' }, text: { args: {}, key: 'core.error.no-scan' } };

const COMMAND_NAMES = [
  COMMANDS.getLargest,
  COMMANDS.getLargestFolders,
  COMMANDS.getLargestPlace,
  COMMANDS.pickOwnItem,
  COMMANDS.revealNode,
] as const;

type Answer = { readonly answer: unknown } | { readonly refused: Refusal };

// The page's name of the hook the mocked core calls first for every command.
export const LARGEST_HOOK = '__fresherLargestInvoke';

const ANSWER = '__fresherLargestAnswer';

// The nodes the mock revealed in Finder, by index, in the page.
const REVEALED = '__revealedNodes';

type Setup = {
  readonly answer: string;
  readonly commands: readonly string[];
  readonly hook: string;
  readonly revealed: string;
  readonly revealNodeCommand: string;
  readonly startScan: string;
};

// The nodes revealed in Finder so far, in the order of the calls.
export const revealed = (page: Page): Promise<readonly number[]> =>
  page.evaluate((key) => (globalThis as unknown as Record<string, number[]>)[key] ?? [], REVEALED);

// Adds the Largest commands to the mocked core of `core-mock.ts`, which calls the hook for every command. The findings
// the user picked join `findings`, so the cleanup mock plans over them; `moved` is told what a cleanup moved.
export const mockLargest = async (
  page: Page,
  findings: Finding[],
  folders: FoldersPage = SCRIPTED_FOLDERS,
): Promise<(plan: CleanupPlanView) => void> => {
  const ids = new Map<number, FindingId>();
  const gone = new Set<number>();
  const picks = (): Picks => ({ gone, ids });

  await page.exposeFunction(ANSWER, (command: string, args: Record<string, unknown>, scanned: boolean): Answer => {
    switch (command) {
      case COMMANDS.getLargest:
        return { answer: largestAnswerOf(args.query as LargestQuery, picks()) };
      case COMMANDS.getLargestFolders:
        return scanned ? { answer: foldersAnswerOf(folders, args.query as FoldersQuery) } : { refused: NO_SCAN };
      case COMMANDS.getLargestPlace: {
        const answer = placeAnswerOf(
          args.folder as NodeId | null,
          args.offset as number,
          args.limit as number,
          picks(),
        );

        return answer === 'stale-node'
          ? { refused: STALE_NODE }
          : answer === 'not-own-item'
            ? { refused: NOT_OWN_ITEM }
            : { answer };
      }
      case COMMANDS.revealNode:
        if (!scanned) {
          return { refused: NO_SCAN };
        }

        return revealsNode(folders, args.node as NodeId) ? { answer: null } : { refused: STALE_NODE };
      default: {
        const node = args.node as NodeId;
        const finding = pickAnswerOf(node, picks());

        if (finding === null) {
          return { refused: NOT_OWN_ITEM };
        }
        if (!ids.has(keyOf(node))) {
          ids.set(keyOf(node), finding.id);
          findings.push(finding);
        }

        return { answer: finding };
      }
    }
  });
  await page.addInitScript(
    (setup: Setup) => {
      const scope = globalThis as unknown as Record<string, unknown>;

      const seen: number[] = [];

      let scanned = false;

      scope[setup.revealed] = seen;
      scope[setup.hook] = (command: string, args: Record<string, unknown>): Promise<unknown> | undefined => {
        if (command === setup.startScan) {
          scanned = true;
        }
        if (!setup.commands.includes(command)) {
          return undefined;
        }
        const answer = scope[setup.answer] as (
          command: string,
          args: Record<string, unknown>,
          scanned: boolean,
        ) => Promise<Answer>;

        return answer(command, args, scanned).then((result) => {
          if ('refused' in result) {
            // eslint-disable-next-line @typescript-eslint/only-throw-error -- the core rejects with a plain object
            throw result.refused;
          }
          if (command === setup.revealNodeCommand) {
            seen.push((args.node as { readonly index: number }).index);
          }

          return result.answer;
        });
      };
    },
    {
      answer: ANSWER,
      commands: COMMAND_NAMES,
      hook: LARGEST_HOOK,
      revealed: REVEALED,
      revealNodeCommand: COMMANDS.revealNode,
      startScan: COMMANDS.startScan,
    } satisfies Setup,
  );

  return (plan) => {
    const moved = new Set(plan.groups.flatMap((group) => group.items.map((item) => item.finding)));

    for (const [index, id] of ids) {
      if (moved.has(id)) {
        gone.add(index);
      }
    }
  };
};
