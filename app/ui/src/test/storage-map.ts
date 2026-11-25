import type {
  CategoryTotal,
  MapPart,
  MeaningGroup,
  PartKind,
  Sizes,
  StorageMap,
  SummaryKind,
  SummaryPart,
  VolumeMap,
} from '../ipc/bindings.generated';

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

// The summary part each kind of map part belongs to. Test code only: the rule lives in
// crates/core/src/storage/summary.rs (`SummaryKind::of`), which the fixtures mirror.
const SUMMARY_OF: Readonly<Record<PartKind['kind'], SummaryKind>> = {
  'disk-own': 'system-space',
  explained: 'can-be-freed',
  free: 'free',
  'mac-os': 'mac-os',
  'not-scanned-yet': 'files-and-apps',
  'other-volume': 'files-and-apps',
  remainder: 'files-and-apps',
  'swap-and-sleep': 'system-space',
  'your-folders': 'files-and-apps',
};

export const SUMMARY_KINDS: readonly SummaryKind[] = [
  'can-be-freed',
  'files-and-apps',
  'mac-os',
  'system-space',
  'free',
];

const COMPLETENESS: readonly Sizes['completeness'][] = ['complete', 'partial', 'unknown'];

const addSizes = (a: Sizes, b: Sizes): Sizes => ({
  allocated: a.allocated + b.allocated,
  completeness:
    COMPLETENESS[Math.max(COMPLETENESS.indexOf(a.completeness), COMPLETENESS.indexOf(b.completeness))] ?? 'unknown',
  logical: a.logical + b.logical,
  reclaimable: a.reclaimable + b.reclaimable,
});

const ZERO: Sizes = { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 };

// The volume with its five summary parts filled from its parts, as the core's `VolumeMap::new` fills them.
export const withSummary = (volume: Omit<VolumeMap, 'summary'>): VolumeMap => ({
  ...volume,
  summary: SUMMARY_KINDS.map((kind): SummaryPart => ({
    kind,
    sizes: volume.parts
      .filter((mapPart) => SUMMARY_OF[mapPart.kind.kind] === kind)
      .reduce((sum, mapPart) => addSizes(sum, mapPart.sizes), ZERO),
  })),
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
    withSummary({
      capacity: 499_600_000_000,
      diskChanged: false,
      kind: 'internal',
      name: 'Macintosh HD',
      parts: [...SAMPLE_PARTS],
      places: [],
      purgeable: 18 * GB,
      roots: [{ index: 0, state: 1 }],
    }),
  ],
};

export const SAMPLE_MEANING: readonly MeaningGroup[] = SAMPLE_PARTS.map((sample) => ({
  kind: 'part',
  value: sample,
}));

// The core's texts of the storage map (crates/core/locales/en.json, `core.storage.*`), for the tests and the stories.
export const STORAGE_DICTIONARY: Readonly<Record<string, string>> = {
  'core.danger.high.meaning': 'May hold the only copy of your data',
  'core.danger.high.zone': 'Only if you are sure',
  'core.danger.low.meaning': 'You may need to download them again',
  'core.danger.low.zone': 'Usually safe',
  'core.danger.medium.meaning': 'May hold something you still want',
  'core.danger.medium.zone': 'Look before removing',
  'core.danger.safe.meaning': 'Apps make these again on their own',
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
    'Space the disk keeps for itself, such as the recovery system. It cannot be cleaned.',
  'core.storage.part.disk-own.name': "The disk's own space",
  'core.storage.part.explained.description':
    'What Fresher found that you can remove, such as caches, logs and old builds.',
  'core.storage.part.explained.name': 'Can be freed',
  'core.storage.part.free.description': 'Space you can still use for new files.',
  'core.storage.part.free.name': 'Free',
  'core.storage.part.mac-os.description': 'The system itself. macOS looks after it, and it cannot be cleaned.',
  'core.storage.part.mac-os.name': 'macOS',
  'core.storage.part.not-scanned-yet.description':
    'Space the scan has not reached yet. It shrinks as the scan goes on.',
  'core.storage.part.not-scanned-yet.name': 'Not scanned yet',
  'core.storage.part.other-volume.description':
    'Another part of the same disk. Add it in Settings → Scan to look inside.',
  'core.storage.part.other-volume.name': 'Another part of the disk',
  'core.storage.part.swap-and-sleep.description':
    'Memory macOS keeps on the disk when it runs low or the Mac sleeps. Restarting the Mac can make it smaller.',
  'core.storage.part.swap-and-sleep.name': 'Memory kept on disk',
  'core.storage.part.your-folders.description': 'Files Fresher looked at but does not offer to remove.',
  'core.storage.part.your-folders.name': 'Your files and apps',
  'core.storage.place.trash.description': 'You get this space back when you empty the Trash.',
  'core.storage.place.trash.name': 'Trash',
  'core.storage.purgeable': 'macOS can free about {size} more on its own when space runs low',
  'core.storage.reason.changed-during-scan': 'Files changed on the disk while the scan ran.',
  'core.storage.reason.snapshots-and-bookkeeping': "Copies macOS keeps for backups, and the disk's own records.",
  'core.storage.reason.unreadable': 'Folders Fresher could not read. Allowing access lets the next scan measure them.',
  'core.storage.remainder.not-measured.description':
    'Used space the scan could not measure. The reasons are listed below.',
  'core.storage.remainder.not-measured.name': 'Not measured',
  'core.storage.remainder.outside.description':
    "Apps, the system's data and other users' files, which the scan did not look at. Add the disk in Settings → Scan " +
    'to see inside.',
  'core.storage.remainder.outside.name': 'Outside the scanned folders',
  'core.storage.summary.can-be-freed.description':
    'What Fresher found that you can remove, sorted below by what removing it costs.',
  'core.storage.summary.can-be-freed.name': 'Can be freed',
  'core.storage.summary.files-and-apps.description':
    'Your documents, apps and everything else Fresher does not offer to remove.',
  'core.storage.summary.files-and-apps.name': 'Your files and apps',
  'core.storage.summary.free.description': 'Space you can still use for new files.',
  'core.storage.summary.free.name': 'Free',
  'core.storage.summary.mac-os.description': 'The system itself, which macOS looks after; Fresher never touches it.',
  'core.storage.summary.mac-os.name': 'macOS',
  'core.storage.summary.system-space.description': 'Memory macOS keeps on disk, and space the disk keeps for itself.',
  'core.storage.summary.system-space.name': 'System space',
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
    withSummary({
      capacity: 499_600_000_000,
      diskChanged: false,
      kind: 'internal',
      name: 'Macintosh HD',
      parts: [...scanned, ...DEMO_VOLUME_PARTS],
      places: [],
      purgeable: 18 * GB,
      roots: phase.kind === 'complete' ? [{ index: 0, state: phase.state }] : [],
    }),
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
