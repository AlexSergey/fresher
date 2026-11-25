import type {
  Dictionary,
  Event,
  Finding,
  LeftOutCount,
  ModuleInfo,
  Overview,
  SettingsView,
  Sizes,
  StorageMap,
  TextArg,
} from '../ipc/bindings.generated';

import coreTexts from '../../../../crates/core/locales/en.json' with { type: 'json' };
import tempTexts from '../../../../modules/temp/locales/en.json' with { type: 'json' };
import { FULL_COVERAGE, readyCategory, SCRIPTS_DICTIONARY } from './demo-scan';
import { FINDING_DICTIONARY } from './finding-fixtures';
import { forNowCount, keptCount, leftOutEntry } from './left-out';
import { DEMO_MAP_COMPLETE } from './storage-map';

// A scan of the temporary files module as the core streams it: old items in the user's temporary folder and in the
// shared one, and what was recognised and left out (thousands of recent items, a few of the system's, one pipe). The
// texts are the module's own and the core's, read from their locale files, so a changed text changes the tests.
// Shared by the unit tests and the e2e mock.

const MB = 1_000_000;

const sized = (bytes: number): Sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

const text = (value: string): TextArg => ({ kind: 'text', value });

const USER_TEMP = '/private/var/folders/ab/c1d2e3f4g5h6/T';
const SHARED_TEMP = '/private/tmp';

type Spec = {
  readonly bytes: number;
  readonly days: number;
  readonly id: number;
  readonly name: string;
  readonly place: 'shared-temp' | 'user-temp';
};

const findingOf = ({ bytes, days, id, name, place }: Spec): Finding => {
  const path = `${place === 'user-temp' ? USER_TEMP : SHARED_TEMP}/${name}`;

  return {
    category: 'temp.old-files',
    danger: 'safe',
    id,
    key: { identity: `${place}/${name}`, module: 'temp' },
    module: 'temp',
    reason: {
      args: { days: { kind: 'count', value: days } },
      key: `temp.finding.reason-${place.replace('-temp', '')}`,
    },
    removability: 'removable',
    removal: { kind: 'move-to-trash', value: path },
    sizes: sized(bytes),
    subject: { kind: 'path', value: path },
    title: { args: { name: text(name) }, key: 'temp.finding.title' },
  };
};

export const TEMP_FINDINGS: readonly Finding[] = [
  findingOf({ bytes: 420 * MB, days: 21, id: 1, name: 'pip-unpack-ak3j2', place: 'user-temp' }),
  findingOf({ bytes: 180 * MB, days: 12, id: 2, name: 'swift-build-7f3a', place: 'user-temp' }),
  findingOf({ bytes: 95 * MB, days: 9, id: 3, name: 'npm-cache-tmp', place: 'user-temp' }),
  findingOf({ bytes: 140 * MB, days: 30, id: 4, name: 'installer-staging', place: 'shared-temp' }),
  findingOf({ bytes: 2 * MB, days: 8, id: 5, name: 'export.log', place: 'shared-temp' }),
];

// `get_modules`: the module in the group System, its one category and its setting.
export const TEMP_MODULES: readonly ModuleInfo[] = [
  {
    categories: [readyCategory('temp.old-files', 'safe')],
    choices: [],
    group: 'system',
    id: 'temp',
    settings: ['temp.enabled', 'temp.older-than-days'],
  },
];

// The ten largest of the recent items, as the engine lists them: 60 MB down to 15 MB.
const RECENT_SHOWN = Array.from({ length: 10 }, (_, index) =>
  leftOutEntry(
    { args: { name: text(`item-${index + 1}`) }, key: 'temp.finding.title' },
    `${USER_TEMP}/item-${index + 1}`,
    sized((60 - index * 5) * MB),
  ),
);

// Thousands of recent items (the ten largest listed), a few of the system's and one pipe, both kept for good.
export const TEMP_LEFT_OUT: readonly LeftOutCount[] = [
  forNowCount('temp.old-files', 'temp.left-out.recent', sized(790 * MB), RECENT_SHOWN, 4_712),
  keptCount('temp.old-files', 'temp.left-out.system', 3, sized(12 * MB)),
  keptCount('temp.old-files', 'temp.left-out.special', 1, sized(0)),
];

export const TEMP_SCAN: readonly Event[] = [
  { modules: ['temp'], scan: 1, type: 'scan-started' },
  { findings: TEMP_FINDINGS.slice(0, 3), scan: 1, type: 'findings-added' },
  { findings: TEMP_FINDINGS.slice(3), scan: 1, type: 'findings-added' },
  {
    insights: [],
    leftOut: [...TEMP_LEFT_OUT],
    module: 'temp',
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' },
];

export const TEMP_OVERVIEW: Overview = {
  found: sized(837 * MB),
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
  totals: [{ category: 'temp.old-files', count: 5, highestDanger: 'safe', sizes: sized(837 * MB) }],
};

// The module's settings: its switch and the days an item must stay unchanged, from 7 (the check only gets stricter).
export const TEMP_SETTINGS: SettingsView = {
  file: 'loaded',
  problems: [],
  settings: [
    {
      default: { kind: 'switch', value: true },
      effect: 'next-scan',
      group: 'module',
      key: 'temp.enabled',
      kind: { kind: 'switch' },
    },
    {
      default: { kind: 'number', value: 7 },
      effect: 'next-scan',
      group: 'module',
      key: 'temp.older-than-days',
      kind: { kind: 'number', max: 30, min: 7, unit: 'days' },
    },
  ],
  values: {
    'temp.enabled': { kind: 'switch', value: true },
    'temp.older-than-days': { kind: 'number', value: 7 },
  },
};

// The storage map after the scan: the home and the two temporary folders as roots, the folders named by the core.
const HOME_ROOT = DEMO_MAP_COMPLETE.volumes[0]?.roots[0] ?? { index: 0, state: 1 };
const USER_ROOT = { index: 40, state: 1 };
const SHARED_ROOT = { index: 41, state: 1 };

export const TEMP_MAP: StorageMap = {
  ...DEMO_MAP_COMPLETE,
  volumes: DEMO_MAP_COMPLETE.volumes.map((volume) => ({
    ...volume,
    places: [
      { name: { args: {}, key: 'core.place.user-temp.name' }, root: USER_ROOT },
      { name: { args: {}, key: 'core.place.shared-temp.name' }, root: SHARED_ROOT },
    ],
    roots: [HOME_ROOT, USER_ROOT, SHARED_ROOT],
  })),
};

const named = (...keys: readonly string[]): Dictionary =>
  Object.fromEntries(keys.map((key) => [key, (coreTexts as Dictionary)[key] ?? '']));

// The module's own texts, with the core's that the sidebar, the places, the ignore tabs and the permissions show.
export const TEMP_DICTIONARY: Dictionary = {
  ...SCRIPTS_DICTIONARY,
  ...FINDING_DICTIONARY,
  ...tempTexts,
  ...named(
    'core.access.all-files.name',
    'core.access.all-files.unlocks',
    'core.access.state.granted',
    'core.group.system.description',
    'core.group.system.name',
    'core.ignore.groups.description',
    'core.place.shared-temp.name',
    'core.place.user-temp.name',
  ),
};
