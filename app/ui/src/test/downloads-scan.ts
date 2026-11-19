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

import coreTexts from '../../../../crates/core/locales/en.json' with { type: 'json' };
import downloadsTexts from '../../../../modules/downloads/locales/en.json' with { type: 'json' };
import { accessStatus, FULL_COVERAGE, SCRIPTS_DICTIONARY } from './demo-scan';
import { FINDING_DICTIONARY } from './finding-fixtures';

// A scan of the downloads and installers module as the core streams it, and a second state of the same module before
// Downloads is allowed. In the first one an unfinished download and its empty placeholder, a Safari download, a
// Firefox installer and a package, an old macOS installer and a big file are offered; another unfinished download is
// left out (its browser is open) and a big file too (it changed within the days). An id is offered or left out, never
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

const FOLDER = '/Users/someone/Downloads';
const DOWNLOADS = { kind: 'folder', value: 'downloads' } as const;

type Spec = {
  readonly app?: string;
  readonly bytes: number;
  readonly category: string;
  readonly danger: Finding['danger'];
  readonly id: number;
  readonly name: string;
  readonly reason: Finding['reason'];
};

const findingOf = (spec: Spec): Finding => ({
  ...(spec.app === undefined ? {} : { app: spec.app }),
  category: spec.category,
  danger: spec.danger,
  id: spec.id,
  key: { identity: `${spec.category.slice('downloads.'.length)}/${spec.name}`, module: 'downloads' },
  module: 'downloads',
  reason: spec.reason,
  removability: 'removable',
  removal: { kind: 'move-to-trash', value: `${FOLDER}/${spec.name}` },
  sizes: sized(spec.bytes),
  subject: { kind: 'path', value: `${FOLDER}/${spec.name}` },
  title: { args: { name: text(spec.name) }, key: 'downloads.title' },
});

const stopped = (days: number): Finding['reason'] => ({
  args: { days: { kind: 'count', value: days } },
  key: 'downloads.reason.unfinished',
});

// Low unfinished downloads, a Medium Safari one, Low installers of installed apps and of macOS, a High big file.
export const DOWNLOADS_FINDINGS: readonly Finding[] = [
  findingOf({
    bytes: 310 * MB,
    category: 'downloads.unfinished',
    danger: 'low',
    id: 301,
    name: 'movie.mp4.crdownload',
    reason: stopped(30),
  }),
  findingOf({
    bytes: 0,
    category: 'downloads.unfinished',
    danger: 'low',
    id: 302,
    name: 'movie.mp4',
    reason: { args: {}, key: 'downloads.reason.placeholder' },
  }),
  findingOf({
    bytes: 120 * MB,
    category: 'downloads.safari-resumable',
    danger: 'medium',
    id: 303,
    name: 'trip.zip.download',
    reason: { args: { days: { kind: 'count', value: 30 } }, key: 'downloads.reason.safari-resumable' },
  }),
  findingOf({
    app: 'org.mozilla.firefox',
    bytes: 135 * MB,
    category: 'downloads.installers',
    danger: 'low',
    id: 304,
    name: 'Firefox 133.0.dmg',
    reason: { args: { app: text('Firefox'), version: text('133.0') }, key: 'downloads.reason.installer' },
  }),
  findingOf({
    bytes: 30 * MB,
    category: 'downloads.installers',
    danger: 'low',
    id: 305,
    name: 'Example-2.0.pkg',
    reason: { args: { date: { kind: 'date', value: 1_766_361_600 } }, key: 'downloads.reason.installer-receipt' },
  }),
  findingOf({
    bytes: 12_800 * MB,
    category: 'downloads.macos-installers',
    danger: 'low',
    id: 306,
    name: 'Install macOS Sonoma.app',
    reason: { args: { running: text('15.1') }, key: 'downloads.reason.macos-installer' },
  }),
  findingOf({
    bytes: 2_400 * MB,
    category: 'downloads.large-untouched',
    danger: 'high',
    id: 307,
    name: 'video.mov',
    reason: {
      args: { date: { kind: 'date', value: 1_732_665_600 }, size: { kind: 'size', value: 2_400 * MB } },
      key: 'downloads.reason.large-untouched',
    },
  }),
];

const needsDownloads = [{ capability: DOWNLOADS, kind: 'required' }] as const;

const descriptorOf = (id: string, danger: Finding['danger'], ready: boolean): ModuleInfo['categories'][number] => ({
  availability: ready ? { kind: 'ready' } : { kind: 'unavailable', missing: [DOWNLOADS] },
  descriptor: { danger, id, needs: [...needsDownloads] },
});

const modulesOf = (ready: boolean): readonly ModuleInfo[] => [
  {
    categories: [
      descriptorOf('downloads.unfinished', 'low', ready),
      descriptorOf('downloads.safari-resumable', 'medium', ready),
      descriptorOf('downloads.installers', 'low', ready),
      descriptorOf('downloads.macos-installers', 'low', ready),
      descriptorOf('downloads.large-untouched', 'high', ready),
    ],
    choices: [],
    group: 'files',
    id: 'downloads',
    settings: ['downloads.enabled', 'downloads.older-than-days'],
  },
];

