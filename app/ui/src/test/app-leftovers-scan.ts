import type {
  Dictionary,
  Event,
  Finding,
  LeftOutCount,
  MeaningGroup,
  ModuleInfo,
  Overview,
  SettingsView,
  Sizes,
  TextArg,
} from '../ipc/bindings.generated';

import appLeftoversTexts from '../../../../modules/app-leftovers/locales/en.json' with { type: 'json' };
import { FULL_COVERAGE, readyCategory, SCRIPTS_DICTIONARY } from './demo-scan';
import { FINDING_DICTIONARY } from './finding-fixtures';

// A scan of the removed apps' leftovers module as the core streams it: `com.example.Gone` is no longer installed and
// left caches, logs, settings, web data, a support folder and a container; another app is open (its files left out as
// `running`), the files of a third changed within the days (`used-recently`). An id is offered or left out, never
// both. The texts are the module's own, read from its locale file, so a changed text changes the tests. Shared by the
// unit tests and the e2e mock.

const MB = 1_000_000;

const sized = (bytes: number): Sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

const text = (value: string): TextArg => ({ kind: 'text', value });

const HOME = '/Users/someone/Library';
const GONE = 'com.example.Gone';

type Spec = {
  readonly bytes: number;
  readonly category: string;
  readonly danger: Finding['danger'];
  readonly folder: string;
  readonly id: number;
  readonly title: string;
};

const findingOf = (spec: Spec): Finding => ({
  app: GONE,
  category: spec.category,
  danger: spec.danger,
  id: spec.id,
  key: { identity: spec.folder, module: 'app-leftovers' },
  module: 'app-leftovers',
  reason: { args: { days: { kind: 'count', value: 30 } }, key: 'app-leftovers.finding.reason' },
  removability: 'removable',
  removal: { kind: 'move-to-trash', value: `${HOME}/${spec.folder}` },
  sizes: sized(spec.bytes),
  subject: { kind: 'path', value: `${HOME}/${spec.folder}` },
  title: { args: { id: text(GONE) }, key: `app-leftovers.title.${spec.title}` },
});

// Safe caches and logs, Medium settings and web data, High support folder and container.
export const APP_LEFTOVERS_FINDINGS: readonly Finding[] = [
  findingOf({
    bytes: 640 * MB,
    category: 'app-leftovers.caches',
    danger: 'safe',
    folder: `Caches/${GONE}`,
    id: 101,
    title: 'caches',
  }),
  findingOf({
    bytes: 24 * MB,
    category: 'app-leftovers.caches',
    danger: 'safe',
    folder: `Logs/${GONE}`,
    id: 102,
    title: 'logs',
  }),
  findingOf({
    bytes: 2 * MB,
    category: 'app-leftovers.settings',
    danger: 'medium',
    folder: `Preferences/${GONE}.plist`,
    id: 103,
    title: 'preferences',
  }),
  findingOf({
    bytes: 18 * MB,
    category: 'app-leftovers.settings',
    danger: 'medium',
    folder: `HTTPStorages/${GONE}`,
    id: 104,
    title: 'http-storage',
  }),
  findingOf({
    bytes: 1_800 * MB,
    category: 'app-leftovers.data',
    danger: 'high',
    folder: `Application Support/${GONE}`,
    id: 105,
    title: 'support',
  }),
  findingOf({
    bytes: 310 * MB,
    category: 'app-leftovers.data',
    danger: 'high',
    folder: `Containers/${GONE}`,
    id: 106,
    title: 'container',
  }),
];

// `get_modules`: the module in the group Apps, its three categories, its two settings, its two choices.
export const APP_LEFTOVERS_MODULES: readonly ModuleInfo[] = [
  {
    categories: [
      readyCategory('app-leftovers.caches', 'safe'),
      readyCategory('app-leftovers.settings', 'medium'),
      readyCategory('app-leftovers.data', 'high'),
    ],
    choices: [
      { categories: ['app-leftovers.caches', 'app-leftovers.settings', 'app-leftovers.data'], name: 'everything' },
      { categories: ['app-leftovers.caches'], name: 'keep-settings' },
    ],
    group: 'apps',
    id: 'app-leftovers',
    settings: ['app-leftovers.enabled', 'app-leftovers.uninstalled-after-days'],
  },
];

