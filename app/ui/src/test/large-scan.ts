import type {
  CategoryTotal,
  DangerLevel,
  Dictionary,
  Event,
  Finding,
  Overview,
  Sizes,
} from '../ipc/bindings.generated';

import { DEMO_DICTIONARY, FULL_COVERAGE } from './demo-scan';

// A scan of many findings of the demo module, the same on every run: for the unit tests, the e2e mock and the
// performance spec. Sizes spread over four orders of magnitude (10 kB to 100 MB), every danger level appears.

export type LargeScan = {
  readonly dictionary: Dictionary;
  // As the core streams them: started, batches of 256 findings, the module and the scan finished.
  readonly events: readonly Event[];
  // In the order the scan found them, ids from 1.
  readonly findings: readonly Finding[];
  // What `get_overview` answers once the scan finished.
  readonly overview: Overview;
};

const DANGERS: readonly DangerLevel[] = ['safe', 'low', 'medium', 'high'];

const BATCH = 256;

// A small linear congruential generator: the same sequence for the same seed.
const sequence = (seed: number): (() => number) => {
  let state = seed;

  return (): number => {
    state = (Math.imul(state, 1_664_525) + 1_013_904_223) >>> 0;

    return state;
  };
};

const complete = (bytes: number): Sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

const add = (a: Sizes, b: Sizes): Sizes => complete(a.allocated + b.allocated);

const findingAt = (id: number, category: string, bytes: number, danger: DangerLevel): Finding => {
  const path = `/demo/large/${id}`;

  return {
    category,
    danger,
    id,
    key: { identity: path, module: 'demo' },
    module: 'demo',
    reason: { args: {}, key: 'demo.finding.reason' },
    removability: 'removable',
    removal: { kind: 'move-to-trash', value: path },
    sizes: complete(bytes),
    subject: { kind: 'path', value: path },
    title: { args: { name: { kind: 'text', value: `item-${id}` } }, key: 'demo.finding.title' },
  };
};

const totalsOf = (findings: readonly Finding[], categories: readonly string[]): readonly CategoryTotal[] =>
  categories.flatMap((category) => {
    const own = findings.filter((finding) => finding.category === category);
    const [first] = own;

    if (first === undefined) {
      return [];
    }

    return [
      {
        category,
        count: own.length,
        highestDanger: own.reduce(
          (highest, finding) => (DANGERS.indexOf(finding.danger) > DANGERS.indexOf(highest) ? finding.danger : highest),
          first.danger,
        ),
        sizes: own.reduce((sum, finding) => add(sum, finding.sizes), complete(0)),
      },
    ];
  });

// What `get_overview` answers while a scan has streamed `findings` so far: their totals and their sum, still scanning.
export const scanningOverviewOf = (findings: readonly Finding[]): Overview => ({
  found: findings.reduce((sum, finding) => add(sum, finding.sizes), complete(0)),
  ignored: { categories: 0, paths: 0 },
  lastScan: null,
  status: { scan: 1, state: 'scanning' },
  totals: [...totalsOf(findings, [...new Set(findings.map(({ category }) => category))])],
});

// `count` findings spread over `categories` categories (`demo.large-0`, `demo.large-1`, …) in turn.
export const largeScan = ({
  categories,
  count,
}: {
  readonly categories: number;
  readonly count: number;
}): LargeScan => {
  const next = sequence(count);
  const ids = Array.from({ length: categories }, (_, at) => `demo.large-${at}`);
  const findings = Array.from({ length: count }, (_, at) => {
    const magnitude = 10 ** (4 + (next() % 4));
    const bytes = magnitude + (next() % (magnitude * 9));

    return findingAt(at + 1, ids[at % categories] ?? 'demo.large-0', bytes, DANGERS[next() % DANGERS.length] ?? 'safe');
  });
  const batches = Array.from({ length: Math.ceil(count / BATCH) }, (_, at): Event => ({
    findings: findings.slice(at * BATCH, (at + 1) * BATCH),
    scan: 1,
    type: 'findings-added',
  }));

  return {
    dictionary: {
      ...DEMO_DICTIONARY,
      'core.danger.high.name': 'High',
      'core.danger.medium.name': 'Medium',
      ...Object.fromEntries(ids.map((id, at) => [`${id}.name`, `Example group ${at + 1}`])),
    },
    events: [
      { modules: ['demo'], scan: 1, type: 'scan-started' },
      ...batches,
      { leftOut: [], module: 'demo', outcome: { kind: 'completed' }, scan: 1, type: 'module-finished' },
      { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' },
    ],
    findings,
    overview: {
      found: findings.reduce((sum, finding) => add(sum, finding.sizes), complete(0)),
      ignored: { categories: 0, paths: 0 },
      lastScan: {
        coverage: FULL_COVERAGE,
        finished: 1_735_689_600,
        kind: { kind: 'full' },
        outcome: { kind: 'completed' },
        scan: 1,
        settingsChanged: false,
      },
      status: { state: 'idle' },
      totals: [...totalsOf(findings, ids)],
    },
  };
};
