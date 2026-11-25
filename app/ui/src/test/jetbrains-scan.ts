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

import jetbrainsTexts from '../../../../modules/jetbrains/locales/en.json' with { type: 'json' };
import { FULL_COVERAGE, readyCategory, SCRIPTS_DICTIONARY } from './demo-scan';
import { FINDING_DICTIONARY } from './finding-fixtures';

// A scan of the JetBrains module as the core streams it: WebStorm 2024.3 left its settings, caches and logs behind
// (you use 2025.1 now), DataGrip is no longer installed, and the texts are the module's own, read from its locale
// file, so a changed text changes the tests. Shared by the unit tests and the e2e mock.

const MB = 1_000_000;

const sized = (bytes: number): Sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

const text = (value: string): TextArg => ({ kind: 'text', value });

const HOME = '/Users/someone/Library';

type Spec = {
  readonly app: string;
  readonly bytes: number;
  readonly category: string;
  readonly folder: string;
  readonly id: number;
  readonly product: string;
  readonly uninstalled: 'caches' | 'logs' | 'settings' | null;
  readonly version: string;
};

const findingOf = (spec: Spec): Finding => {
  const path = `${HOME}/${spec.folder}`;
  const removed = spec.uninstalled !== null;

  return {
    app: spec.app,
    category: spec.category,
    danger: spec.category === 'jetbrains.caches' || spec.category === 'jetbrains.logs' ? 'safe' : 'medium',
    id: spec.id,
    key: { identity: spec.folder, module: 'jetbrains' },
    module: 'jetbrains',
    reason: removed
      ? {
          args: { days: { kind: 'count', value: 30 }, product: text(spec.product), version: text(spec.version) },
          key: 'jetbrains.finding.uninstalled-reason',
        }
      : {
          args: { current: text(`${spec.product} 2025.1`), product: text(spec.product), version: text(spec.version) },
          key: 'jetbrains.finding.reason',
        },
    removability: 'removable',
    removal: { kind: 'move-to-trash', value: path },
    sizes: sized(spec.bytes),
    subject: { kind: 'path', value: path },
    title: {
      args: { product: text(spec.product), version: text(spec.version) },
      key: removed ? `jetbrains.finding.uninstalled-${spec.uninstalled}` : 'jetbrains.finding.title',
    },
  };
};

const WEBSTORM = { app: 'com.jetbrains.WebStorm', product: 'WebStorm', uninstalled: null, version: '2024.3' } as const;
const DATAGRIP = { app: 'com.jetbrains.datagrip', product: 'DataGrip', version: '2024.3' } as const;

export const JETBRAINS_FINDINGS: readonly Finding[] = [
  findingOf({
    ...WEBSTORM,
    bytes: 1_200 * MB,
    category: 'jetbrains.caches',
    folder: 'Caches/JetBrains/WebStorm2024.3',
    id: 1,
  }),
  findingOf({
    ...WEBSTORM,
    bytes: 40 * MB,
    category: 'jetbrains.logs',
    folder: 'Logs/JetBrains/WebStorm2024.3',
    id: 2,
  }),
  findingOf({
    ...WEBSTORM,
    bytes: 85 * MB,
    category: 'jetbrains.settings',
    folder: 'Application Support/JetBrains/WebStorm2024.3',
    id: 3,
  }),
  findingOf({
    ...DATAGRIP,
    bytes: 300 * MB,
    category: 'jetbrains.uninstalled',
    folder: 'Caches/JetBrains/DataGrip2024.3',
    id: 4,
    uninstalled: 'caches',
  }),
  findingOf({
    ...DATAGRIP,
    bytes: 120 * MB,
    category: 'jetbrains.uninstalled',
    folder: 'Application Support/JetBrains/DataGrip2024.3',
    id: 5,
    uninstalled: 'settings',
  }),
];

// `get_modules`: the module in the group Development, its four categories and its two settings.
export const JETBRAINS_MODULES: readonly ModuleInfo[] = [
  {
    categories: [
      readyCategory('jetbrains.caches', 'safe'),
      readyCategory('jetbrains.logs', 'safe'),
      readyCategory('jetbrains.settings', 'medium'),
      readyCategory('jetbrains.uninstalled', 'medium'),
    ],
    choices: [],
    group: 'development',
    id: 'jetbrains',
    settings: ['jetbrains.enabled', 'jetbrains.uninstalled-after-days'],
  },
];

