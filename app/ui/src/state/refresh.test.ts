import { describe, expect, it, vi } from 'vitest';

import type { Event, Overview } from '../ipc/bindings.generated';

import { mockCore, refuse } from '../ipc/mock';
import { BUSY, DEMO_OVERVIEW, FIRST_BATCH, SCAN_STARTED } from '../test/demo-scan';
import { createFakeClock, type FakeClock } from '../test/fake-clock';
import { createOverviewRefresher } from './refresh';
import { type CoreStore, createCoreStore } from './store';

const SCANNING: Overview = { ...DEMO_OVERVIEW, status: { scan: 1, state: 'scanning' } };

// Another batch of the scan; what it holds does not matter to the refresher.
const MORE: Event = { findings: [], scan: 1, type: 'findings-added' };

type Core = {
  // Answers the oldest request still waiting.
  readonly answer: (overview: Overview) => Promise<void>;
  readonly requests: () => number;
};

// `get_overview` answered by hand, so a test sees a request while it is pending.
const coreAnsweringByHand = (): Core => {
  const waiting: ((overview: Overview) => void)[] = [];
  let requests = 0;
  mockCore({
    getOverview: () =>
      new Promise<Overview>((resolve) => {
        requests += 1;
        waiting.push(resolve);
      }),
  });

  return {
    answer: async (overview): Promise<void> => {
      waiting.shift()?.(overview);
      // The answer and the refresher's `finally` run as microtasks: a macrotask runs after all of them.
      await new Promise((resolve) => {
        setTimeout(resolve, 0);
      });
    },
    requests: () => requests,
  };
};

const refreshed = (): { readonly clock: FakeClock; readonly stop: () => void; readonly store: CoreStore } => {
  const store = createCoreStore();
  const clock = createFakeClock();
  const stop = createOverviewRefresher(store, { clock, interval: 250 });

  return { clock, stop, store };
};

describe('the overview refresher', () => {
  describe('negative cases', () => {
    it('asks nothing while the overview is current', () => {
      const core = coreAnsweringByHand();
      const { clock, store } = refreshed();

      store.dispatch(SCAN_STARTED);
      clock.advance(1000);

      expect(core.requests()).toBe(0);
    });

    it('no second overview request while one is pending', async () => {
      const core = coreAnsweringByHand();
      const { clock, store } = refreshed();
      store.dispatch(SCAN_STARTED);

      store.dispatch(FIRST_BATCH);
      store.dispatch(MORE);
      clock.advance(1000);

      expect(core.requests()).toBe(1);
      expect(clock.waiting()).toBe(0);
      await core.answer(SCANNING);
      expect(core.requests()).toBe(2);
    });

    it('asks at most once per interval', async () => {
      const core = coreAnsweringByHand();
      const { clock, store } = refreshed();
      store.dispatch(SCAN_STARTED);
      store.dispatch(FIRST_BATCH);
      await core.answer(SCANNING);

      store.dispatch(MORE);
      clock.advance(249);

      expect(core.requests()).toBe(1);
      clock.advance(1);
      expect(core.requests()).toBe(2);
    });

    it('keeps a refused overview as the failure and asks no more', async () => {
      mockCore({ getOverview: refuse(BUSY) });
      const { clock, store } = refreshed();
      store.dispatch(SCAN_STARTED);

      store.dispatch(FIRST_BATCH);
      await vi.waitFor(() => {
        expect(store.getState().failure).toEqual({ kind: 'refused', refusal: BUSY });
      });
      clock.advance(1000);

      expect(store.getState().stale).toBe(false);
    });

    it('asks nothing once stopped', async () => {
      const core = coreAnsweringByHand();
      const { clock, stop, store } = refreshed();
      store.dispatch(SCAN_STARTED);
      store.dispatch(FIRST_BATCH);
      store.dispatch(MORE);

      stop();
      await core.answer(SCANNING);
      store.dispatch(MORE);
      clock.advance(1000);

      expect(core.requests()).toBe(1);
    });
  });

  describe('positive cases', () => {
    it('findings mark the overview stale and one refresh follows within 250 ms', async () => {
      const core = coreAnsweringByHand();
      const { clock, store } = refreshed();
      store.dispatch(SCAN_STARTED);
      store.dispatch(FIRST_BATCH);
      await core.answer(SCANNING);

      for (let batch = 0; batch < 10; batch += 1) {
        store.dispatch(MORE);
        clock.advance(20);
      }
      clock.advance(50);

      expect(core.requests()).toBe(2);
      await core.answer(SCANNING);
      expect(store.getState().overview).toBe(SCANNING);
      expect(store.getState().stale).toBe(false);
    });

    it('scan-finished always refreshes once more', async () => {
      const core = coreAnsweringByHand();
      const { clock, store } = refreshed();
      store.dispatch(SCAN_STARTED);
      store.dispatch(FIRST_BATCH);

      store.dispatch({ kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' });
      await core.answer(SCANNING);
      clock.advance(250);
      await core.answer(DEMO_OVERVIEW);

      expect(core.requests()).toBe(2);
      expect(store.getState().overview).toBe(DEMO_OVERVIEW);
    });
  });
});
