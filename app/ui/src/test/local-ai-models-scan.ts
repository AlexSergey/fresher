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

import aiTexts from '../../../../modules/local-ai-models/locales/en.json' with { type: 'json' };
import { FULL_COVERAGE, SCRIPTS_DICTIONARY } from './demo-scan';
import { FINDING_DICTIONARY } from './finding-fixtures';

// A scan of the local AI models module as the core streams it: two Hugging Face models and a dataset, an LM Studio
// model are offered; one more Hugging Face model is left out (its folder is not the user's). The second scan has
// LM Studio open: every LM Studio model is left out, so that category has a left-out line and no finding. An item is
// offered or left out, never both. The texts are the module's own, read from its locale file, so a changed text
// changes the tests. Used by the e2e mock.

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
  readonly folder: string;
  readonly id: number;
  readonly name: string;
  readonly path: string;
  readonly row: 'hugging-face' | 'lm-studio';
};

// 2024-03-10 12:00 UTC: midday, so the date is Mar 10 in every time zone the tests run in.
const SEEN = 1_710_072_000;

const findingOf = (spec: Spec): Finding => ({
  category: spec.category,
  danger: spec.danger,
  id: spec.id,
  key: { identity: `${spec.row}/${spec.folder}`, module: 'local-ai-models' },
  module: 'local-ai-models',
  reason: {
    args: { date: { kind: 'date', value: SEEN }, size: { kind: 'size', value: spec.bytes } },
    key: `local-ai-models.reason.${spec.row}`,
  },
  removability: 'removable',
  removal: { kind: 'move-to-trash', value: `${HOME}/${spec.path}` },
  sizes: sized(spec.bytes),
  subject: { kind: 'path', value: `${HOME}/${spec.path}` },
  title: { args: { name: text(spec.name) }, key: 'local-ai-models.title' },
});

const HUB = '.cache/huggingface/hub';

const HUGGING_FACE_MODELS = 'local-ai-models.hugging-face-models';
const HUGGING_FACE_DATASETS = 'local-ai-models.hugging-face-datasets';
const LM_STUDIO = 'local-ai-models.lm-studio';

const HUGGING_FACE_FINDINGS: readonly Finding[] = [
  findingOf({
    bytes: 2_000 * MB,
    category: HUGGING_FACE_MODELS,
    danger: 'low',
    folder: 'models--meta--tiny',
    id: 501,
    name: 'meta/tiny',
    path: `${HUB}/models--meta--tiny`,
    row: 'hugging-face',
  }),
  findingOf({
    bytes: 548 * MB,
    category: HUGGING_FACE_MODELS,
    danger: 'low',
    folder: 'models--gpt2',
    id: 502,
    name: 'gpt2',
    path: `${HUB}/models--gpt2`,
    row: 'hugging-face',
  }),
  findingOf({
    bytes: 1_200 * MB,
    category: HUGGING_FACE_DATASETS,
    danger: 'low',
    folder: 'datasets--o--words',
    id: 503,
    name: 'o/words',
    path: `${HUB}/datasets--o--words`,
    row: 'hugging-face',
  }),
];

const LM_STUDIO_MODEL: Finding = findingOf({
  bytes: 4_700 * MB,
  category: LM_STUDIO,
  danger: 'medium',
  folder: 'pub/model-a',
  id: 504,
  name: 'pub/model-a',
  path: '.lmstudio/models/pub/model-a',
  row: 'lm-studio',
});

export const LOCAL_AI_MODELS_FINDINGS: readonly Finding[] = [...HUGGING_FACE_FINDINGS, LM_STUDIO_MODEL];

// The same scan with LM Studio open: its model is not offered.
export const LOCAL_AI_MODELS_LM_STUDIO_OPEN_FINDINGS: readonly Finding[] = HUGGING_FACE_FINDINGS;

const readyCategory = (id: string, danger: Finding['danger']): ModuleInfo['categories'][number] => ({
  availability: { kind: 'ready' },
  descriptor: { danger, id, needs: [] },
});

