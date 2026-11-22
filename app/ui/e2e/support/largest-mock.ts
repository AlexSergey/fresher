import type { Page } from '@playwright/test';

import type {
  CleanupPlanView,
  Finding,
  FindingId,
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
import { LARGEST_ROWS, largestPageOf, ownFindingOf } from '../../src/test/largest-scan';

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

const folderRowOf = ({ node, offer, opens }: Folder, picks: Picks): OwnRow => ({
  explained: sized(0),
  kind: 'folder',
  node,
  offer,
  opens,
  picked: picks.ids.get(keyOf(node.id)) ?? null,
});

const fileRowOf = (row: LargestRow, picks: Picks): OwnRow => ({
  explained: sized(0),
  kind: row.kind,
  node: row.node,
  offer: row.offer,
  opens: false,
  picked: picks.ids.get(keyOf(row.node.id)) ?? null,
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
    const rows = FOLDERS.filter(({ node }) => !picks.gone.has(keyOf(node.id))).map((one) => folderRowOf(one, picks));

    return pageOf([HOME], rows, offset, limit);
  }
  const opened = FOLDERS.find(({ node }) => keyOf(node.id) === keyOf(folder));

  if (opened === undefined || folder.state !== STATE) {
    return 'stale-node';
  }
  if (!opened.opens) {
    return 'not-own-item';
  }
  const rows = LARGEST_ROWS.filter((row) => topOf(row) === opened && !picks.gone.has(keyOf(row.node.id))).map((row) =>
    fileRowOf(row, picks),
  );

  return pageOf([HOME, opened.node], rows, offset, limit);
};

// The finding the core makes of a node the user ticks, or `null` when it does not offer it: a row of the Files list
// or a folder of the home that is offered.
export const pickAnswerOf = (node: NodeId, picks: Picks = NO_PICKS): Finding | null => {
  const id = ownIdOf(node);
  const row = LARGEST_ROWS.find((one) => keyOf(one.node.id) === keyOf(node));
  const folder = FOLDERS.find((one) => keyOf(one.node.id) === keyOf(node));

  if (picks.gone.has(keyOf(node)) || node.state !== STATE) {
    return null;
  }
  if (row !== undefined) {
    return row.offer.kind === 'offered' ? ownFindingOf(row, id) : null;
  }
  if (folder?.offer.kind !== 'offered') {
    return null;
  }

  return ownFindingOf(
    { kind: 'folder', node: folder.node, offer: folder.offer, picked: null, place: '~', unpacked: null },
    id,
  );
};

const NOT_OWN_ITEM: Refusal = {
  error: { code: 'not-own-item' },
  text: { args: {}, key: 'core.error.not-own-item' },
};

const STALE_NODE: Refusal = { error: { code: 'stale-node' }, text: { args: {}, key: 'core.error.stale-node' } };

const COMMAND_NAMES = [COMMANDS.getLargest, COMMANDS.getLargestPlace, COMMANDS.pickOwnItem] as const;

type Answer = { readonly answer: unknown } | { readonly refused: Refusal };

// The page's name of the hook the mocked core calls first for every command.
export const LARGEST_HOOK = '__fresherLargestInvoke';

const ANSWER = '__fresherLargestAnswer';

type Setup = {
  readonly answer: string;
  readonly commands: readonly string[];
  readonly hook: string;
};

// Adds the Largest commands to the mocked core of `core-mock.ts`, which calls the hook for every command. The findings
// the user picked join `findings`, so the cleanup mock plans over them; `moved` is told what a cleanup moved.
export const mockLargest = async (page: Page, findings: Finding[]): Promise<(plan: CleanupPlanView) => void> => {
  const ids = new Map<number, FindingId>();
  const gone = new Set<number>();
  const picks = (): Picks => ({ gone, ids });

  await page.exposeFunction(ANSWER, (command: string, args: Record<string, unknown>): Answer => {
    switch (command) {
      case COMMANDS.getLargest:
        return { answer: largestAnswerOf(args.query as LargestQuery, picks()) };
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

      scope[setup.hook] = (command: string, args: Record<string, unknown>): Promise<unknown> | undefined => {
        if (!setup.commands.includes(command)) {
          return undefined;
        }
        const answer = scope[setup.answer] as (command: string, args: Record<string, unknown>) => Promise<Answer>;

        return answer(command, args).then((result) =>
          'refused' in result
            ? // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors -- the core rejects with a plain object
              Promise.reject(result.refused)
            : result.answer,
        );
      };
    },
    { answer: ANSWER, commands: COMMAND_NAMES, hook: LARGEST_HOOK } satisfies Setup,
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
