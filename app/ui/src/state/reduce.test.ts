import { describe, expect, it } from 'vitest';

import type { Event, Finding, Overview, ScanId } from '../ipc/bindings.generated';

import { DEMO_DICTIONARY, DEMO_FINDINGS, DEMO_SCAN, IDLE } from '../test/demo-scan';
import { type CoreAction, type CoreState, INITIAL_STATE, reduce } from './reduce';

const run = (actions: readonly CoreAction[], from: CoreState = INITIAL_STATE): CoreState =>
  actions.reduce(reduce, from);

const started: Event = { modules: ['demo'], scan: 1, type: 'scan-started' };

const scanning = (scan: ScanId): Overview => ({ ...IDLE, status: { scan, state: 'scanning' } });

// `count` findings of scan 1, spread over `categories` categories in turn, with ids from 0.
const manyFindings = (count: number, categories: number): readonly Finding[] => {
  const [model] = DEMO_FINDINGS;

  if (model === undefined) {
    throw new Error('The demo scan has no finding');
  }

  return Array.from({ length: count }, (_, id) => ({ ...model, category: `demo.category-${id % categories}`, id }));
};

// The findings in batches of the engine's size (256), as a scan sends them.
const inBatches = (findings: readonly Finding[]): readonly CoreAction[] =>
  Array.from({ length: Math.ceil(findings.length / 256) }, (_, at) => ({
    findings: findings.slice(at * 256, (at + 1) * 256),
    scan: 1,
    type: 'findings-added',
  }));