// `get_modules`: the module in the group Your files, its five categories, its two settings, no choice.
export const DOWNLOADS_MODULES: readonly ModuleInfo[] = modulesOf(true);

// The same before Downloads is allowed: every category needs it and none can be cleaned.
export const DOWNLOADS_UNAVAILABLE_MODULES: readonly ModuleInfo[] = modulesOf(false);

const left = (category: string, key: string, bytes: number): LeftOutCount => ({
  category,
  items: 1,
  reason: { args: {}, key: `downloads.left-out.${key}` },
  sizes: sized(bytes),
});

// One download whose browser is open and one big file that changed within the days: no finding of their own.
export const DOWNLOADS_LEFT_OUT: readonly LeftOutCount[] = [
  left('downloads.unfinished', 'browser-open', 48 * MB),
  left('downloads.large-untouched', 'recent', 1_200 * MB),
];

const FINISHED: Event = { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' };

export const DOWNLOADS_SCAN: readonly Event[] = [
  { modules: ['downloads'], scan: 1, type: 'scan-started' },
  { findings: DOWNLOADS_FINDINGS.slice(0, 4), scan: 1, type: 'findings-added' },
  { findings: DOWNLOADS_FINDINGS.slice(4), scan: 1, type: 'findings-added' },
  {
    leftOut: [...DOWNLOADS_LEFT_OUT],
    module: 'downloads',
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  FINISHED,
];

// The first scan before Downloads is allowed: the module could not look, and says what it missed.
export const DOWNLOADS_UNAVAILABLE_SCAN: readonly Event[] = [
  { modules: ['downloads'], scan: 1, type: 'scan-started' },
  {
    leftOut: [],
    module: 'downloads',
    outcome: { kind: 'unavailable', missing: [DOWNLOADS] },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  FINISHED,
];

export const DOWNLOADS_TOTALS: Overview['totals'] = [
  { category: 'downloads.unfinished', count: 2, highestDanger: 'low', sizes: sized(310 * MB) },
  { category: 'downloads.safari-resumable', count: 1, highestDanger: 'medium', sizes: sized(120 * MB) },
  { category: 'downloads.installers', count: 2, highestDanger: 'low', sizes: sized(165 * MB) },
  { category: 'downloads.macos-installers', count: 1, highestDanger: 'low', sizes: sized(12_800 * MB) },
  { category: 'downloads.large-untouched', count: 1, highestDanger: 'high', sizes: sized(2_400 * MB) },
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

export const DOWNLOADS_OVERVIEW: Overview = overviewOf(15_795 * MB, DOWNLOADS_TOTALS);

export const DOWNLOADS_UNAVAILABLE_OVERVIEW: Overview = overviewOf(0, []);

// The by-meaning view: the Firefox installer is the app's, named and installed.
export const DOWNLOADS_MEANING: readonly MeaningGroup[] = [
  {
    kind: 'app',
    value: {
      app: { id: 'org.mozilla.firefox', installed: true, name: 'Firefox' },
      categories: [{ category: 'downloads.installers', sizes: sized(135 * MB) }],
      data: [],
      sizes: sized(135 * MB),
    },
  },
];

// The module's settings: its switch and the days after which an unfinished download or a big file is offered.
export const DOWNLOADS_SETTINGS: SettingsView = {
  file: 'loaded',
  problems: [],
  settings: [
    {
      default: { kind: 'switch', value: true },
      effect: 'next-scan',
      group: 'module',
      key: 'downloads.enabled',
      kind: { kind: 'switch' },
    },
    {
      default: { kind: 'number', value: 30 },
      effect: 'next-scan',
      group: 'module',
      key: 'downloads.older-than-days',
      kind: { kind: 'number', max: 3650, min: 7, unit: 'days' },
    },
  ],
  values: {
    'downloads.enabled': { kind: 'switch', value: true },
    'downloads.older-than-days': { kind: 'number', value: 30 },
  },
};

// What `get_access_status` answers with Downloads allowed and not yet asked.
export const DOWNLOADS_GRANTED = accessStatus([
  [{ kind: 'all-files' }, 'granted'],
  [DOWNLOADS, 'granted'],
]);

export const DOWNLOADS_NOT_ASKED = accessStatus([
  [{ kind: 'all-files' }, 'granted'],
  [DOWNLOADS, 'not-asked'],
]);

// The core's own texts the page shows beside the module's: the group, Downloads' access sentences.
const coreOwn: Dictionary = Object.fromEntries(
  Object.entries(coreTexts).filter(
    ([key]) => key.startsWith('core.access.downloads.') || key.startsWith('core.group.files.'),
  ),
);

export const DOWNLOADS_DICTIONARY: Dictionary = {
  ...SCRIPTS_DICTIONARY,
  ...FINDING_DICTIONARY,
  ...downloadsTexts,
  ...coreOwn,
};
