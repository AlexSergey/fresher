import type {
  Dictionary,
  Finding,
  FolderRow,
  FoldersPage,
  FoldersQuery,
  ItemKind,
  LargestPage,
  LargestQuery,
  LargestRow,
  NodeInfo,
  Offer,
  Sizes,
} from '../ipc/bindings.generated';

import coreTexts from '../../../../crates/core/locales/en.json' with { type: 'json' };
import { DEMO_DICTIONARY } from './demo-scan';

// The Files list of the Largest screen as the core answers it for one scan of a home folder, and the finding each
// offered row becomes when the user ticks it. The texts are the core's own, read from its locale file, so a changed
// text changes the tests. Only what the core produces together: a package is a collapsed node, offered; a package read
// in part is withheld; the archive names the folder beside it; the rows come largest first. Shared by the unit tests
// and the e2e mock.

const MB = 1_000_000;
const GB = 1_000 * MB;

const HOME = '/Users/someone';

const STATE = 1;

// 1 January 2025 and 1 January 2023, in seconds.
const RECENT = 1_735_689_600;
const OLD = 1_672_531_200;

const sized = (bytes: number, completeness: Sizes['completeness'] = 'complete'): Sizes => ({
  allocated: bytes,
  completeness,
  logical: bytes,
  reclaimable: bytes,
});

// The core's texts of the Largest screen and of the items it picks, with the demo's.
export const LARGEST_DICTIONARY: Dictionary = {
  ...DEMO_DICTIONARY,
  ...Object.fromEntries(
    Object.entries(coreTexts as Dictionary).filter(
      ([key]) => key.startsWith('core.largest.') || key.startsWith('core.own-file.'),
    ),
  ),
};

type Spec = {
  readonly bytes: number;
  readonly completeness?: Sizes['completeness'];
  readonly index: number;
  readonly kind: ItemKind;
  readonly modified: null | number;
  readonly name: string;
  readonly node?: NodeInfo['kind'];
  readonly offer: Offer;
  readonly place: string;
  readonly unpacked?: string;
};

const OFFERED: Offer = { kind: 'offered' };

const SPECS: readonly Spec[] = [
  {
    bytes: 12 * GB,
    index: 101,
    kind: 'disk-image',
    modified: OLD,
    name: 'ubuntu-24.04.dmg',
    offer: OFFERED,
    place: '~/Downloads',
  },
  {
    bytes: 9 * GB,
    index: 102,
    kind: 'virtual-machine',
    modified: RECENT,
    name: 'Windows 11.pvm',
    node: 'collapsed',
    offer: OFFERED,
    place: '~/Parallels',
  },
  {
    bytes: 4_800 * MB,
    completeness: 'partial',
    index: 103,
    kind: 'media-library',
    modified: RECENT,
    name: 'Photos Library.photoslibrary',
    node: 'collapsed',
    offer: { kind: 'withheld', reason: 'read-in-part' },
    place: '~/Pictures',
  },
  {
    bytes: 2_400 * MB,
    index: 104,
    kind: 'video',
    modified: OLD,
    name: 'holiday-2022.mov',
    offer: OFFERED,
    place: '~/Movies',
  },
  {
    bytes: 1_100 * MB,
    index: 105,
    kind: 'archive',
    modified: null,
    name: 'project-backup.zip',
    offer: OFFERED,
    place: '~/Documents',
    unpacked: 'project-backup',
  },
];

const nodeOf = (spec: Spec): NodeInfo => ({
  files: 0,
  folders: 0,
  id: { index: spec.index, state: STATE },
  kind: spec.node ?? 'file',
  modified: spec.modified,
  name: spec.name,
  notScanned: null,
  owner: null,
  sizes: sized(spec.bytes, spec.completeness),
});

const rowOf = (spec: Spec): LargestRow => ({
  kind: spec.kind,
  node: nodeOf(spec),
  offer: spec.offer,
  picked: null,
  place: spec.place,
  unpacked: spec.unpacked ?? null,
});

// The rows of the last scan, largest first: a disk image, a virtual machine package, a package read in part (withheld),
// a video and an archive with the folder beside it.
export const LARGEST_ROWS: readonly LargestRow[] = SPECS.map(rowOf);

// The smallest size listed: the setting's default, 100 MB.
export const LARGEST_MIN_SIZE = 100 * MB;

const totalSizes = (rows: readonly LargestRow[]): Sizes => {
  const bytes = rows.reduce((sum, { node }) => sum + node.sizes.allocated, 0);

  return sized(bytes, rows.some(({ node }) => node.sizes.completeness !== 'complete') ? 'partial' : 'complete');
};

// What `get_largest` answers for a query over `rows` (offset and limit only): the page, the total and the sizes of
// every row.
export const largestPageOf = (query: LargestQuery, rows: readonly LargestRow[] = LARGEST_ROWS): LargestPage => ({
  kinds: [],
  minSize: LARGEST_MIN_SIZE,
  offset: query.offset,
  places: [],
  rows: rows.slice(query.offset, query.offset + query.limit),
  sizes: totalSizes(rows),
  total: rows.length,
  truncated: false,
});