describe('reduce', () => {
  describe('negative cases', () => {
    it('ignores an event of another scan', () => {
      const state = run([
        started,
        { findings: [...DEMO_FINDINGS], scan: 2, type: 'findings-added' },
        { outcome: { kind: 'completed' }, scan: 2, type: 'scan-finished' },
      ]);

      expect(state.groups).toEqual([]);
      expect(state.status).toEqual({ scan: 1, state: 'scanning' });
    });

    it('keeps findings when the scan is cancelled and marks it partial', () => {
      const state = run([
        started,
        { findings: DEMO_FINDINGS.slice(0, 1), scan: 1, type: 'findings-added' },
        { module: 'demo', outcome: { kind: 'cancelled' }, scan: 1, type: 'module-finished' },
        { outcome: { kind: 'partial' }, scan: 1, type: 'scan-finished' },
      ]);

      expect(state.groups).toEqual([{ category: 'demo.caches', findings: DEMO_FINDINGS.slice(0, 1) }]);
      expect(state.outcome).toEqual({ kind: 'partial' });
      expect(state.modules).toEqual([{ module: 'demo', outcome: { kind: 'cancelled' } }]);
      expect(state.status).toEqual({ state: 'idle' });
    });

    it('keeps a failure until the next scan starts', () => {
      const failed = run([{ failure: { kind: 'unreachable' }, type: 'command-failed' }]);

      expect(failed.failure).toEqual({ kind: 'unreachable' });
      expect(run([started], failed).failure).toBeNull();
    });

    it('keeps a scan the events started when a stale idle overview arrives', () => {
      const state = run([started, { overview: IDLE, type: 'overview-loaded' }]);

      expect(state.status).toEqual({ scan: 1, state: 'scanning' });
      expect(state.scan).toBe(1);
    });

    it('keeps a scan the events finished when a stale overview says it still runs', () => {
      const state = run([
        { bytes: 2048, entries: 10, scan: 4, type: 'scan-progress' },
        { outcome: { kind: 'completed' }, scan: 4, type: 'scan-finished' },
        { overview: scanning(4), type: 'overview-loaded' },
      ]);

      expect(state.status).toEqual({ state: 'idle' });
    });

    it('keeps the scan of the events when the overview reports an older one', () => {
      const state = run([
        { modules: ['demo'], scan: 5, type: 'scan-started' },
        { overview: scanning(4), type: 'overview-loaded' },
      ]);

      expect(state.status).toEqual({ scan: 5, state: 'scanning' });
      expect(state.scan).toBe(5);
    });

    it('leaves the state as it is for the settings and the permissions', () => {
      const state = run([
        { key: 'demo.count', type: 'setting-changed', value: { kind: 'number', value: 3 } },
        { status: { fullDiskAccess: 'granted' }, type: 'access-changed' },
      ]);

      expect(state).toBe(INITIAL_STATE);
    });
  });

  describe('positive cases', () => {
    it('adds findings in batches', () => {
      const state = run(DEMO_SCAN.slice(0, 2));

      expect(state.groups.flatMap((group) => group.findings)).toEqual(DEMO_FINDINGS.slice(0, 2));
      expect(run(DEMO_SCAN.slice(2, 3), state).groups.flatMap((group) => group.findings)).toHaveLength(3);
    });

    it('groups a batch of 20,000 findings in the order they came', () => {
      const findings = manyFindings(20_000, 7);

      const state = run([started, { findings: [...findings], scan: 1, type: 'findings-added' }]);

      expect(state.groups.map((group) => group.category)).toEqual(
        Array.from({ length: 7 }, (_, category) => `demo.category-${category}`),
      );
      expect(state.groups.map((group) => group.findings.length)).toEqual([2858, 2857, 2857, 2857, 2857, 2857, 2857]);
      expect(state.groups[3]?.findings.map((finding) => finding.id).slice(0, 3)).toEqual([3, 10, 17]);
    });

    // The timeout only guards against a quadratic reducer: a linear one takes a few tens of milliseconds.
    it(
      'adds a scan of 50,000 findings in one category without copying the group per finding',
      { timeout: 1000 },
      () => {
        const findings = manyFindings(50_000, 1);

        const state = run([started, ...inBatches(findings)]);

        expect(state.groups).toHaveLength(1);
        expect(state.groups[0]?.findings).toHaveLength(50_000);
        expect(state.groups[0]?.findings.at(-1)).toBe(findings.at(-1));
      },
    );

    it('groups findings by category', () => {
      const state = run(DEMO_SCAN);

      expect(state.groups.map((group) => [group.category, group.findings.map((finding) => finding.id)])).toEqual([
        ['demo.caches', [1, 3]],
        ['demo.downloads', [2]],
      ]);
      expect(state.outcome).toEqual({ kind: 'completed' });
      expect(state.modules).toEqual([{ module: 'demo', outcome: { kind: 'completed' } }]);
    });

    it('starts a new scan from nothing', () => {
      const state = run([...DEMO_SCAN, { modules: ['demo'], scan: 2, type: 'scan-started' }]);

      expect(state.groups).toEqual([]);
      expect(state.outcome).toBeNull();
      expect(state.scan).toBe(2);
    });

    it('keeps the progress, the dictionary and the state of the engine', () => {
      const state = run([
        { dictionary: DEMO_DICTIONARY, type: 'dictionary-loaded' },
        { overview: scanning(4), type: 'overview-loaded' },
        { bytes: 2048, entries: 10, scan: 4, type: 'scan-progress' },
      ]);

      expect(state.dictionary).toBe(DEMO_DICTIONARY);
      expect(state.scan).toBe(4);
      expect(state.progress).toEqual({ bytes: 2048, entries: 10 });
      const cleaning = run([{ overview: { ...IDLE, status: { state: 'cleaning' } }, type: 'overview-loaded' }]);
      expect(cleaning.status).toEqual({ state: 'cleaning' });
    });

    it('follows the scan an overview reports after a reload during that scan', () => {
      const state = run([
        { bytes: 1024, entries: 5, scan: 4, type: 'scan-progress' },
        { overview: scanning(4), type: 'overview-loaded' },
        { findings: DEMO_FINDINGS.slice(0, 1).map((finding) => ({ ...finding })), scan: 4, type: 'findings-added' },
        { outcome: { kind: 'completed' }, scan: 4, type: 'scan-finished' },
      ]);

      expect(state.scan).toBe(4);
      expect(state.groups.flatMap((group) => group.findings)).toEqual(DEMO_FINDINGS.slice(0, 1));
      expect(state.status).toEqual({ state: 'idle' });
      expect(state.outcome).toEqual({ kind: 'completed' });
    });
  });
});
