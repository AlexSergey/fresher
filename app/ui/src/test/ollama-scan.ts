import type { Event, Finding, LeftOutCount, Overview, Sizes, TextArg } from '../ipc/bindings.generated';

import { FULL_COVERAGE } from './demo-scan';
import { OLLAMA } from './local-ai-models-scan';

// Scans of the local AI models module with Ollama's models as the core streams them: three models are offered, each
// one item whose parts go together (the manifests of its tags, then the layers no other model names). The first two
// share a layer, so each names the other and frees less than it lists; the third shares none. The second scan has an
// unfinished download: nothing of Ollama is offered, only the store's left-out line. The texts are the module's own
// (the dictionary of `local-ai-models-scan.ts`). Used by the e2e mock.

const MB = 1_000_000;

const sized = (bytes: number): Sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

const text = (value: string): TextArg => ({ kind: 'text', value });

const STORE = '/Users/someone/.ollama/models';
const LIBRARY = `${STORE}/manifests/registry.ollama.ai/library`;

const layer = (digest: string): string => `${STORE}/blobs/sha256-${digest}`;

// 2024-03-10 12:00 UTC: midday, so the date is Mar 10 in every time zone the tests run in.
const SEEN = 1_710_072_000;

type Spec = {
  // What the model's parts take.
  readonly freed: number;
  readonly id: number;
  readonly identity: string;
  // What a pull downloads, shared layers included.
  readonly listed: number;
  // The tags of the model, first one first.
  readonly name: string;
  // The manifests, then the layers; the first path is the subject.
  readonly parts: readonly [string, ...string[]];
  // The first tag of the other model that names one of its layers; null for a model that shares none.
  readonly sharer: null | string;
  // The first tag.
  readonly tag: string;
};

const reasonOf = (spec: Spec): Finding['reason'] => {
  const date = { kind: 'date', value: SEEN } as const;
  const size = { kind: 'size', value: spec.freed } as const;
  if (spec.sharer === null) {
    return { args: { date, name: text(spec.tag), size }, key: 'local-ai-models.reason.ollama' };
  }

  return {
    args: { date, listed: { kind: 'size', value: spec.listed }, other: text(spec.sharer), size },
    key: 'local-ai-models.reason.ollama-shared',
  };
};

const findingOf = (spec: Spec): Finding => ({
  category: OLLAMA,
  danger: 'medium',
  id: spec.id,
  key: { identity: spec.identity, module: 'local-ai-models' },
  module: 'local-ai-models',
  reason: reasonOf(spec),
  removability: 'removable',
  removal: { kind: 'move-together', value: [...spec.parts] },
  sizes: sized(spec.freed),
  subject: { kind: 'path', value: spec.parts[0] },
  title: { args: { name: text(spec.name) }, key: 'local-ai-models.title' },
});

export const OLLAMA_FINDINGS: readonly Finding[] = [
  // One model with two tags, in name order: two manifests, then two layers.
  findingOf({
    freed: 4_700 * MB,
    id: 511,
    identity: 'ollama/registry.ollama.ai/library/llama3/8b',
    listed: 5_000 * MB,
    name: 'llama3:8b, llama3:latest',
    parts: [`${LIBRARY}/llama3/8b`, `${LIBRARY}/llama3/latest`, layer('a1f0c2'), layer('b7d3e4')],
    sharer: 'llama3-custom:latest',
    tag: 'llama3:8b',
  }),
  // Made from the first: it shares one layer with it.
  findingOf({
    freed: 4_600 * MB,
    id: 512,
    identity: 'ollama/registry.ollama.ai/library/llama3-custom/latest',
    listed: 4_900 * MB,
    name: 'llama3-custom:latest',
    parts: [`${LIBRARY}/llama3-custom/latest`, layer('c9e5f6'), layer('d2a4b8')],
    sharer: 'llama3:8b',
    tag: 'llama3-custom:latest',
  }),
  findingOf({
    freed: 1_600 * MB,
    id: 513,
    identity: 'ollama/registry.ollama.ai/library/gemma2/2b',
    listed: 1_600 * MB,
    name: 'gemma2:2b',
    parts: [`${LIBRARY}/gemma2/2b`, layer('e4b1a7'), layer('f8c6d3')],
    sharer: null,
    tag: 'gemma2:2b',
  }),
];

// The store is left out as a whole: a model is being downloaded.
export const OLLAMA_DOWNLOAD_LEFT_OUT: readonly LeftOutCount[] = [
  {
    category: OLLAMA,
    items: 1,
    reason: { args: {}, key: 'local-ai-models.left-out.ollama-download' },
    sizes: sized(6_300 * MB),
  },
];

const findingsAdded = (findings: readonly Finding[]): Event[] =>
  findings.length === 0 ? [] : [{ findings: [...findings], scan: 1, type: 'findings-added' }];

const scanOf = (findings: readonly Finding[], leftOut: readonly LeftOutCount[]): readonly Event[] => [
  { modules: ['local-ai-models'], scan: 1, type: 'scan-started' },
  ...findingsAdded(findings),
  {
    insights: [],
    leftOut: [...leftOut],
    module: 'local-ai-models',
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' },
];

export const OLLAMA_SCAN: readonly Event[] = scanOf(OLLAMA_FINDINGS, []);

export const OLLAMA_DOWNLOAD_SCAN: readonly Event[] = scanOf([], OLLAMA_DOWNLOAD_LEFT_OUT);

export const OLLAMA_TOTALS: Overview['totals'] = [
  { category: OLLAMA, count: 3, highestDanger: 'medium', sizes: sized(10_900 * MB) },
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

export const OLLAMA_OVERVIEW: Overview = overviewOf(10_900 * MB, OLLAMA_TOTALS);

export const OLLAMA_DOWNLOAD_OVERVIEW: Overview = overviewOf(0, []);
