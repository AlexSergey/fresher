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
import toolchainTexts from '../../../../modules/unused-toolchains/locales/en.json' with { type: 'json' };
import { iconOf } from './category-icons';
import { accessStatus, FULL_COVERAGE, SCRIPTS_DICTIONARY } from './demo-scan';
import { FINDING_DICTIONARY } from './finding-fixtures';
import { keptCount } from './left-out';

// A scan of the unused toolchains module as the core streams it, with the Node switch on: two Rust toolchains and a
// Node version are offered; another Node version is left out (it belongs to another user). An item is offered or left
// out, never both. The second scan is the module before Documents is allowed: every
// category needs it and none could look. The texts are the module's own, read from its locale file, so a changed
// text changes the tests. Used by the e2e mock.

const MB = 1_000_000;

const sized = (bytes: number): Sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

const text = (value: string): TextArg => ({ kind: 'text', value });

const HOME = '/Users/someone';
const DOCUMENTS = { kind: 'folder', value: 'documents' } as const;

type Spec = {
  readonly bytes: number;
  readonly category: string;
  readonly command: string;
  readonly danger: Finding['danger'];
  readonly id: number;
  readonly kind: 'node' | 'rust';
  readonly name: string;
  readonly path: string;
};

// 2024-03-10 12:00 UTC: midday, so the date is Mar 10 in every time zone the tests run in.
const SEEN = 1_710_072_000;

const findingOf = (spec: Spec): Finding => ({
  category: spec.category,
  danger: spec.danger,
  id: spec.id,
  key: { identity: `${spec.kind}/${spec.name}`, module: 'unused-toolchains' },
  module: 'unused-toolchains',
  reason: {
    args: {
      command: text(spec.command),
      date: { kind: 'date', value: SEEN },
      size: { kind: 'size', value: spec.bytes },
    },
    key: `unused-toolchains.reason.${spec.kind}`,
  },
  removability: 'removable',
  removal: { kind: 'move-to-trash', value: `${HOME}/${spec.path}` },
  sizes: sized(spec.bytes),
  subject: { kind: 'path', value: `${HOME}/${spec.path}` },
  title: { args: { name: text(spec.name) }, key: `unused-toolchains.title.${spec.kind}` },
});

const RUST = 'unused-toolchains.rust';
const NODE = 'unused-toolchains.node';

const TOOLCHAINS = '.rustup/toolchains';

export const UNUSED_TOOLCHAINS_FINDINGS: readonly Finding[] = [
  findingOf({
    bytes: 1_200 * MB,
    category: RUST,
    command: 'rustup toolchain install 1.70.0-aarch64-apple-darwin',
    danger: 'low',
    id: 601,
    kind: 'rust',
    name: '1.70.0-aarch64-apple-darwin',
    path: `${TOOLCHAINS}/1.70.0-aarch64-apple-darwin`,
  }),
  findingOf({
    bytes: 1_500 * MB,
    category: RUST,
    command: 'rustup toolchain install nightly-2023-06-01-aarch64-apple-darwin',
    danger: 'low',
    id: 602,
    kind: 'rust',
    name: 'nightly-2023-06-01-aarch64-apple-darwin',
    path: `${TOOLCHAINS}/nightly-2023-06-01-aarch64-apple-darwin`,
  }),
  findingOf({
    bytes: 90 * MB,
    category: NODE,
    command: 'nvm install 12.22.12',
    danger: 'medium',
    id: 603,
    kind: 'node',
    name: 'v12.22.12',
    path: '.nvm/versions/node/v12.22.12',
  }),
];

const needed = (id: string, danger: Finding['danger'], ready: boolean): ModuleInfo['categories'][number] => ({
  availability: ready ? { kind: 'ready' } : { kind: 'unavailable', missing: [DOCUMENTS] },
  descriptor: {
    danger,
    icon: iconOf(id),
    id,
    needs: [{ capability: DOCUMENTS, kind: 'required' }],
  },
});

const modulesOf = (ready: boolean): readonly ModuleInfo[] => [
  {
    categories: [needed(RUST, 'low', ready), needed(NODE, 'medium', ready)],
    choices: [],
    group: 'development',
    id: 'unused-toolchains',
    settings: ['unused-toolchains.enabled', 'unused-toolchains.node-versions'],
  },
];

