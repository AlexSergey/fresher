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
import xcodeTexts from '../../../../modules/xcode/locales/en.json' with { type: 'json' };
import { FULL_COVERAGE, readyCategory, SCRIPTS_DICTIONARY } from './demo-scan';
import { FINDING_DICTIONARY } from './finding-fixtures';

// A scan of the xcode module as the core streams it: two iOS device support folders (the newer one carries the
// "recent" reason), Xcode's previews, the simulators' caches and one archive are offered, each for the app Xcode; one
// more device support folder is left out because it holds a socket. A folder is offered or left out, never both. The
// left-out folder counts towards the two newest versions, so 15.0 plain beside 17.2 recent is producible only because
// the left-out folder is a version above 15.0. The texts are the module's own, read from its locale file, so a
// changed text changes the tests. Played by the e2e mock.

const MB = 1_000_000;

const sized = (bytes: number): Sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

const text = (value: string): TextArg => ({ kind: 'text', value });

const DEVELOPER = '/Users/someone/Library/Developer';

export const DEVICE_SUPPORT = 'xcode.device-support';
export const PREVIEWS = 'xcode.previews';
export const SIMULATOR_CACHES = 'xcode.simulator-caches';
export const ARCHIVES = 'xcode.archives';

type Spec = {
  readonly bytes: number;
  readonly category: string;
  readonly danger: Finding['danger'];
  readonly id: number;
  readonly identity: string;
  readonly path: string;
  readonly reason: Finding['reason'];
  readonly removal: 'empty-folder' | 'move-to-trash';
  readonly title: Finding['title'];
};

const findingOf = (spec: Spec): Finding => ({
  app: 'com.apple.dt.Xcode',
  category: spec.category,
  danger: spec.danger,
  id: spec.id,
  key: { identity: spec.identity, module: 'xcode' },
  module: 'xcode',
  reason: spec.reason,
  removability: 'removable',
  removal: { kind: spec.removal, value: spec.path },
  sizes: sized(spec.bytes),
  subject: { kind: 'path', value: spec.path },
  title: spec.title,
});

const deviceSupport = (id: number, version: string, build: string, bytes: number, recent: boolean): Finding => {
  const folder = `${version} (${build})`;
  const args = { platform: text('iOS'), version: text(version) };

  return findingOf({
    bytes,
    category: DEVICE_SUPPORT,
    danger: 'low',
    id,
    identity: `ios/${folder}`,
    path: `${DEVELOPER}/Xcode/iOS DeviceSupport/${folder}`,
    reason: {
      args: { ...args, size: { kind: 'size', value: bytes } },
      key: recent ? 'xcode.reason.device-support-recent' : 'xcode.reason.device-support',
    },
    removal: 'move-to-trash',
    title: { args: { ...args, build: text(build) }, key: 'xcode.title.device-support' },
  });
};

const ARCHIVE_NAME = 'Shop 01-03-2026, 10.15';

export const XCODE_FINDINGS: readonly Finding[] = [
  deviceSupport(801, '15.0', '19A346', 3_200 * MB, false),
  deviceSupport(802, '17.2', '21C62', 4_100 * MB, true),
  findingOf({
    bytes: 640 * MB,
    category: PREVIEWS,
    danger: 'safe',
    id: 803,
    identity: 'previews',
    path: `${DEVELOPER}/Xcode/UserData/Previews`,
    reason: { args: { size: { kind: 'size', value: 640 * MB } }, key: 'xcode.reason.previews' },
    removal: 'empty-folder',
    title: { args: {}, key: 'xcode.title.previews' },
  }),
  findingOf({
    bytes: 1_200 * MB,
    category: SIMULATOR_CACHES,
    danger: 'safe',
    id: 804,
    identity: 'simulator-caches',
    path: `${DEVELOPER}/CoreSimulator/Caches`,
    reason: { args: { size: { kind: 'size', value: 1_200 * MB } }, key: 'xcode.reason.simulator-caches' },
    removal: 'empty-folder',
    title: { args: {}, key: 'xcode.title.simulator-caches' },
  }),
  findingOf({
    bytes: 860 * MB,
    category: ARCHIVES,
    danger: 'high',
    id: 805,
    identity: `archives/2026-03-01/${ARCHIVE_NAME}.xcarchive`,
    path: `${DEVELOPER}/Xcode/Archives/2026-03-01/${ARCHIVE_NAME}.xcarchive`,
    reason: {
      args: { date: text('2026-03-01'), size: { kind: 'size', value: 860 * MB } },
      key: 'xcode.reason.archive',
    },
    removal: 'move-to-trash',
    title: { args: { name: text(ARCHIVE_NAME) }, key: 'xcode.title.archive' },
  }),
];

// `get_modules`: the module in the group Development, its four categories, the core's generic switch, no choice.
export const XCODE_MODULES: readonly ModuleInfo[] = [
  {
    categories: [
      readyCategory(DEVICE_SUPPORT, 'low'),
      readyCategory(PREVIEWS, 'safe'),
      readyCategory(SIMULATOR_CACHES, 'safe'),
      readyCategory(ARCHIVES, 'high'),
    ],
    choices: [],
    group: 'development',
    id: 'xcode',
    settings: ['xcode.enabled'],
  },
];

// Another device support folder holds a socket a running program may use: kept.
export const XCODE_LEFT_OUT: readonly LeftOutCount[] = [
  {
    category: DEVICE_SUPPORT,
    items: 1,
    reason: { args: {}, key: 'xcode.left-out.special' },
    sizes: sized(300 * MB),
  },
];

export const XCODE_SCAN: readonly Event[] = [
  { modules: ['xcode'], scan: 1, type: 'scan-started' },
  { findings: XCODE_FINDINGS.slice(0, 3), scan: 1, type: 'findings-added' },
  { findings: XCODE_FINDINGS.slice(3), scan: 1, type: 'findings-added' },
  {
    leftOut: [...XCODE_LEFT_OUT],
    module: 'xcode',
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' },
];

export const XCODE_TOTALS: Overview['totals'] = [
  { category: DEVICE_SUPPORT, count: 2, highestDanger: 'low', sizes: sized(7_300 * MB) },
  { category: PREVIEWS, count: 1, highestDanger: 'safe', sizes: sized(640 * MB) },
  { category: SIMULATOR_CACHES, count: 1, highestDanger: 'safe', sizes: sized(1_200 * MB) },
  { category: ARCHIVES, count: 1, highestDanger: 'high', sizes: sized(860 * MB) },
];

export const XCODE_OVERVIEW: Overview = {
  found: sized(10_000 * MB),
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
  totals: XCODE_TOTALS,
};

// The module's settings: the core's generic switch, on by default.
export const XCODE_SETTINGS: SettingsView = {
  file: 'loaded',
  problems: [],
  settings: [
    {
      default: { kind: 'switch', value: true },
      effect: 'next-scan',
      group: 'module',
      key: 'xcode.enabled',
      kind: { kind: 'switch' },
    },
  ],
  values: { 'xcode.enabled': { kind: 'switch', value: true } },
};

// The core's own texts the page shows beside the module's: its group.
const coreOwn: Dictionary = Object.fromEntries(
  Object.entries(coreTexts).filter(([key]) => key.startsWith('core.group.development.')),
);

export const XCODE_DICTIONARY: Dictionary = {
  ...SCRIPTS_DICTIONARY,
  ...FINDING_DICTIONARY,
  ...xcodeTexts,
  ...coreOwn,
};