// The first page of the last scan's list, as the e2e mock answers it.
export const LARGEST_PAGE: LargestPage = largestPageOf({
  kind: null,
  limit: 100,
  offset: 0,
  place: null,
  unchanged: 'any',
});

type FolderSpec = {
  readonly bytes: number;
  readonly index: number;
  readonly leftFor: FolderRow['leftFor'];
  readonly name: string;
  readonly place: string;
  readonly project: boolean;
};

const FOLDER_SPECS: readonly FolderSpec[] = [
  {
    bytes: 16 * GB,
    index: 201,
    leftFor: { count: 3, unit: 'years' },
    name: 'Archive',
    place: '~',
    project: false,
  },
  {
    bytes: 13 * GB,
    index: 202,
    leftFor: { count: 14, unit: 'months' },
    name: 'old-app',
    place: '~/Code',
    project: true,
  },
  {
    bytes: 9 * GB,
    index: 203,
    leftFor: { count: 2, unit: 'years' },
    name: 'Renders',
    place: '~',
    project: false,
  },
  {
    bytes: 7 * GB,
    index: 204,
    leftFor: { count: 18, unit: 'months' },
    name: 'datasets',
    place: '~/Work',
    project: true,
  },
  {
    bytes: 2 * GB,
    index: 205,
    leftFor: { count: 5, unit: 'months' },
    name: 'old-export',
    place: '~/Public',
    project: false,
  },
];

const folderRowOf = (spec: FolderSpec): FolderRow => ({
  leftFor: spec.leftFor,
  newest: OLD,
  node: nodeOf({ ...spec, kind: 'folder', modified: OLD, node: 'folder', offer: OFFERED }),
  place: spec.place,
  project: spec.project,
});

const FOLDER_ROWS: FolderRow[] = FOLDER_SPECS.map(folderRowOf);

// The folders of the home left alone, as `get_largest_folders` answers the first page: five folders, two of them
// projects, largest first, with the home's sizes for the bar.
export const FOLDERS: FoldersPage = {
  days: 365,
  home: sized(212 * GB),
  minSize: GB,
  offset: 0,
  projects: 2,
  rows: FOLDER_ROWS,
  sizes: sized(FOLDER_SPECS.reduce((sum, { bytes }) => sum + bytes, 0)),
  total: FOLDER_ROWS.length,
};

// The five folders, then `extra` smaller ones of a project each, for the bar's rest and the paging.
export const manyFolders = (extra: number): readonly FolderRow[] => [
  ...FOLDER_ROWS,
  ...Array.from({ length: extra }, (_, at) =>
    folderRowOf({
      bytes: (1_000 - at) * MB,
      index: 1_200 + at,
      leftFor: { count: 13, unit: 'months' },
      name: `folder-${String(at + 1)}`,
      place: '~/Work',
      project: true,
    }),
  ),
];

// What `get_largest_folders` answers for a query over `rows`: the page, the total, the sizes and the projects of every
// row, with `FOLDERS`' home and rule.
export const foldersPageOf = ({ limit, offset }: FoldersQuery, rows: readonly FolderRow[]): FoldersPage => ({
  ...FOLDERS,
  offset,
  projects: rows.filter(({ project }) => project).length,
  rows: rows.slice(offset, offset + limit),
  sizes: sized(rows.reduce((sum, { node }) => sum + node.sizes.allocated, 0)),
  total: rows.length,
});

// A list of `count` offered files, each smaller than the one before, for the paging.
export const manyRows = (count: number): readonly LargestRow[] =>
  Array.from({ length: count }, (_, at) =>
    rowOf({
      bytes: (count - at + 100) * MB,
      index: 1_000 + at,
      kind: 'file',
      modified: RECENT,
      name: `file-${String(at + 1)}.bin`,
      offer: OFFERED,
      place: '~/Downloads',
    }),
  );

// The finding the engine makes of an offered row the user ticks: the core's own category, High, to the Trash.
export const ownFindingOf = (row: LargestRow, id: number): Finding => {
  const inHome = `${row.place.replace(/^~\/?/, '')}/${row.node.name}`.replace(/^\//, '');
  const path = `${HOME}/${inHome}`;
  const folder = row.node.kind === 'folder' || row.node.kind === 'collapsed';

  return {
    category: 'core.own-file',
    danger: 'high',
    id,
    key: { identity: `own/${inHome}`, module: 'core' },
    module: 'core',
    reason: { args: {}, key: folder ? 'core.own-file.reason.folder' : 'core.own-file.reason.file' },
    removability: 'removable',
    removal: { kind: 'move-to-trash', value: path },
    sizes: row.node.sizes,
    subject: { kind: 'path', value: path },
    title: { args: { name: { kind: 'text', value: row.node.name } }, key: 'core.own-file.title' },
  };
};
