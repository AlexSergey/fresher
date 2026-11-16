import { describe, expect, it } from 'vitest';

import type { Finding, FindingsPage, FindingsQuery, Overview } from '../ipc/bindings.generated';

import { CommandFailed } from '../ipc';
import { BUSY } from '../test/demo-scan';
import { createFakeClock, type FakeClock } from '../test/fake-clock';
import { largeScan } from '../test/large-scan';
import { pageOf } from '../test/page-of';
import { createFindingsSource, type FindingsSource, type FindingsView } from './findings';
import { type CoreStore, createCoreStore } from './store';

const SCAN = largeScan({ categories: 3, count: 3000 });

const LARGEST: FindingsView = { category: 'demo.large-0', module: 'demo', order: 'largest' };

const FOUND: FindingsView = { ...LARGEST, order: 'found' };

// Every finding of the scan: 30 pages.
const ALL: FindingsView = { category: null, module: 'demo', order: 'largest' };

type Asked = {
  readonly query: FindingsQuery;
  readonly refuse: () => void;
  // Answers the page from the scan's findings, as the engine would.
  readonly resolve: () => void;
};

type Core = {
  readonly asked: readonly Asked[];
  readonly getFindings: (query: FindingsQuery) => Promise<FindingsPage>;
};

// `get_findings` answered by hand, so a test sees a request while it is on its way.
const coreByHand = (findings: readonly Finding[] = SCAN.findings): Core => {
  const asked: Asked[] = [];

  return {
    asked,
    getFindings: (query) =>
      new Promise((resolve, reject) => {
        asked.push({
          query,
          refuse: () => {
            reject(new CommandFailed(BUSY));
          },
          resolve: () => {
            resolve(pageOf(findings, query));
          },
        });
      }),
  };
};

// `get_findings` answered at once from the findings.
const coreAnswering =
  (findings: readonly Finding[]) =>
  (query: FindingsQuery): Promise<FindingsPage> =>
    Promise.resolve(pageOf(findings, query));

// The answers and what follows them run as microtasks: a macrotask runs after all of them.
const settle = (): Promise<void> =>
  new Promise((resolve) => {
    setTimeout(resolve, 0);
  });

const answerAll = async (core: Core): Promise<void> => {
  for (const asked of core.asked) {
    asked.resolve();
  }
  await settle();
};

const sourceOf = (
  getFindings: (query: FindingsQuery) => Promise<FindingsPage>,
  clock: FakeClock = createFakeClock(),
): FindingsSource => createFindingsSource(getFindings, { clock, maxPages: 10, pageSize: 100 });

const scanning = (): Overview => ({ ...SCAN.overview, status: { scan: 1, state: 'scanning' } });

// A store that follows scan 1, running.
const storeOfScan = (): CoreStore => {
  const store = createCoreStore();
  store.dispatch({ modules: ['demo'], scan: 1, type: 'scan-started' });

  return store;
};

