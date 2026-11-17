import type { NodeInfo, NodeKind, PlaceRow, Sizes } from '../../ipc/bindings.generated';

// The places of the place list's stories and tests: a home, a descent into an app's container, a partial folder, an
// unreadable folder and an empty one.

const GB = 1_000_000_000;

const sizesOf = (gigabytes: number, completeness: Sizes['completeness'] = 'complete'): Sizes => ({
  allocated: Math.round(gigabytes * GB),
  completeness,
  logical: Math.round(gigabytes * GB),
  reclaimable: Math.round(gigabytes * GB),
});

const NONE = sizesOf(0);

// A node of state 1, at `index`.
const nodeOf = (
  name: string,
  gigabytes: number,
  {
    completeness = 'complete',
    index,
    kind = 'folder',
    notScanned = null,
  }: {
    readonly completeness?: Sizes['completeness'];
    readonly index: number;
    readonly kind?: NodeKind;
    readonly notScanned?: NodeInfo['notScanned'];
  },
): NodeInfo => ({
  files: kind === 'folder' ? 10 : 0,
  folders: kind === 'folder' ? 2 : 0,
  id: { index, state: 1 },
  kind,
  modified: null,
  name,
  notScanned,
  owner: null,
  sizes: sizesOf(gigabytes, completeness),
});

const rowOf = (node: NodeInfo, explained = 0): PlaceRow => ({
  explained: explained === 0 ? NONE : sizesOf(explained),
  node,
});

// A place as the list shows it: the way down, the folder open and its rows, largest first.
export type StoryPlace = {
  readonly folder: NodeInfo;
  readonly path: readonly NodeInfo[];
  readonly rows: readonly PlaceRow[];
};

const HOME_FOLDER = nodeOf('home', 300, { index: 0 });

export const HOME: StoryPlace = {
  folder: HOME_FOLDER,
  path: [HOME_FOLDER],
  rows: [
    rowOf(nodeOf('Library', 140, { index: 1 }), 9),
    rowOf(nodeOf('Movies', 80, { index: 2 })),
    rowOf(nodeOf('Downloads', 40, { index: 3 }), 40),
    rowOf(nodeOf('Documents', 30, { index: 4 })),
    rowOf(nodeOf('archive.zip', 6, { index: 5, kind: 'file' })),
    rowOf(nodeOf('Shared', 0.001, { index: 6, kind: 'link' })),
  ],
};

const CONTAINERS = nodeOf('Containers', 95, { index: 11 });
const APP = nodeOf('com.example.app', 92, { index: 12 });

export const DESCENT: StoryPlace = {
  folder: APP,
  path: [nodeOf('Library', 100, { index: 10 }), CONTAINERS, APP],
  rows: [
    rowOf(nodeOf('Data', 88, { index: 13 }), 12),
    rowOf(nodeOf('.com.apple.containermanagerd.metadata.plist', 0.004, { index: 14, kind: 'file' })),
  ],
};

const PARTIAL_FOLDER = nodeOf('Library', 140, { completeness: 'partial', index: 20 });

export const PARTIAL: StoryPlace = {
  folder: PARTIAL_FOLDER,
  path: [PARTIAL_FOLDER],
  rows: [
    rowOf(nodeOf('Application Support', 70, { completeness: 'partial', index: 21 }), 4),
    rowOf(nodeOf('Caches', 40, { index: 22 }), 38),
    rowOf(nodeOf('Containers', 30, { completeness: 'partial', index: 23 })),
  ],
};

const UNREADABLE_FOLDER = nodeOf('Library', 60, { completeness: 'partial', index: 30 });

export const UNREADABLE: StoryPlace = {
  folder: UNREADABLE_FOLDER,
  path: [UNREADABLE_FOLDER],
  rows: [
    rowOf(nodeOf('Caches', 52, { index: 31 }), 50),
    rowOf(
      nodeOf('Mail', 0, {
        completeness: 'unknown',
        index: 32,
        notScanned: { kind: 'unreadable', value: 'guarded' },
      }),
    ),
    rowOf(nodeOf('Mobile Documents', 0, { completeness: 'unknown', index: 33, notScanned: { kind: 'avoided' } })),
    rowOf(nodeOf('Backup', 8, { index: 34, notScanned: { kind: 'other-volume' } })),
  ],
};

const EMPTY_FOLDER = nodeOf('Empty', 0, { index: 40 });

export const EMPTY: StoryPlace = { folder: EMPTY_FOLDER, path: [EMPTY_FOLDER], rows: [] };
