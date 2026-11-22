import type {
  Dictionary,
  Event,
  Finding,
  LeftOutCount,
  ModuleInfo,
  Overview,
  SettingsView,
  Sizes,
  TextArg,
} from '../ipc/bindings.generated';

import coreTexts from '../../../../crates/core/locales/en.json' with { type: 'json' };
import appCachesTexts from '../../../../modules/app-caches/locales/en.json' with { type: 'json' };
import { FULL_COVERAGE, readyCategory, SCRIPTS_DICTIONARY } from './demo-scan';
import { FINDING_DICTIONARY } from './finding-fixtures';

// A scan of the installed app caches module as the core streams it: the caches of two closed apps are offered (the
// folder of Editor, one of its helper's, one of Player's updater, each titled by its app), and the cache of a third
// app that is open is left out. A folder is offered or left out, never both. The texts are the module's own, read from
// its locale file, so a changed text changes the tests. Played by the e2e mock.

const MB = 1_000_000;

export const sized = (bytes: number): Sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

export const text = (value: string): TextArg => ({ kind: 'text', value });

const CACHES = '/Users/someone/Library/Caches';

export const APP_CACHES = 'app-caches.library-caches';

type CacheFinding = {
  readonly app: string;
  readonly bytes: number;
  readonly category: string;
  readonly id: number;
  readonly identity: string;
  readonly name: string;
  readonly path: string;
  readonly title: Finding['title'];
};

type Spec = {
  readonly app: string;
  readonly bytes: number;
  readonly folder: string;
  readonly id: number;
  readonly name: string;
};

// The module's finding of a cache folder: Low, reasoned by the app and the size, it empties the folder.
export const cacheFinding = (spec: CacheFinding): Finding => ({
  app: spec.app,
  category: spec.category,
  danger: 'low',
  id: spec.id,
  key: { identity: spec.identity, module: 'app-caches' },
  module: 'app-caches',
  reason: {
    args: { app: text(spec.name), size: { kind: 'size', value: spec.bytes } },
    key: 'app-caches.reason',
  },
  removability: 'removable',
  removal: { kind: 'empty-folder', value: spec.path },
  sizes: sized(spec.bytes),
  subject: { kind: 'path', value: spec.path },
  title: spec.title,
});

// A folder of Library's Caches: the one of the app's own id is titled by the app, any other by the app and the folder.
const findingOf = (spec: Spec): Finding =>
  cacheFinding({
    app: spec.app,
    bytes: spec.bytes,
    category: APP_CACHES,
    id: spec.id,
    identity: `library-caches/${spec.folder}`,
    name: spec.name,
    path: `${CACHES}/${spec.folder}`,
    title:
      spec.folder === spec.app
        ? { args: { app: text(spec.name) }, key: 'app-caches.title.own' }
        : { args: { app: text(spec.name), folder: text(spec.folder) }, key: 'app-caches.title.part' },
  });

export const APP_CACHES_FINDINGS: readonly Finding[] = [
  findingOf({ app: 'com.example.Editor', bytes: 920 * MB, folder: 'com.example.Editor', id: 701, name: 'Editor' }),
  findingOf({
    app: 'com.example.Editor',
    bytes: 156 * MB,
    folder: 'com.example.Editor.Helper',
    id: 702,
    name: 'Editor',
  }),
  findingOf({
    app: 'com.example.Player',
    bytes: 64 * MB,
    folder: 'com.example.Player.ShipIt',
    id: 703,
    name: 'Player',
  }),
];

// `get_modules`: the module in the group System, its three categories, the core's generic switch, no choice.
export const APP_CACHES_MODULES: readonly ModuleInfo[] = [
  {
    categories: [
      readyCategory(APP_CACHES, 'low'),
      readyCategory('app-caches.browsers', 'low'),
      readyCategory('app-caches.web-engine', 'low'),
    ],
    choices: [],
    group: 'system',
    id: 'app-caches',
    settings: ['app-caches.enabled'],
  },
];

// The cache of another app, which is open: kept.
export const APP_CACHES_LEFT_OUT: readonly LeftOutCount[] = [
  {
    category: APP_CACHES,
    items: 1,
    reason: { args: {}, key: 'app-caches.left-out.app-open' },
    sizes: sized(90 * MB),
  },
];

export const APP_CACHES_SCAN: readonly Event[] = [
  { modules: ['app-caches'], scan: 1, type: 'scan-started' },
  { findings: APP_CACHES_FINDINGS.slice(0, 2), scan: 1, type: 'findings-added' },
  { findings: APP_CACHES_FINDINGS.slice(2), scan: 1, type: 'findings-added' },
  {
    leftOut: [...APP_CACHES_LEFT_OUT],
    module: 'app-caches',
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' },
];

export const APP_CACHES_TOTALS: Overview['totals'] = [
  { category: APP_CACHES, count: 3, highestDanger: 'low', sizes: sized(1_140 * MB) },
];

export const APP_CACHES_OVERVIEW: Overview = {
  found: sized(1_140 * MB),
  hiddenModules: [],
  ignored: { categories: 0, groups: 0, modules: 0, paths: 0 },
  lastScan: {
    coverage: FULL_COVERAGE,
    finished: 1_735_689_600,
    kind: { kind: 'full' },
    outcome: { kind: 'completed' },
    scan: 1,
    settingsChanged: false,
  },
  status: { state: 'idle' },
  totals: APP_CACHES_TOTALS,
};

// The module's settings: the core's generic switch, on by default.
export const APP_CACHES_SETTINGS: SettingsView = {
  file: 'loaded',
  problems: [],
  settings: [
    {
      default: { kind: 'switch', value: true },
      effect: 'next-scan',
      group: 'module',
      key: 'app-caches.enabled',
      kind: { kind: 'switch' },
    },
  ],
  values: { 'app-caches.enabled': { kind: 'switch', value: true } },
};

// The core's own texts the page shows beside the module's: its group.
const coreOwn: Dictionary = Object.fromEntries(
  Object.entries(coreTexts).filter(([key]) => key.startsWith('core.group.system.')),
);

export const APP_CACHES_DICTIONARY: Dictionary = {
  ...SCRIPTS_DICTIONARY,
  ...FINDING_DICTIONARY,
  ...appCachesTexts,
  ...coreOwn,
};