// `get_modules`: the module in the group Development, its two categories, its two switches, no choice.
export const UNUSED_TOOLCHAINS_MODULES: readonly ModuleInfo[] = modulesOf(true);

// The same before Documents is allowed: every category needs it and none can be cleaned.
export const UNUSED_TOOLCHAINS_NO_DOCUMENTS_MODULES: readonly ModuleInfo[] = modulesOf(false);

// Another Node version that belongs to another user: no finding of its own, kept for good.
export const UNUSED_TOOLCHAINS_LEFT_OUT: readonly LeftOutCount[] = [
  keptCount(NODE, 'unused-toolchains.left-out.not-yours', 1, sized(110 * MB)),
];

const FINISHED: Event = { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' };

export const UNUSED_TOOLCHAINS_SCAN: readonly Event[] = [
  { modules: ['unused-toolchains'], scan: 1, type: 'scan-started' },
  { findings: UNUSED_TOOLCHAINS_FINDINGS.slice(0, 2), scan: 1, type: 'findings-added' },
  { findings: UNUSED_TOOLCHAINS_FINDINGS.slice(2), scan: 1, type: 'findings-added' },
  {
    insights: [],
    leftOut: [...UNUSED_TOOLCHAINS_LEFT_OUT],
    module: 'unused-toolchains',
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  FINISHED,
];

// The scan before Documents is allowed: the module could not look, and says what it missed.
export const UNUSED_TOOLCHAINS_NO_DOCUMENTS_SCAN: readonly Event[] = [
  { modules: ['unused-toolchains'], scan: 1, type: 'scan-started' },
  {
    insights: [],
    leftOut: [],
    module: 'unused-toolchains',
    outcome: { kind: 'unavailable', missing: [DOCUMENTS] },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  FINISHED,
];

export const UNUSED_TOOLCHAINS_TOTALS: Overview['totals'] = [
  { category: RUST, count: 2, highestDanger: 'low', sizes: sized(2_700 * MB) },
  { category: NODE, count: 1, highestDanger: 'medium', sizes: sized(90 * MB) },
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

export const UNUSED_TOOLCHAINS_OVERVIEW: Overview = overviewOf(2_790 * MB, UNUSED_TOOLCHAINS_TOTALS);

export const UNUSED_TOOLCHAINS_NO_DOCUMENTS_OVERVIEW: Overview = overviewOf(0, []);

const settingsOf = (nodeVersions: boolean): SettingsView => ({
  file: 'loaded',
  problems: [],
  settings: [
    {
      default: { kind: 'switch', value: true },
      effect: 'next-scan',
      group: 'module',
      key: 'unused-toolchains.enabled',
      kind: { kind: 'switch' },
    },
    {
      default: { kind: 'switch', value: false },
      effect: 'next-scan',
      group: 'module',
      key: 'unused-toolchains.node-versions',
      kind: { kind: 'switch' },
    },
  ],
  values: {
    'unused-toolchains.enabled': { kind: 'switch', value: true },
    'unused-toolchains.node-versions': { kind: 'switch', value: nodeVersions },
  },
});

// The module's settings as the scan has them: the user turned the Node switch on, or it is at its default, off.
export const UNUSED_TOOLCHAINS_SETTINGS: SettingsView = settingsOf(true);
export const UNUSED_TOOLCHAINS_DEFAULT_SETTINGS: SettingsView = settingsOf(false);

// What `get_access_status` answers with Documents not yet asked.
export const UNUSED_TOOLCHAINS_NOT_ASKED = accessStatus([
  [{ kind: 'all-files' }, 'granted'],
  [DOCUMENTS, 'not-asked'],
]);

// The core's own texts the page shows beside the module's: the group, Documents' access sentences.
const coreOwn: Dictionary = Object.fromEntries(
  Object.entries(coreTexts).filter(
    ([key]) => key.startsWith('core.access.documents.') || key.startsWith('core.group.development.'),
  ),
);

export const UNUSED_TOOLCHAINS_DICTIONARY: Dictionary = {
  ...SCRIPTS_DICTIONARY,
  ...FINDING_DICTIONARY,
  ...toolchainTexts,
  ...coreOwn,
};
