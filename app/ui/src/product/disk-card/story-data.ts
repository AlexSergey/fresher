import type { JSX } from 'react';

import type { MapPart, VolumeMap } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { STORAGE_DICTIONARY, withSummary } from '../../test/storage-map';

// The data of the disk card's stories and tests: the design's volume (3.8 GB can be freed, 109 GB free of 494 GB).

const GB = 1_000_000_000;

const part = (kind: MapPart['kind'], gigabytes: number, reclaimable = 0): MapPart => ({
  kind,
  sizes: {
    allocated: Math.round(gigabytes * GB),
    completeness: 'complete',
    logical: Math.round(gigabytes * GB),
    reclaimable: Math.round(reclaimable * GB),
  },
});

const volumeOf = (parts: readonly MapPart[], purgeable: null | number = 18 * GB): VolumeMap =>
  withSummary({
    capacity: 494 * GB,
    diskChanged: false,
    kind: 'internal',
    name: 'Macintosh HD',
    parts: [...parts],
    places: [],
    purgeable,
    roots: [{ index: 0, state: 1 }],
  });

const PARTS: readonly MapPart[] = [
  part({ kind: 'explained' }, 3.8, 3.8),
  part({ kind: 'your-folders' }, 331),
  part({ kind: 'remainder', outsideScan: true, reasons: [] }, 22),
  part({ kind: 'mac-os' }, 23),
  part({ kind: 'swap-and-sleep' }, 2),
  part({ kind: 'disk-own' }, 3.2),
  part({ kind: 'free' }, 109),
];

// A volume after a complete scan.
export const DISK_VOLUME: VolumeMap = volumeOf(PARTS);

// The same volume with nothing to free and no purgeable note.
export const DISK_VOLUME_NOTHING: VolumeMap = volumeOf(
  PARTS.map((each) => (each.kind.kind === 'explained' ? part({ kind: 'explained' }, 0) : each)),
  null,
);

// A volume whose can-be-freed part is far too small to see on the ring.
export const DISK_VOLUME_TINY: VolumeMap = volumeOf(
  PARTS.map((each) => (each.kind.kind === 'explained' ? part({ kind: 'explained' }, 0.004, 0.004) : each)),
);

// A volume whose disk changed during the scan.
export const DISK_VOLUME_CHANGED: VolumeMap = { ...DISK_VOLUME, diskChanged: true };

// A story's render with the core's texts loaded first.
export const withTexts =
  (render: () => JSX.Element): (() => JSX.Element) =>
  () => {
    setCoreDictionary({
      ...STORAGE_DICTIONARY,
      'demo.error.failed': 'The demo module could not build its example findings.',
    });

    return render();
  };
