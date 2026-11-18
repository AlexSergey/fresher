import type { JSX } from 'react';

import type { MapPart, PartKind, VolumeMap } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { SAMPLE_STORAGE_MAP, STORAGE_DICTIONARY } from '../../test/storage-map';
import { STORY_DICTIONARY } from '../scan-hero/story-data';

// The volumes of the storage map's stories, the meaning list's, and the core's texts they need.

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

const SYSTEM_PARTS: readonly MapPart[] = [
  part({ kind: 'mac-os' }, 23),
  part({ kind: 'swap-and-sleep' }, 2),
  part({ kind: 'disk-own' }, 1.6),
];

const volume = (parts: readonly MapPart[], overrides: Partial<VolumeMap> = {}): VolumeMap => ({
  capacity: parts.reduce((sum, { sizes }) => sum + sizes.allocated, 0),
  diskChanged: false,
  kind: 'internal',
  name: 'Macintosh HD',
  parts: [...parts],
  places: [],
  purgeable: 18 * GB,
  roots: [],
  ...overrides,
});

// After a complete scan of the home folder: the gallery's sample volume.
export const COMPLETE = SAMPLE_STORAGE_MAP.volumes[0] as VolumeMap;

// Before any scan: the used space not scanned yet.
export const NO_SCAN = volume([part({ kind: 'not-scanned-yet' }, 364), ...SYSTEM_PARTS, part({ kind: 'free' }, 109)]);

// Half way through a scan.
export const SCANNING = volume([
  part({ kind: 'explained' }, 21, 19),
  part({ kind: 'not-scanned-yet' }, 343),
  ...SYSTEM_PARTS,
  part({ kind: 'free' }, 109),
]);

// The scan did not cover the volume: what it did not look at is outside the scanned folders, with its reasons.
export const OUTSIDE_SCAN = volume([
  part({ kind: 'explained' }, 64, 58),
  part({ kind: 'your-folders' }, 180),
  part(
    {
      kind: 'remainder',
      outsideScan: true,
      reasons: [
        { folders: 3, kind: 'unreadable', needs: { kind: 'all-files' } },
        { kind: 'snapshots-and-bookkeeping' },
      ],
    },
    120,
  ),
  ...SYSTEM_PARTS,
  part({ kind: 'free' }, 109),
]);

// The scan covered the whole volume; what it could not measure, and the disk changed while it ran.
export const NOT_MEASURED = volume(
  [
    part({ kind: 'explained' }, 64, 58),
    {
      ...part({ kind: 'your-folders' }, 290),
      sizes: { ...part({ kind: 'your-folders' }, 290).sizes, completeness: 'partial' },
    },
    part(
      {
        kind: 'remainder',
        outsideScan: false,
        reasons: [{ kind: 'snapshots-and-bookkeeping' }, { kind: 'changed-during-scan' }],
      },
      10,
    ),
    ...SYSTEM_PARTS,
    part({ kind: 'free' }, 109),
  ],
  { diskChanged: true },
);

// An engine without volume facts: only the scanned parts, drawn over their own sum, no purgeable note.
export const WITHOUT_VOLUME_FACTS = volume([part({ kind: 'explained' }, 64, 58), part({ kind: 'your-folders' }, 180)], {
  purgeable: null,
});

// A second volume of the user's, on an external disk.
export const EXTERNAL = volume(
  [
    part({ kind: 'explained' }, 4, 4),
    part({ kind: 'your-folders' }, 610),
    part({ kind: 'other-volume', name: 'Backups' }, 220),
    part({ kind: 'free' }, 166),
  ],
  { kind: 'external', name: 'Archive', purgeable: null },
);

// A story's render with the core's texts loaded first, the storage map's among them.
export const withStorageTexts =
  (render: () => JSX.Element): (() => JSX.Element) =>
  () => {
    setCoreDictionary({ ...STORY_DICTIONARY, ...STORAGE_DICTIONARY });

    return render();
  };
