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
import logsTexts from '../../../../modules/logs/locales/en.json' with { type: 'json' };
import { FULL_COVERAGE, readyCategory, SCRIPTS_DICTIONARY } from './demo-scan';
import { FINDING_DICTIONARY } from './finding-fixtures';
import { forNowCount, leftOutEntry } from './left-out';

// A scan of the logs module as the core streams it: old logs in the user's Logs folder, an old crash report, and one
// recent log left out. The texts are the module's own and the core's, read from their locale files. Shared by the
// unit tests and the e2e mock.

const KB = 1_000;
const MB = 1_000_000;

const LOGS = '/Users/demo/Library/Logs';

const sized = (bytes: number): Sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

const text = (value: string): TextArg => ({ kind: 'text', value });
const count = (value: number): TextArg => ({ kind: 'count', value });

type Spec = {
  readonly bytes: number;
  readonly category: 'logs.app-logs' | 'logs.crash-reports';
  readonly folder: null | string;
  readonly id: number;
  readonly identity: string;
  readonly name: string;
  readonly path: string;
};

const findingOf = ({ bytes, category, folder, id, identity, name, path }: Spec): Finding => {
  const reason =
    category === 'logs.crash-reports'
      ? { args: { days: count(30) }, key: 'logs.reason.report' }
      : folder === null
        ? { args: { days: count(30) }, key: 'logs.reason.top' }
        : { args: { days: count(30), folder: text(folder) }, key: 'logs.reason.folder' };

  return {
    category,
    danger: 'safe',
    id,
    key: { identity, module: 'logs' },
    module: 'logs',
    reason,
    removability: 'removable',
    removal: { kind: 'move-to-trash', value: path },
    sizes: sized(bytes),
    subject: { kind: 'path', value: path },
    title: { args: { name: text(name) }, key: 'logs.title' },
  };
};

export const LOGS_FINDINGS: readonly Finding[] = [
  findingOf({
    bytes: 38 * MB,
    category: 'logs.app-logs',
    folder: null,
    id: 1,
    identity: 'app-logs/old-tool.log',
    name: 'old-tool.log',
    path: `${LOGS}/old-tool.log`,
  }),
  findingOf({
    bytes: 6 * MB,
    category: 'logs.app-logs',
    folder: 'Writer',
    id: 2,
    identity: 'app-logs/Writer/writer.1.log',
    name: 'writer.1.log',
    path: `${LOGS}/Writer/writer.1.log`,
  }),
  findingOf({
    bytes: 640 * KB,
    category: 'logs.crash-reports',
    folder: null,
    id: 3,
    identity: 'crash-reports/Writer-2026-10-01-101010.ips',
    name: 'Writer-2026-10-01-101010.ips',
    path: `${LOGS}/DiagnosticReports/Writer-2026-10-01-101010.ips`,
  }),
];

// `get_modules`: the module in the group System, its two categories, its switch and its setting.
export const LOGS_MODULES: readonly ModuleInfo[] = [
  {
    categories: [readyCategory('logs.app-logs', 'safe'), readyCategory('logs.crash-reports', 'safe')],
    choices: [],
    group: 'system',
    id: 'logs',
    settings: ['logs.enabled', 'logs.older-than-days'],
  },
];

// What the module recognised and left out: one log written within the last week.
export const LOGS_LEFT_OUT: readonly LeftOutCount[] = [
  forNowCount('logs.app-logs', 'logs.left-out.recent', sized(3 * MB), [
    leftOutEntry({ args: { name: text('writer.log') }, key: 'logs.title' }, `${LOGS}/Writer/writer.log`, sized(3 * MB)),
  ]),
];

export const LOGS_SCAN: readonly Event[] = [
  { modules: ['logs'], scan: 1, type: 'scan-started' },
  { findings: [...LOGS_FINDINGS], scan: 1, type: 'findings-added' },
  {
    insights: [],
    leftOut: [...LOGS_LEFT_OUT],
    module: 'logs',
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' },
];

const APP_LOGS_BYTES = 44 * MB;
const CRASH_BYTES = 640 * KB;

export const LOGS_OVERVIEW: Overview = {
  found: sized(APP_LOGS_BYTES + CRASH_BYTES),
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
    { category: 'logs.app-logs', count: 2, highestDanger: 'safe', sizes: sized(APP_LOGS_BYTES) },
    { category: 'logs.crash-reports', count: 1, highestDanger: 'safe', sizes: sized(CRASH_BYTES) },
  ],
};

// The module's settings: its switch and the days a file must stay unchanged, from 7 (the check only gets stricter).
export const LOGS_SETTINGS: SettingsView = {
  file: 'loaded',
  problems: [],
  settings: [
    {
      default: { kind: 'switch', value: true },
      effect: 'next-scan',
      group: 'module',
      key: 'logs.enabled',
      kind: { kind: 'switch' },
    },
    {
      default: { kind: 'number', value: 7 },
      effect: 'next-scan',
      group: 'module',
      key: 'logs.older-than-days',
      kind: { kind: 'number', max: 90, min: 7, unit: 'days' },
    },
  ],
  values: {
    'logs.enabled': { kind: 'switch', value: true },
    'logs.older-than-days': { kind: 'number', value: 7 },
  },
};

const named = (...keys: readonly string[]): Dictionary =>
  Object.fromEntries(keys.map((key) => [key, (coreTexts as Dictionary)[key] ?? '']));

// The module's own texts, with the core's that the sidebar, the ignore tabs and the permissions show.
export const LOGS_DICTIONARY: Dictionary = {
  ...SCRIPTS_DICTIONARY,
  ...FINDING_DICTIONARY,
  ...logsTexts,
  ...named(
    'core.access.all-files.name',
    'core.access.all-files.unlocks',
    'core.access.state.granted',
    'core.group.system.description',
    'core.group.system.name',
    'core.ignore.groups.description',
  ),
};
