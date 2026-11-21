import type { JSX } from 'react';

import type { CategoryTotal, DangerLevel, Dictionary, Overview, Sizes, VolumeMap } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { DEMO_OVERVIEW, FULL_COVERAGE } from '../../test/demo-scan';
import { largeScan } from '../../test/large-scan';
import { SAMPLE_STORAGE_MAP, STORAGE_DICTIONARY } from '../../test/storage-map';

// The data of the product components' stories and tests: the demo scan's overview, a generated scan of twelve
// categories, and the core's texts they need.

export const TWELVE_CATEGORIES = largeScan({ categories: 12, count: 48 });

const whatOfLargeGroups = Object.fromEntries(
  TWELVE_CATEGORIES.overview.totals.map(({ category }) => [
    `${category}.what`,
    'Example data: a made-up group of findings, generated for the gallery.',
  ]),
);

export const STORY_DICTIONARY: Dictionary = {
  ...TWELVE_CATEGORIES.dictionary,
  ...STORAGE_DICTIONARY,
  ...whatOfLargeGroups,
  'core.completeness.partial': 'Some files could not be read, so the real size is a little bigger.',
  'core.group.debug.name': 'Debug',
  'core.group.development.name': 'Development',
  'core.group.system.name': 'System',
  'core.ignore.modules.description': 'Hides everything this module finds and never cleans it.',
  'core.left-out.summary.one': '{count} item left out ({size}):',
  'core.left-out.summary.other': '{count} items left out ({size}):',
  'demo.caches.what': 'Example data: made-up cache folders of made-up apps, shown by the demo module.',
  'demo.downloads.what': 'Example data: made-up installers and archives in a made-up Downloads folder.',
  'demo.enabled.label': 'Demo module',
  'demo.error.failed': 'The demo module could not build its example findings.',
  'demo.images.name': 'Example images',
  'demo.images.what': 'Example data: made-up images of a made-up tool, removed by a command of that tool.',
  'demo.left-out.open': 'The app that made it is open. Quit the app, then scan again.',
  'demo.volumes.name': 'Example volumes',
  'demo.volumes.what': 'Example data: made-up data volumes of a made-up tool; real ones can hold data you need.',
  'docker.enabled.label': 'Docker',
  'docker.error.unreachable': 'Docker could not be reached.',
  'logs.enabled.label': 'Logs',
  'node.enabled.label': 'Node modules',
  'xcode.enabled.label': 'Xcode',
};

// A story's render with the core's texts loaded first: every story sets them, so the order the gallery loads its
// story files in does not matter.
export const withTexts =
  (render: () => JSX.Element): (() => JSX.Element) =>
  () => {
    setCoreDictionary(STORY_DICTIONARY);

    return render();
  };

const partial = (sizes: Sizes): Sizes => ({ ...sizes, completeness: 'partial' });

// The demo scan's overview, finished.
export const DEMO_FINISHED: Overview = DEMO_OVERVIEW;

// The demo scan's overview when some folders could not be read: the totals are lower bounds.
export const DEMO_PARTIAL: Overview = {
  ...DEMO_OVERVIEW,
  found: partial(DEMO_OVERVIEW.found),
  hiddenModules: [],
  ignored: { categories: 0, groups: 0, modules: 0, paths: 0 },
  lastScan: {
    coverage: FULL_COVERAGE,
    finished: 1_735_689_600,
    kind: { kind: 'full' },
    outcome: { kind: 'partial' },
    scan: 1,
    settingsChanged: false,
  },
  totals: DEMO_OVERVIEW.totals.map((total, index) => (index === 0 ? { ...total, sizes: partial(total.sizes) } : total)),
};

const complete = (bytes: number): Sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

// The demo scan's overview half way: its first batch counted.
export const DEMO_RUNNING: Overview = {
  found: complete(2_250_000_000),
  hiddenModules: [],
  ignored: { categories: 0, groups: 0, modules: 0, paths: 0 },
  lastScan: null,
  status: { scan: 1, state: 'scanning' },
  totals: [
    { category: 'demo.caches', count: 1, highestDanger: 'safe', sizes: complete(1_500_000_000) },
    { category: 'demo.downloads', count: 1, highestDanger: 'low', sizes: complete(750_000_000) },
  ],
};

// A finished scan that found nothing.
export const NOTHING_FOUND: Overview = {
  ...DEMO_OVERVIEW,
  found: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 },
  totals: [],
};

// Every category of the demo module, one per danger level, largest first: the cards' and the grid's stories.
export const DEMO_TOTALS: readonly CategoryTotal[] = [
  { category: 'demo.caches', count: 2, highestDanger: 'safe', sizes: complete(1_820_000_000) },
  { category: 'demo.downloads', count: 1, highestDanger: 'low', sizes: complete(750_000_000) },
  { category: 'demo.images', count: 4, highestDanger: 'medium', sizes: complete(420_000_000) },
  { category: 'demo.volumes', count: 1, highestDanger: 'high', sizes: complete(96_000_000) },
];

// Everything the demo totals hold.
export const DEMO_TOTALS_FOUND: Sizes = complete(DEMO_TOTALS.reduce((sum, { sizes }) => sum + sizes.allocated, 0));

const GB = 1_000_000_000;

// The home volume the hero draws after a complete scan: the gallery's sample volume, 64 GB of it explained.
export const HERO_VOLUME: VolumeMap = SAMPLE_STORAGE_MAP.volumes[0] as VolumeMap;

// The sample volume's cleanable 64 GB by danger level, as the core's shares would sum.
export const HERO_LEVELS: ReadonlyMap<DangerLevel, Sizes> = new Map([
  ['high', complete(2 * GB)],
  ['low', complete(14 * GB)],
  ['medium', complete(8 * GB)],
  ['safe', complete(40 * GB)],
]);

const HERO_GB: readonly number[] = [40, 14, 8, 2];

// The overview that goes with the levels above: the same 64 GB found, in one category per level.
export const HERO_OVERVIEW: Overview = {
  ...DEMO_FINISHED,
  found: complete(64 * GB),
  totals: DEMO_TOTALS.map((total, index) => ({ ...total, sizes: complete((HERO_GB[index] ?? 0) * GB) })),
};

// A volume with a disk that changed during the scan.
export const HERO_VOLUME_CHANGED: VolumeMap = { ...HERO_VOLUME, diskChanged: true };

// What a scan that found nothing has by level.
export const NO_LEVELS: ReadonlyMap<DangerLevel, Sizes> = new Map();
