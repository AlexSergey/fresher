import type { Dictionary, Event, Insight, ModuleInfo, SettingsView, TextArg } from '../ipc/bindings.generated';

import dockerTexts from '../../../../modules/docker/locales/en.json' with { type: 'json' };
import { FULL_COVERAGE } from './demo-scan';
import {
  XCODE_DICTIONARY,
  XCODE_FINDINGS,
  XCODE_LEFT_OUT,
  XCODE_MODULES,
  XCODE_OVERVIEW,
  XCODE_SETTINGS,
} from './xcode-scan';

// A scan in which the docker module described a runtime and offered nothing, beside the Xcode module's findings so the
// zones are not empty. The insights are what the module sends (fixtures/golden/docker.json and the sandbox's stopped
// case); the texts are the module's own, read from its locale file. Used by the e2e mock.

const text = (value: string): TextArg => ({ kind: 'text', value });
const size = (value: number): TextArg => ({ kind: 'size', value });
const key = (name: string): { args: Record<string, never>; key: string } => ({ args: {}, key: name });
const counted = (name: string, value: number): { args: { count: TextArg }; key: string } => ({
  args: { count: { kind: 'count', value } },
  key: name,
});

const GB = 1_000_000_000;
const MB = 1_000_000;

// OrbStack answered: its disk file is 1 MiB on the disk (a sparse file), 3.59 GB used inside, four parts of which the
// stopped containers (0 B) have no command. The maximum is not shown for OrbStack.
export const ORBSTACK_INSIGHT: Insight = {
  figures: [
    { label: key('docker.figure.on-disk'), size: 1_048_576 },
    { label: key('docker.figure.inside'), size: 3_587_201_090 },
  ],
  identity: 'orbstack',
  notes: [key('docker.runtime.orbstack.gives-back'), key('docker.note.never-runs')],
  parts: [
    {
      command: {
        args: ['--context', 'orbstack', 'builder', 'prune', '--all'],
        line: 'docker --context orbstack builder prune --all',
        tool: 'docker',
      },
      danger: 'safe',
      detail: key('docker.part.build-cache.detail'),
      effect: key('docker.part.build-cache.effect'),
      label: key('docker.part.build-cache.label'),
      size: 512 * MB,
    },
    {
      command: {
        args: ['--context', 'orbstack', 'image', 'prune', '--all'],
        line: 'docker --context orbstack image prune --all',
        tool: 'docker',
      },
      danger: 'low',
      detail: counted('docker.part.images.detail', 5),
      effect: key('docker.part.images.effect'),
      label: key('docker.part.images.label'),
      size: 1_071 * MB,
    },
    {
      command: null,
      danger: 'medium',
      detail: key('docker.part.containers.none'),
      effect: key('docker.part.containers.effect'),
      label: key('docker.part.containers.label'),
      size: 0,
    },
    {
      command: {
        args: ['--context', 'orbstack', 'volume', 'prune'],
        line: 'docker --context orbstack volume prune',
        tool: 'docker',
      },
      danger: 'high',
      detail: counted('docker.part.volumes.detail', 1),
      effect: key('docker.part.volumes.effect'),
      label: key('docker.part.volumes.label'),
      size: 41_200_000,
    },
  ],
  summary: {
    args: { inside: size(3_587_201_090), runtime: text('OrbStack'), unused: size(1_624_200_000) },
    key: 'docker.summary.answered',
  },
  title: { args: { runtime: text('OrbStack') }, key: 'docker.title' },
};

// Docker Desktop is not running: its disk file is seen (the maximum and what it takes), nothing inside is known.
export const DOCKER_DESKTOP_STOPPED_INSIGHT: Insight = {
  figures: [
    { label: key('docker.figure.maximum'), size: 64 * GB },
    { label: key('docker.figure.on-disk'), size: 4_194_304 },
  ],
  identity: 'docker-desktop',
  notes: [key('docker.runtime.docker-desktop.gives-back')],
  parts: [],
  summary: { args: { runtime: text('Docker Desktop') }, key: 'docker.summary.stopped' },
  title: { args: { runtime: text('Docker Desktop') }, key: 'docker.title' },
};

// `get_modules`: Xcode's, then the docker module in the group Development with no category and its switch.
export const DOCKER_MODULES: readonly ModuleInfo[] = [
  ...XCODE_MODULES,
  { categories: [], choices: [], group: 'development', id: 'docker', settings: ['docker.enabled'] },
];

const scanOf = (insight: Insight): readonly Event[] => [
  { modules: ['xcode', 'docker'], scan: 1, type: 'scan-started' },
  { findings: XCODE_FINDINGS.slice(0, 3), scan: 1, type: 'findings-added' },
  { findings: XCODE_FINDINGS.slice(3), scan: 1, type: 'findings-added' },
  {
    insights: [],
    leftOut: [...XCODE_LEFT_OUT],
    module: 'xcode',
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  },
  {
    insights: [insight],
    leftOut: [],
    module: 'docker',
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' },
];

export const DOCKER_SCAN: readonly Event[] = scanOf(ORBSTACK_INSIGHT);
export const DOCKER_STOPPED_SCAN: readonly Event[] = scanOf(DOCKER_DESKTOP_STOPPED_INSIGHT);

// Docker adds nothing to what is offered: the totals and the overview are Xcode's.
export const DOCKER_OVERVIEW = XCODE_OVERVIEW;

export const DOCKER_SETTINGS: SettingsView = {
  ...XCODE_SETTINGS,
  settings: [
    ...XCODE_SETTINGS.settings,
    {
      default: { kind: 'switch', value: true },
      effect: 'next-scan',
      group: 'module',
      key: 'docker.enabled',
      kind: { kind: 'switch' },
    },
  ],
  values: { ...XCODE_SETTINGS.values, 'docker.enabled': { kind: 'switch', value: true } },
};

export const DOCKER_DICTIONARY: Dictionary = { ...XCODE_DICTIONARY, ...dockerTexts };