const left = (category: string, key: string, bytes: number): LeftOutCount => ({
  category,
  items: 1,
  reason: { args: {}, key: `app-leftovers.left-out.${key}` },
  sizes: sized(bytes),
});

// One app that is open and one whose files changed within the days: each has no finding of its own.
export const APP_LEFTOVERS_LEFT_OUT: readonly LeftOutCount[] = [
  left('app-leftovers.caches', 'running', 220 * MB),
  left('app-leftovers.settings', 'used-recently', 3 * MB),
];

// The events of a scan of `modules` that found `findings`; only this module leaves anything out.
export const scanOf = (
  findings: readonly Finding[],
  modules: readonly string[] = ['app-leftovers'],
): readonly Event[] => [
  { modules: [...modules], scan: 1, type: 'scan-started' },
  { findings: findings.slice(0, 3), scan: 1, type: 'findings-added' },
  { findings: findings.slice(3), scan: 1, type: 'findings-added' },
  ...modules.map((module): Event => ({
    leftOut: module === 'app-leftovers' ? [...APP_LEFTOVERS_LEFT_OUT] : [],
    module,
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  })),
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' },
];

export const APP_LEFTOVERS_SCAN: readonly Event[] = scanOf(APP_LEFTOVERS_FINDINGS);

export const APP_LEFTOVERS_TOTALS: Overview['totals'] = [
  { category: 'app-leftovers.caches', count: 2, highestDanger: 'safe', sizes: sized(664 * MB) },
  { category: 'app-leftovers.settings', count: 2, highestDanger: 'medium', sizes: sized(20 * MB) },
  { category: 'app-leftovers.data', count: 2, highestDanger: 'high', sizes: sized(2_110 * MB) },
];

const overviewOf = (found: number, totals: Overview['totals']): Overview => ({
  found: sized(found),
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
  totals,
});

export const APP_LEFTOVERS_OVERVIEW: Overview = overviewOf(2_794 * MB, APP_LEFTOVERS_TOTALS);

// The by-meaning view: the removed app's line is named by its id and marked as no longer installed.
export const APP_LEFTOVERS_MEANING: readonly MeaningGroup[] = [
  {
    kind: 'app',
    value: {
      app: { id: GONE, installed: false, name: GONE },
      categories: APP_LEFTOVERS_TOTALS.map(({ category, sizes }) => ({ category, sizes })),
      data: [],
      sizes: sized(2_794 * MB),
    },
  },
];

// The module's settings: its switch and the days after which a removed app's files are offered.
export const APP_LEFTOVERS_SETTINGS: SettingsView = {
  file: 'loaded',
  problems: [],
  settings: [
    {
      default: { kind: 'switch', value: true },
      effect: 'next-scan',
      group: 'module',
      key: 'app-leftovers.enabled',
      kind: { kind: 'switch' },
    },
    {
      default: { kind: 'number', value: 30 },
      effect: 'next-scan',
      group: 'module',
      key: 'app-leftovers.uninstalled-after-days',
      kind: { kind: 'number', max: 3650, min: 7, unit: 'days' },
    },
  ],
  values: {
    'app-leftovers.enabled': { kind: 'switch', value: true },
    'app-leftovers.uninstalled-after-days': { kind: 'number', value: 30 },
  },
};

// The module's own texts, with the core's that the page and the confirmation show.
export const APP_LEFTOVERS_DICTIONARY: Dictionary = {
  ...SCRIPTS_DICTIONARY,
  ...FINDING_DICTIONARY,
  ...appLeftoversTexts,
  'core.group.apps.description': 'What apps leave behind, such as the files of apps you removed.',
  'core.group.apps.name': 'Apps',
  'core.ignore.groups.description': 'Hides everything the modules of this group find and never cleans it.',
};
