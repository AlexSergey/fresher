import type { NodeInfo, NodeKind, PlaceRow, PlaceView, Sizes } from '../ipc/bindings.generated';

// The core's answers of the place query on the demo volume, for the e2e mock (no screen shows the view by place now):
// the home folder, and a descent into an app's container.

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

const HOME_FOLDER = nodeOf('home', 300, { index: 0 });

// The demo volume's root after the scan: its folders largest first.
export const DEMO_PLACE: PlaceView = {
  offset: 0,
  path: [HOME_FOLDER],
  rows: [
    rowOf(nodeOf('Library', 140, { index: 1 }), 9),
    rowOf(nodeOf('Movies', 80, { index: 2 })),
    rowOf(nodeOf('Downloads', 40, { index: 3 }), 40),
    rowOf(nodeOf('Documents', 30, { index: 4 })),
    rowOf(nodeOf('archive.zip', 6, { index: 5, kind: 'file' })),
    rowOf(nodeOf('Shared', 0.001, { index: 6, kind: 'link' })),
  ],
  total: 6,
};

const CONTAINERS = nodeOf('Containers', 95, { index: 11 });
const APP = nodeOf('com.example.app', 92, { index: 12 });

// The demo volume's Library opened: the descent into an app's container.
export const DEMO_DESCENT: PlaceView = {
  offset: 0,
  path: [nodeOf('Library', 100, { index: 10 }), CONTAINERS, APP],
  rows: [
    rowOf(nodeOf('Data', 88, { index: 13 }), 12),
    rowOf(nodeOf('.com.apple.containermanagerd.metadata.plist', 0.004, { index: 14, kind: 'file' })),
  ],
  total: 2,
};