// `get_modules`: the module in the group Development, its three categories, its switch, no choice.
export const LOCAL_AI_MODELS_MODULES: readonly ModuleInfo[] = [
  {
    categories: [
      readyCategory(HUGGING_FACE_MODELS, 'low'),
      readyCategory(HUGGING_FACE_DATASETS, 'low'),
      readyCategory(LM_STUDIO, 'medium'),
    ],
    choices: [],
    group: 'development',
    id: 'local-ai-models',
    settings: ['local-ai-models.enabled'],
  },
];

const left = (category: string, key: string, items: number, bytes: number): LeftOutCount => ({
  category,
  items,
  reason: { args: {}, key: `local-ai-models.left-out.${key}` },
  sizes: sized(bytes),
});

// A Hugging Face model whose folder belongs to another user.
export const LOCAL_AI_MODELS_LEFT_OUT: readonly LeftOutCount[] = [left(HUGGING_FACE_MODELS, 'not-yours', 1, 300 * MB)];

// LM Studio is open: its one model is kept.
export const LOCAL_AI_MODELS_LM_STUDIO_OPEN_LEFT_OUT: readonly LeftOutCount[] = [
  left(HUGGING_FACE_MODELS, 'not-yours', 1, 300 * MB),
  left(LM_STUDIO, 'app-open', 1, 4_700 * MB),
];

const scanOf = (findings: readonly Finding[], leftOut: readonly LeftOutCount[]): readonly Event[] => [
  { modules: ['local-ai-models'], scan: 1, type: 'scan-started' },
  { findings: findings.slice(0, 2), scan: 1, type: 'findings-added' },
  { findings: findings.slice(2), scan: 1, type: 'findings-added' },
  {
    leftOut: [...leftOut],
    module: 'local-ai-models',
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' },
];

export const LOCAL_AI_MODELS_SCAN: readonly Event[] = scanOf(LOCAL_AI_MODELS_FINDINGS, LOCAL_AI_MODELS_LEFT_OUT);

export const LOCAL_AI_MODELS_LM_STUDIO_OPEN_SCAN: readonly Event[] = scanOf(
  LOCAL_AI_MODELS_LM_STUDIO_OPEN_FINDINGS,
  LOCAL_AI_MODELS_LM_STUDIO_OPEN_LEFT_OUT,
);

const HUGGING_FACE_TOTALS: Overview['totals'] = [
  { category: HUGGING_FACE_MODELS, count: 2, highestDanger: 'low', sizes: sized(2_548 * MB) },
  { category: HUGGING_FACE_DATASETS, count: 1, highestDanger: 'low', sizes: sized(1_200 * MB) },
];

export const LOCAL_AI_MODELS_TOTALS: Overview['totals'] = [
  ...HUGGING_FACE_TOTALS,
  { category: LM_STUDIO, count: 1, highestDanger: 'medium', sizes: sized(4_700 * MB) },
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

export const LOCAL_AI_MODELS_OVERVIEW: Overview = overviewOf(8_448 * MB, LOCAL_AI_MODELS_TOTALS);

export const LOCAL_AI_MODELS_LM_STUDIO_OPEN_OVERVIEW: Overview = overviewOf(3_748 * MB, HUGGING_FACE_TOTALS);

// The module's settings: its switch.
export const LOCAL_AI_MODELS_SETTINGS: SettingsView = {
  file: 'loaded',
  problems: [],
  settings: [
    {
      default: { kind: 'switch', value: true },
      effect: 'next-scan',
      group: 'module',
      key: 'local-ai-models.enabled',
      kind: { kind: 'switch' },
    },
  ],
  values: { 'local-ai-models.enabled': { kind: 'switch', value: true } },
};

export const LOCAL_AI_MODELS_DICTIONARY: Dictionary = {
  ...SCRIPTS_DICTIONARY,
  ...FINDING_DICTIONARY,
  ...aiTexts,
};
