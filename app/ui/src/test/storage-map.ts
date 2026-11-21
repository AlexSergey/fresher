import type { CategoryTotal, MapPart, MeaningGroup, PartKind, PlaceView, StorageMap } from '../ipc/bindings.generated';

const GB = 1_000_000_000;

const part = (kind: PartKind, gigabytes: number, reclaimable = 0): MapPart => ({
  kind,
  sizes: {
    allocated: Math.round(gigabytes * GB),
    completeness: 'complete',
    logical: Math.round(gigabytes * GB),
    reclaimable: Math.round(reclaimable * GB),
  },
});

// The parts of the gallery's sample volume: a home volume after a complete scan; they add up to its capacity.
export const SAMPLE_PARTS: readonly MapPart[] = [
  part({ kind: 'explained' }, 64, 58),
  part({ kind: 'your-folders' }, 180),
  part({ kind: 'remainder', outsideScan: true, reasons: [] }, 120),
  part({ kind: 'mac-os' }, 23),
  part({ kind: 'swap-and-sleep' }, 2),
  part({ kind: 'disk-own' }, 1.6),
  part({ kind: 'free' }, 109),
];

// The sample volume's map, phase complete: a 499.6 GB home volume, 18 GB of it purgeable.
export const SAMPLE_STORAGE_MAP: StorageMap = {
  phase: { kind: 'complete', state: 1 },
  volumes: [
    {
      capacity: 499_600_000_000,
      diskChanged: false,
      kind: 'internal',
      name: 'Macintosh HD',
      parts: [...SAMPLE_PARTS],
      places: [],
      purgeable: 18 * GB,
      roots: [{ index: 0, state: 1 }],
    },
  ],
};

export const SAMPLE_MEANING: readonly MeaningGroup[] = SAMPLE_PARTS.map((sample) => ({
  kind: 'part',
  value: sample,
}));

export const EMPTY_PLACE: PlaceView = { offset: 0, path: [], rows: [], total: 0 };

// The core's texts of the storage map (crates/core/locales/en.json, `core.storage.*`), for the tests and the stories.
export const STORAGE_DICTIONARY: Readonly<Record<string, string>> = {
  'core.danger.high.meaning': 'May hold the only copy of your data. Removing it can lose work.',
  'core.danger.high.zone': 'High risk',
  'core.danger.low.meaning': 'Comes back when needed, but that takes time or a download.',
  'core.danger.low.zone': 'Low risk',
  'core.danger.medium.meaning': 'Can be restored, but not by itself. You may need to set something up again.',
  'core.danger.medium.zone': 'Medium risk',
  'core.danger.safe.meaning': 'Comes back by itself when needed. Removing it costs only a little time.',
  'core.danger.safe.zone': 'Safe to remove',
  'core.storage.changed':
    'The disk changed during the scan, so some numbers are a little off. Scan again for exact numbers.',
  'core.storage.kind.external': 'external',
  'core.storage.kind.internal': 'internal',
  'core.storage.kind.network': 'network',
  'core.storage.not-scanned.avoided': 'Not opened: macOS guards it, and Fresher looks there only with your permission.',
  'core.storage.not-scanned.other-volume': 'On another disk, so it was not scanned with this folder.',
  'core.storage.not-scanned.others-folder': 'Belongs to another user of this Mac, so Fresher did not open it.',
  'core.storage.not-scanned.system-items': 'macOS keeps its own items here private, so Fresher never opens them.',
  'core.storage.not-scanned.unreadable.failed': 'Could not be read.',
  'core.storage.not-scanned.unreadable.guarded': 'Protected by macOS. Full Disk Access lets the next scan measure it.',
  'core.storage.not-scanned.unreadable.no-permission': 'Its permissions do not let Fresher read it.',
  'core.storage.part.disk-own.description':
    "Space the disk keeps for itself, such as the recovery system and the file system's records. It cannot be cleaned.",
  'core.storage.part.disk-own.name': "The disk's own space",
  'core.storage.part.explained.description':
    'What Fresher recognised, such as caches, logs and old build files. Each item says how safe it is to remove.',
  'core.storage.part.explained.name': 'Explained',
  'core.storage.part.free.description': 'Space you can still use for new files.',
  'core.storage.part.free.name': 'Free',
  'core.storage.part.mac-os.description':
    'The system itself and the files it needs to start and update. macOS looks after this space; it cannot be cleaned.',
  'core.storage.part.mac-os.name': 'macOS',
  'core.storage.part.not-scanned-yet.description':
    'Space the scan has not reached yet. It shrinks as the scan goes on.',
  'core.storage.part.not-scanned-yet.name': 'Not scanned yet',
  'core.storage.part.other-volume.description':
    'Another volume of yours on the same disk. Add it in Settings → Scan to see inside.',
  'core.storage.part.other-volume.name': 'Another volume',
  'core.storage.part.swap-and-sleep.description':
    'Memory macOS keeps on the disk when it runs low or the Mac sleeps. macOS looks after it; restarting the Mac can ' +
    'make it smaller.',
  'core.storage.part.swap-and-sleep.name': 'Swap and sleep image',
  'core.storage.part.your-folders.description':
    'Files Fresher looked at but does not recognise. Fresher never suggests removing them.',
  'core.storage.part.your-folders.name': 'Your folders',
  'core.storage.place.trash.description': 'You get this space back when you empty the Trash.',
  'core.storage.place.trash.name': 'Trash',
  'core.storage.purgeable': 'macOS can free about {size} more on its own when space runs low',
  'core.storage.reason.changed-during-scan': 'Files changed on the disk while the scan ran.',
  'core.storage.reason.snapshots-and-bookkeeping':
    "Local snapshots and the file system's own records, which macOS looks after.",
  'core.storage.reason.unreadable': 'Folders Fresher could not read. Allowing access lets the next scan measure them.',
  'core.storage.remainder.not-measured.description':
    'Used space the scan could not measure. The reasons are listed below.',
  'core.storage.remainder.not-measured.name': 'Not measured',
  'core.storage.remainder.outside.description':
    "Apps, the system's data and other users' files, which the scan did not look at. Add the disk in Settings → Scan " +
    'to see inside.',
  'core.storage.remainder.outside.name': 'Outside the scanned folders',
};

