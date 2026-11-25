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

import devCachesTexts from '../../../../modules/dev-caches/locales/en.json' with { type: 'json' };
import { FULL_COVERAGE, readyCategory, SCRIPTS_DICTIONARY } from './demo-scan';
import { FINDING_DICTIONARY } from './finding-fixtures';

// Scans of the developer caches module as the core streams them. The module's own texts are read from its locale
// file, so a changed text changes the tests. Shared by the e2e files.
//
// Two scans, because the core leaves out every derived data project while Xcode is open (a project is never offered
// and left out in one scan): `DEV_CACHES_SCAN` has Xcode closed (npm, Cargo's two folders, Gradle and two projects
// offered; a third project not the user's), `DEV_CACHES_XCODE_OPEN_SCAN` has it open (the projects left out
// `app-open`, the other caches offered, one Gradle folder not the user's).

const MB = 1_000_000;

const sized = (bytes: number): Sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

const text = (value: string): TextArg => ({ kind: 'text', value });

const HOME = '/Users/someone';
const XCODE = 'com.apple.dt.Xcode';

type Spec = {
  readonly app?: string;
  readonly bytes: number;
  readonly category: string;
  readonly danger: Finding['danger'];
  readonly id: number;
  readonly identity: string;
  readonly path: string;
  readonly reason: 'build' | 'download';
  readonly title: Finding['title'];
  readonly tool: string;
};

const findingOf = (spec: Spec): Finding => ({
  ...(spec.app === undefined ? {} : { app: spec.app }),
  category: spec.category,
  danger: spec.danger,
  id: spec.id,
  key: { identity: spec.identity, module: 'dev-caches' },
  module: 'dev-caches',
  reason: { args: { tool: text(spec.tool) }, key: `dev-caches.finding.reason-${spec.reason}` },
  removability: 'removable',
  removal: { kind: 'move-to-trash', value: `${HOME}/${spec.path}` },
  sizes: sized(spec.bytes),
  subject: { kind: 'path', value: `${HOME}/${spec.path}` },
  title: spec.title,
});

const titled = (key: string, args: Finding['title']['args'] = {}): Finding['title'] => ({
  args,
  key: `dev-caches.title.${key}`,
});

const project = (id: number, name: string, bytes: number): Finding =>
  findingOf({
    app: XCODE,
    bytes,
    category: 'dev-caches.xcode-derived-data',
    danger: 'safe',
    id,
    identity: `xcode-derived-data/${name}`,
    path: `Library/Developer/Xcode/DerivedData/${name}`,
    reason: 'build',
    title: titled('xcode-derived-data', { project: text(name.slice(0, name.lastIndexOf('-'))) }),
    tool: 'Xcode',
  });

const NPM = findingOf({
  bytes: 8_900 * MB,
  category: 'dev-caches.npm',
  danger: 'low',
  id: 1,
  identity: 'npm/_cacache',
  path: '.npm/_cacache',
  reason: 'download',
  title: titled('npm'),
  tool: 'npm',
});

const CARGO_CACHE = findingOf({
  bytes: 1_100 * MB,
  category: 'dev-caches.cargo',
  danger: 'low',
  id: 2,
  identity: 'cargo-cache/cache',
  path: '.cargo/registry/cache',
  reason: 'download',
  title: titled('cargo-cache'),
  tool: 'Cargo',
});

const CARGO_SRC = findingOf({
  bytes: 640 * MB,
  category: 'dev-caches.cargo',
  danger: 'low',
  id: 3,
  identity: 'cargo-src/src',
  path: '.cargo/registry/src',
  reason: 'download',
  title: titled('cargo-src'),
  tool: 'Cargo',
});

const GRADLE = findingOf({
  bytes: 2_300 * MB,
  category: 'dev-caches.gradle',
  danger: 'low',
  id: 4,
  identity: 'gradle/caches',
  path: '.gradle/caches',
  reason: 'download',
  title: titled('gradle'),
  tool: 'Gradle',
});

const PROJECT_APP = project(5, 'Notes-abcdefghijklmnopqrstuvwxyzab', 3_400 * MB);
const PROJECT_SKETCH = project(6, 'Sketch-bcdefghijklmnopqrstuvwxyzabc', 860 * MB);

// Xcode closed: the downloads and both projects.
export const DEV_CACHES_FINDINGS: readonly Finding[] = [
  NPM,
  CARGO_CACHE,
  CARGO_SRC,
  GRADLE,
  PROJECT_APP,
  PROJECT_SKETCH,
];

// Xcode open: the projects are left out, so only the downloads are offered.
export const DEV_CACHES_XCODE_OPEN_FINDINGS: readonly Finding[] = [NPM, CARGO_CACHE, CARGO_SRC, GRADLE];

