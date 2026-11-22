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

import staleTexts from '../../../../modules/stale-projects/locales/en.json' with { type: 'json' };
import { FULL_COVERAGE, SCRIPTS_DICTIONARY } from './demo-scan';
import { FINDING_DICTIONARY } from './finding-fixtures';

// A scan of the old projects module as the core streams it: the packages of two folders of one project, a build
// folder, a Python environment and a Rust incremental cache are offered; two more `node_modules` and a `target` are
// left out (their projects changed within the days) and a coverage build too (a build used it within 7 days). An item
// is offered or left out, never both. The texts are the module's own, read from its locale file, so a changed text
// changes the tests. Shared by the unit tests and the e2e mock.

const MB = 1_000_000;

const sized = (bytes: number): Sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

const text = (value: string): TextArg => ({ kind: 'text', value });

const HOME = '/Users/someone';

type Spec = {
  readonly bytes: number;
  readonly category: string;
  readonly danger: Finding['danger'];
  readonly id: number;
  readonly path: string;
  readonly project: string;
  readonly reason: Finding['reason'];
};

const findingOf = (spec: Spec): Finding => {
  const folder = spec.path.slice(spec.path.indexOf(spec.project) + spec.project.length + 1);

  return {
    category: spec.category,
    danger: spec.danger,
    id: spec.id,
    key: {
      identity: `${spec.category.slice('stale-projects.'.length)}/${spec.path}`,
      module: 'stale-projects',
    },
    module: 'stale-projects',
    reason: spec.reason,
    removability: 'removable',
    removal: { kind: 'move-to-trash', value: `${HOME}/${spec.path}` },
    sizes: sized(spec.bytes),
    subject: { kind: 'path', value: `${HOME}/${spec.path}` },
    title: { args: { folder: text(folder), project: text(spec.project) }, key: 'stale-projects.title' },
  };
};

// 2023-11-15 12:00 UTC: midday, so the date is Nov 15 in every time zone the tests run in.
const SHOP_SINCE = 1_700_049_600;

const since = (command: string, date: number, key: string): Finding['reason'] => ({
  args: { command: text(command), date: { kind: 'date', value: date } },
  key: `stale-projects.reason.${key}`,
});

// Low packages and build output, a Medium Python environment, a Low Rust build cache.
export const STALE_PROJECTS_FINDINGS: readonly Finding[] = [
  findingOf({
    bytes: 640 * MB,
    category: 'stale-projects.dependencies',
    danger: 'low',
    id: 401,
    path: 'Code/shop/node_modules',
    project: 'shop',
    reason: since('npm install', SHOP_SINCE, 'dependencies'),
  }),
  findingOf({
    bytes: 410 * MB,
    category: 'stale-projects.dependencies',
    danger: 'low',
    id: 402,
    path: 'Code/shop/packages/web/node_modules',
    project: 'shop',
    reason: since('npm install', SHOP_SINCE, 'dependencies'),
  }),
  findingOf({
    bytes: 3_200 * MB,
    category: 'stale-projects.build-output',
    danger: 'low',
    id: 403,
    path: 'Code/tool/target',
    project: 'tool',
    reason: since('cargo build', 1_690_000_000, 'build-output'),
  }),
  findingOf({
    bytes: 520 * MB,
    category: 'stale-projects.python-environments',
    danger: 'medium',
    id: 404,
    path: 'Code/py/.venv',
    project: 'py',
    reason: since('python3 -m venv .venv', 1_695_000_000, 'python-environments'),
  }),
  findingOf({
    bytes: 1_800 * MB,
    category: 'stale-projects.rust-build-cache',
    danger: 'low',
    id: 405,
    path: 'Code/daily/target/debug/incremental',
    project: 'daily',
    reason: {
      args: { command: text('cargo build'), days: { kind: 'count', value: 7 } },
      key: 'stale-projects.reason.rust-build-cache',
    },
  }),
];

const readyCategory = (id: string, danger: Finding['danger']): ModuleInfo['categories'][number] => ({
  availability: { kind: 'ready' },
  descriptor: { danger, id, needs: [] },
});

// `get_modules`: the module in the group Development, its four categories, its two settings, no choice.
export const STALE_PROJECTS_MODULES: readonly ModuleInfo[] = [
  {
    categories: [
      readyCategory('stale-projects.dependencies', 'low'),
      readyCategory('stale-projects.build-output', 'low'),
      readyCategory('stale-projects.python-environments', 'medium'),
      readyCategory('stale-projects.rust-build-cache', 'low'),
    ],
    choices: [],
    group: 'development',
    id: 'stale-projects',
    settings: ['stale-projects.enabled', 'stale-projects.unused-for-days'],
  },
];

const left = (category: string, key: string, items: number, bytes: number): LeftOutCount => ({
  category,
  items,
  reason: { args: {}, key: `stale-projects.left-out.${key}` },
  sizes: sized(bytes),
});

// Two `node_modules` and a `target` of projects changed within the days, a coverage build used within 7 days.
export const STALE_PROJECTS_LEFT_OUT: readonly LeftOutCount[] = [
  left('stale-projects.dependencies', 'recent', 2, 1_300 * MB),
  left('stale-projects.build-output', 'recent', 1, 900 * MB),
  left('stale-projects.rust-build-cache', 'cache-recent', 1, 2_100 * MB),
];

export const STALE_PROJECTS_SCAN: readonly Event[] = [
  { modules: ['stale-projects'], scan: 1, type: 'scan-started' },
  { findings: STALE_PROJECTS_FINDINGS.slice(0, 3), scan: 1, type: 'findings-added' },
  { findings: STALE_PROJECTS_FINDINGS.slice(3), scan: 1, type: 'findings-added' },
  {
    insights: [],
    leftOut: [...STALE_PROJECTS_LEFT_OUT],
    module: 'stale-projects',
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' },
];

export const STALE_PROJECTS_TOTALS: Overview['totals'] = [
  { category: 'stale-projects.dependencies', count: 2, highestDanger: 'low', sizes: sized(1_050 * MB) },
  { category: 'stale-projects.build-output', count: 1, highestDanger: 'low', sizes: sized(3_200 * MB) },
  { category: 'stale-projects.python-environments', count: 1, highestDanger: 'medium', sizes: sized(520 * MB) },
  { category: 'stale-projects.rust-build-cache', count: 1, highestDanger: 'low', sizes: sized(1_800 * MB) },
];

export const STALE_PROJECTS_OVERVIEW: Overview = {
  found: sized(6_570 * MB),
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
  totals: STALE_PROJECTS_TOTALS,
};

// The module's settings: its switch and the days a project must be unchanged before its folders are offered.
export const STALE_PROJECTS_SETTINGS: SettingsView = {
  file: 'loaded',
  problems: [],
  settings: [
    {
      default: { kind: 'switch', value: true },
      effect: 'next-scan',
      group: 'module',
      key: 'stale-projects.enabled',
      kind: { kind: 'switch' },
    },
    {
      default: { kind: 'number', value: 365 },
      effect: 'next-scan',
      group: 'module',
      key: 'stale-projects.unused-for-days',
      kind: { kind: 'number', max: 3650, min: 30, unit: 'days' },
    },
  ],
  values: {
    'stale-projects.enabled': { kind: 'switch', value: true },
    'stale-projects.unused-for-days': { kind: 'number', value: 365 },
  },
};

export const STALE_PROJECTS_DICTIONARY: Dictionary = {
  ...SCRIPTS_DICTIONARY,
  ...FINDING_DICTIONARY,
  ...staleTexts,
};