// The parts every phase of the demo volume shares: the system's volumes, the disk's own space and Free.
const DEMO_VOLUME_PARTS: readonly MapPart[] = [
  part({ kind: 'mac-os' }, 23),
  part({ kind: 'swap-and-sleep' }, 2),
  part({ kind: 'disk-own' }, 1.6),
  part({ kind: 'free' }, 109),
];

const demoVolume = (phase: StorageMap['phase'], scanned: readonly MapPart[]): StorageMap => ({
  phase,
  volumes: [
    {
      capacity: 499_600_000_000,
      diskChanged: false,
      kind: 'internal',
      name: 'Macintosh HD',
      parts: [...scanned, ...DEMO_VOLUME_PARTS],
      places: [],
      purgeable: 18 * GB,
      roots: phase.kind === 'complete' ? [{ index: 0, state: phase.state }] : [],
    },
  ],
});

const UNREADABLE: PartKind = {
  kind: 'remainder',
  outsideScan: true,
  reasons: [{ folders: 3, kind: 'unreadable', needs: { kind: 'all-files' } }, { kind: 'snapshots-and-bookkeeping' }],
};

// The demo scan's volume before a scan: its used space not scanned yet.
export const DEMO_MAP_NO_SCAN: StorageMap = demoVolume({ kind: 'no-scan' }, [part({ kind: 'not-scanned-yet' }, 364)]);

// The demo scan's volume half way: the first batch explained (the demo scan's 2.25 GB).
export const DEMO_MAP_SCANNING: StorageMap = demoVolume({ kind: 'scanning' }, [
  part({ kind: 'explained' }, 2.25, 2.25),
  part({ kind: 'not-scanned-yet' }, 361.75),
]);

// The demo scan's volume after the scan: its 2.57 GB explained, the rest your folders and outside the scan.
export const DEMO_MAP_COMPLETE: StorageMap = demoVolume({ kind: 'complete', state: 1 }, [
  part({ kind: 'explained' }, 2.57, 2.57),
  part({ kind: 'your-folders' }, 241.43),
  part(UNREADABLE, 120),
]);

const sizesOf = (gigabytes: number): MapPart['sizes'] => part({ kind: 'free' }, gigabytes).sizes;

// The by-meaning groups of the complete demo volume: its two categories, the Trash and two of your folders, then the
// volume's lines.
export const DEMO_MEANING: readonly MeaningGroup[] = [
  { kind: 'category', value: { category: 'demo.caches', sizes: sizesOf(1.82) } },
  { kind: 'category', value: { category: 'demo.downloads', sizes: sizesOf(0.75) } },
  { kind: 'place', value: { name: { kind: 'trash' }, node: { index: 7, state: 1 }, sizes: sizesOf(4.2) } },
  {
    kind: 'place',
    value: { name: { kind: 'folder', value: 'Movies' }, node: { index: 8, state: 1 }, sizes: sizesOf(120) },
  },
  {
    kind: 'place',
    value: { name: { kind: 'folder', value: 'Documents' }, node: { index: 9, state: 1 }, sizes: sizesOf(64) },
  },
  { kind: 'part', value: part(UNREADABLE, 120) },
  ...DEMO_VOLUME_PARTS.map((volumePart): MeaningGroup => ({ kind: 'part', value: volumePart })),
];

// The by-meaning groups of the demo volume after a scan that counted `totals`: a group per category, with its counted
// size, then the demo volume's places and lines.
export const meaningFor = (totals: readonly CategoryTotal[]): readonly MeaningGroup[] => [
  ...totals.map(({ category, sizes }): MeaningGroup => ({ kind: 'category', value: { category, sizes } })),
  ...DEMO_MEANING.filter(({ kind }) => kind !== 'category'),
];

// The by-meaning groups while the demo scan runs: the volume's lines only.
export const DEMO_MEANING_SCANNING: readonly MeaningGroup[] = DEMO_VOLUME_PARTS.map((volumePart): MeaningGroup => ({
  kind: 'part',
  value: volumePart,
}));