// `get_modules`: the module in the group Development, its nine categories and its one setting.
export const DEV_CACHES_MODULES: readonly ModuleInfo[] = [
  {
    categories: [
      readyCategory('dev-caches.npm', 'low'),
      readyCategory('dev-caches.yarn', 'low'),
      readyCategory('dev-caches.homebrew', 'low'),
      readyCategory('dev-caches.cargo', 'low'),
      readyCategory('dev-caches.cocoapods', 'low'),
      readyCategory('dev-caches.gradle', 'low'),
      readyCategory('dev-caches.pip', 'low'),
      readyCategory('dev-caches.go-build', 'safe'),
      readyCategory('dev-caches.xcode-derived-data', 'safe'),
    ],
    choices: [],
    group: 'development',
    id: 'dev-caches',
    settings: ['dev-caches.enabled'],
  },
];

const left = (category: string, key: string, items: number, bytes: number): LeftOutCount => ({
  category,
  items,
  reason: { args: {}, key: `dev-caches.left-out.${key}` },
  sizes: sized(bytes),
});

// One project that is not the user's (root's files after a command run with sudo).
export const DEV_CACHES_LEFT_OUT: readonly LeftOutCount[] = [
  left('dev-caches.xcode-derived-data', 'not-yours', 1, 410 * MB),
];

// Xcode open: both projects wait for it to quit, and one Gradle folder is not the user's.
export const DEV_CACHES_XCODE_OPEN_LEFT_OUT: readonly LeftOutCount[] = [
  left('dev-caches.xcode-derived-data', 'app-open', 2, 4_260 * MB),
  left('dev-caches.gradle', 'not-yours', 1, 35 * MB),
];

const scanOf = (findings: readonly Finding[], leftOut: readonly LeftOutCount[]): readonly Event[] => [
  { modules: ['dev-caches'], scan: 1, type: 'scan-started' },
  { findings: findings.slice(0, 3), scan: 1, type: 'findings-added' },
  { findings: findings.slice(3), scan: 1, type: 'findings-added' },
  {
    insights: [],
    leftOut: [...leftOut],
    module: 'dev-caches',
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' },
];

export const DEV_CACHES_SCAN: readonly Event[] = scanOf(DEV_CACHES_FINDINGS, DEV_CACHES_LEFT_OUT);

export const DEV_CACHES_XCODE_OPEN_SCAN: readonly Event[] = scanOf(
  DEV_CACHES_XCODE_OPEN_FINDINGS,
  DEV_CACHES_XCODE_OPEN_LEFT_OUT,
);

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

const DOWNLOAD_TOTALS: Overview['totals'] = [
  { category: 'dev-caches.npm', count: 1, highestDanger: 'low', sizes: sized(8_900 * MB) },
  { category: 'dev-caches.cargo', count: 2, highestDanger: 'low', sizes: sized(1_740 * MB) },
  { category: 'dev-caches.gradle', count: 1, highestDanger: 'low', sizes: sized(2_300 * MB) },
];

export const DEV_CACHES_OVERVIEW: Overview = overviewOf(17_200 * MB, [
  ...DOWNLOAD_TOTALS,
  { category: 'dev-caches.xcode-derived-data', count: 2, highestDanger: 'safe', sizes: sized(4_260 * MB) },
]);

export const DEV_CACHES_XCODE_OPEN_OVERVIEW: Overview = overviewOf(12_940 * MB, DOWNLOAD_TOTALS);

// The by-meaning view: the projects' build data is Xcode's line, the downloads belong to no app.
export const DEV_CACHES_MEANING: readonly MeaningGroup[] = [
  {
    kind: 'app',
    value: {
      app: { id: XCODE, installed: true, name: 'Xcode' },
      categories: [{ category: 'dev-caches.xcode-derived-data', sizes: sized(4_260 * MB) }],
      data: [],
      sizes: sized(4_260 * MB),
    },
  },
];

// The module's settings: its switch alone.
export const DEV_CACHES_SETTINGS: SettingsView = {
  file: 'loaded',
  problems: [],
  settings: [
    {
      default: { kind: 'switch', value: true },
      effect: 'next-scan',
      group: 'module',
      key: 'dev-caches.enabled',
      kind: { kind: 'switch' },
    },
  ],
  values: { 'dev-caches.enabled': { kind: 'switch', value: true } },
};

// The module's own texts, with the core's that the page and the confirmation show.
export const DEV_CACHES_DICTIONARY: Dictionary = {
  ...SCRIPTS_DICTIONARY,
  ...FINDING_DICTIONARY,
  ...devCachesTexts,
  'core.group.development.description':
    'Tools for writing software: code editors, package managers, containers and projects.',
  'core.ignore.groups.description': 'Hides everything the modules of this group find and never cleans it.',
};
