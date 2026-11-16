import type { JSX } from 'react';

import type { CategoryTotal, Dictionary, Overview, Sizes } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { DEMO_OVERVIEW } from '../../test/demo-scan';
import { largeScan } from '../../test/large-scan';

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
  ...whatOfLargeGroups,
  'core.completeness.partial': 'Some files could not be read, so the real size is a little bigger.',
  'demo.caches.what': 'Example data: made-up cache folders of made-up apps, shown by the demo module.',
  'demo.downloads.what': 'Example data: made-up installers and archives in a made-up Downloads folder.',
  'demo.enabled.label': 'Demo module',
  'demo.error.failed': 'The demo module could not build its example findings.',
  'demo.images.name': 'Example images',
  'demo.images.what': 'Example data: made-up images of a made-up tool, removed by a command of that tool.',
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

const partial = ({ allocated, logical }: Sizes): Sizes => ({ allocated, completeness: 'partial', logical });

// The demo scan's overview, finished.
export const DEMO_FINISHED: Overview = DEMO_OVERVIEW;

// The demo scan's overview when some folders could not be read: the totals are lower bounds.
export const DEMO_PARTIAL: Overview = {
  ...DEMO_OVERVIEW,
  found: partial(DEMO_OVERVIEW.found),
  lastScan: { finished: 1_735_689_600, outcome: { kind: 'partial' }, scan: 1 },
  totals: DEMO_OVERVIEW.totals.map((total, index) => (index === 0 ? { ...total, sizes: partial(total.sizes) } : total)),
};

const complete = (bytes: number): Sizes => ({ allocated: bytes, completeness: 'complete', logical: bytes });

// The demo scan's overview half way: its first batch counted.
export const DEMO_RUNNING: Overview = {
  found: complete(2_250_000_000),
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
  found: { allocated: 0, completeness: 'complete', logical: 0 },
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