const left = (category: string, key: string, items: number, bytes: number): LeftOutCount => ({
  category,
  items,
  reason: { args: {}, key: `jetbrains.left-out.${key}` },
  sizes: sized(bytes),
});

// What the module recognised and left out: the version you use (three folders), and two folders of an IDE that is open.
export const JETBRAINS_LEFT_OUT: readonly LeftOutCount[] = [
  left('jetbrains.caches', 'current', 3, 1_400 * MB),
  left('jetbrains.settings', 'running', 2, 600 * MB),
];

export const JETBRAINS_SCAN: readonly Event[] = [
  { modules: ['jetbrains'], scan: 1, type: 'scan-started' },
  { findings: JETBRAINS_FINDINGS.slice(0, 3), scan: 1, type: 'findings-added' },
  { findings: JETBRAINS_FINDINGS.slice(3), scan: 1, type: 'findings-added' },
  {
    insights: [],
    leftOut: [...JETBRAINS_LEFT_OUT],
    module: 'jetbrains',
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' },
];

export const JETBRAINS_OVERVIEW: Overview = {
  found: sized(1_745 * MB),
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
  totals: [
    { category: 'jetbrains.caches', count: 1, highestDanger: 'safe', sizes: sized(1_200 * MB) },
    { category: 'jetbrains.logs', count: 1, highestDanger: 'safe', sizes: sized(40 * MB) },
    { category: 'jetbrains.settings', count: 1, highestDanger: 'medium', sizes: sized(85 * MB) },
    { category: 'jetbrains.uninstalled', count: 2, highestDanger: 'medium', sizes: sized(420 * MB) },
  ],
};

// The by-meaning view: WebStorm's line holds the findings of the version you left (its caches, logs and settings);
// the folders of the version in use are not in it, they stay in Your folders. DataGrip, removed, is its own line.
export const JETBRAINS_MEANING: readonly MeaningGroup[] = [
  {
    kind: 'app',
    value: {
      app: { id: 'com.jetbrains.WebStorm', installed: true, name: 'WebStorm' },
      categories: [
        { category: 'jetbrains.caches', sizes: sized(1_200 * MB) },
        { category: 'jetbrains.logs', sizes: sized(40 * MB) },
        { category: 'jetbrains.settings', sizes: sized(85 * MB) },
      ],
      data: [],
      sizes: sized(1_325 * MB),
    },
  },
  {
    kind: 'app',
    value: {
      app: { id: 'com.jetbrains.datagrip', installed: false, name: 'DataGrip' },
      categories: [{ category: 'jetbrains.uninstalled', sizes: sized(420 * MB) }],
      data: [],
      sizes: sized(420 * MB),
    },
  },
];

// The module's settings: its switch and the days after which a removed IDE's data is offered.
export const JETBRAINS_SETTINGS: SettingsView = {
  file: 'loaded',
  problems: [],
  settings: [
    {
      default: { kind: 'switch', value: true },
      effect: 'next-scan',
      group: 'module',
      key: 'jetbrains.enabled',
      kind: { kind: 'switch' },
    },
    {
      default: { kind: 'number', value: 30 },
      effect: 'next-scan',
      group: 'module',
      key: 'jetbrains.uninstalled-after-days',
      kind: { kind: 'number', max: 3650, min: 7, unit: 'days' },
    },
  ],
  values: {
    'jetbrains.enabled': { kind: 'switch', value: true },
    'jetbrains.uninstalled-after-days': { kind: 'number', value: 30 },
  },
};

// The module's own texts, as its locale file has them, with the core's that the page and the confirmation show.
export const JETBRAINS_DICTIONARY: Dictionary = {
  ...SCRIPTS_DICTIONARY,
  ...FINDING_DICTIONARY,
  ...jetbrainsTexts,
  'core.group.development.description':
    'Tools for writing software: code editors, package managers, containers and projects.',
  'core.ignore.groups.description': 'Hides everything the modules of this group find and never cleans it.',
};