describe('the findings source', () => {
  describe('negative cases', () => {
    it('does not request a page twice while it loads', () => {
      const core = coreByHand();
      const source = sourceOf(core.getFindings);

      source.ensure(LARGEST, 0, 20);
      source.ensure(LARGEST, 10, 30);

      expect(core.asked).toHaveLength(1);
    });

    it('drops the least recently used page past ten', async () => {
      const source = sourceOf(coreAnswering(SCAN.findings));
      for (let page = 0; page < 10; page += 1) {
        source.ensure(ALL, page * 100, page * 100 + 10);
        await settle();
      }

      source.ensure(ALL, 0, 10);
      source.ensure(ALL, 1000, 1000);
      await settle();

      expect(source.held()).toBe(1000);
      expect(source.rowAt(ALL, 0)).toBeDefined();
      expect(source.rowAt(ALL, 100)).toBeUndefined();
      expect(source.rowAt(ALL, 1000)).toBeDefined();
    });

    it('a new scan drops every page', async () => {
      const core = coreByHand();
      const source = sourceOf(core.getFindings);
      const store = storeOfScan();
      source.follow(store);
      source.ensure(LARGEST, 0, 10);
      await answerAll(core);
      source.ensure(LARGEST, 100, 110);

      store.dispatch({ modules: ['demo'], scan: 2, type: 'scan-started' });
      await answerAll(core);

      expect(source.held()).toBe(0);
      expect(source.rowAt(LARGEST, 0)).toBeUndefined();
      expect(source.total(LARGEST)).toBeUndefined();
    });

    it('a refused page keeps the rows it had and reports once', async () => {
      const core = coreByHand();
      const clock = createFakeClock();
      const source = sourceOf(core.getFindings, clock);
      const store = storeOfScan();
      source.follow(store);
      source.ensure(LARGEST, 0, 10);
      await answerAll(core);
      const first = source.rowAt(LARGEST, 0);
      let changes = 0;
      source.subscribe(() => {
        changes += 1;
      });

      for (let refresh = 0; refresh < 2; refresh += 1) {
        store.dispatch({ overview: scanning(), type: 'overview-loaded' });
        clock.advance(500);
        core.asked.at(-1)?.refuse();
        await settle();
      }

      expect(core.asked).toHaveLength(3);
      expect(source.failure(LARGEST)).toEqual({ kind: 'refused', refusal: BUSY });
      expect(changes).toBe(1);
      expect(source.rowAt(LARGEST, 0)).toBe(first);
    });

    it('a range past the total requests nothing', async () => {
      const core = coreByHand();
      const source = sourceOf(core.getFindings);
      source.ensure(LARGEST, 0, 10);
      await answerAll(core);

      source.ensure(LARGEST, 1000, 1100);
      source.ensure(LARGEST, 20, 10);
      source.ensure(LARGEST, -5, -1);

      expect(source.total(LARGEST)).toBe(1000);
      expect(core.asked).toHaveLength(1);
    });

    it('ignores a refusal of an earlier scan', async () => {
      const core = coreByHand();
      const source = sourceOf(core.getFindings);
      const store = storeOfScan();
      source.follow(store);
      source.ensure(LARGEST, 0, 10);

      store.dispatch({ modules: ['demo'], scan: 2, type: 'scan-started' });
      core.asked[0]?.refuse();
      await settle();
      source.ensure(LARGEST, 0, 10);
      await answerAll(core);

      expect(source.failure(LARGEST)).toBeNull();
      expect(source.rowAt(LARGEST, 0)).toBeDefined();
    });

    it('asks nothing again when the store changes but not its scan or overview', async () => {
      const core = coreByHand();
      const source = sourceOf(core.getFindings);
      const store = storeOfScan();
      source.follow(store);
      source.ensure(LARGEST, 0, 10);
      await answerAll(core);

      store.dispatch({ failure: { kind: 'unreachable' }, type: 'command-failed' });

      expect(core.asked).toHaveLength(1);
    });

    it('stops refreshing once it no longer follows the store', async () => {
      const core = coreByHand();
      const clock = createFakeClock();
      const source = sourceOf(core.getFindings, clock);
      const store = storeOfScan();
      const stop = source.follow(store);
      source.ensure(LARGEST, 0, 10);
      await answerAll(core);
      store.dispatch({ overview: scanning(), type: 'overview-loaded' });
      store.dispatch({ overview: scanning(), type: 'overview-loaded' });

      stop();
      clock.advance(1000);

      expect(core.asked).toHaveLength(2);
    });
  });

  describe('positive cases', () => {
    it('loads the pages a range needs', () => {
      const core = coreByHand();
      const source = sourceOf(core.getFindings);

      source.ensure(LARGEST, 90, 210);

      expect(core.asked.map((asked) => asked.query)).toEqual(
        [0, 100, 200].map((offset) => ({ ...LARGEST, limit: 100, offset })),
      );
    });

    it('rowAt gives undefined until its page arrives', async () => {
      const core = coreByHand();
      const source = sourceOf(core.getFindings);
      source.ensure(LARGEST, 0, 10);

      expect(source.rowAt(LARGEST, 5)).toBeUndefined();
      expect(source.total(LARGEST)).toBeUndefined();
      await answerAll(core);

      expect(source.rowAt(LARGEST, 5)).toEqual(pageOf(SCAN.findings, { ...LARGEST, limit: 1, offset: 5 }).findings[0]);
      expect(source.total(LARGEST)).toBe(1000);
    });

    it('findings removed leave the pages', async () => {
      let findings = SCAN.findings;
      const source = sourceOf((query) => Promise.resolve(pageOf(findings, query)));
      const store = storeOfScan();
      source.follow(store);
      source.ensure(LARGEST, 0, 10);
      await settle();
      const [first, second] = [source.rowAt(LARGEST, 0), source.rowAt(LARGEST, 1)];
      const removed = [first?.id ?? 0];
      findings = findings.filter((finding) => !removed.includes(finding.id));

      store.dispatch({ findings: removed, scan: 1, type: 'findings-removed' });
      await settle();

      expect(source.rowAt(LARGEST, 0)).toEqual(second);
      expect(source.total(LARGEST)).toBe(999);
    });

    it('the pages in view are refreshed at most twice a second during a scan', async () => {
      const core = coreByHand();
      const clock = createFakeClock();
      const source = sourceOf(core.getFindings, clock);
      const store = storeOfScan();
      source.follow(store);
      source.ensure(LARGEST, 0, 10);
      await answerAll(core);

      for (let overview = 0; overview < 10; overview += 1) {
        store.dispatch({ overview: scanning(), type: 'overview-loaded' });
        await answerAll(core);
        clock.advance(100);
      }

      // Refreshed at 0, 500 and 1000 ms.
      expect(core.asked).toHaveLength(4);
      expect(core.asked.map((asked) => asked.query.offset)).toEqual([0, 0, 0, 0]);
    });

    it('asks a page once more when it was refreshed on its way', async () => {
      const core = coreByHand();
      const clock = createFakeClock();
      const source = sourceOf(core.getFindings, clock);
      const store = storeOfScan();
      source.follow(store);
      source.ensure(LARGEST, 0, 10);
      await answerAll(core);
      store.dispatch({ overview: scanning(), type: 'overview-loaded' });
      store.dispatch({ overview: scanning(), type: 'overview-loaded' });

      clock.advance(500);
      expect(core.asked).toHaveLength(2);
      await answerAll(core);

      expect(core.asked).toHaveLength(3);
    });

    it('largest order pages are cached apart from found order pages', async () => {
      const source = sourceOf(coreAnswering(SCAN.findings));

      source.ensure(LARGEST, 0, 10);
      source.ensure(FOUND, 0, 10);
      await settle();

      expect(source.rowAt(FOUND, 0)?.id).toBe(1);
      expect(source.rowAt(LARGEST, 0)?.sizes.allocated).toBe(
        Math.max(
          ...SCAN.findings
            .filter((finding) => finding.category === LARGEST.category)
            .map((finding) => finding.sizes.allocated),
        ),
      );
      expect(source.held()).toBe(200);
    });

    it('scrolls 100,000 findings and never holds more than 1,000', async () => {
      const large = largeScan({ categories: 1, count: 100_000 });
      const source = sourceOf(coreAnswering(large.findings));
      // In the order found: sorting 100,000 findings for each page would only slow the test down.
      const view: FindingsView = { category: null, module: null, order: 'found' };
      let most = 0;
      let seen = 0;

      for (let first = 0; first < 100_000; first += 100) {
        source.ensure(view, first, first + 99);
        await settle();
        most = Math.max(most, source.held());
        seen += source.rowAt(view, first + 99) === undefined ? 0 : 1;
      }

      expect(seen).toBe(1000);
      expect(most).toBe(1000);
      expect(source.total(view)).toBe(100_000);
    });
  });
});
