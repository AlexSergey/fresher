import { afterEach, describe, expect, it, vi } from 'vitest';

import type { DemoPlay, Step } from '../ipc/bindings.generated';
import type { PlayerDeps } from './player';

import { createCoreStore } from '../state/store';
import { LEAD_IN_MS, play } from './player';

const clock = { time: 0 };
const deps = (): PlayerDeps => {
  clock.time = 0;

  return {
    now: () => clock.time,
    pointer: { moveTo: vi.fn(() => Promise.resolve()), press: vi.fn(() => Promise.resolve()), remove: vi.fn() },
    root: document.body,
    sleep: (ms): Promise<void> => {
      clock.time += ms;

      return Promise.resolve();
    },
    store: createCoreStore(),
  };
};

const scenario = (steps: Step[], secrets: string[] = []): DemoPlay => ({
  private: secrets,
  scenario: { name: 'x', steps, window: { height: 700, width: 1000 }, world: 'w' },
});

const pause = (ms: number): Step => ({ kind: 'pause', ms });

afterEach(() => {
  document.body.replaceChildren();
});

describe('play', () => {
  describe('negative cases', () => {
    it('fails at the step after which the page shows a private path', async () => {
      document.body.textContent = 'Cache in /Users/someone/Library';
      const outcome = await play(scenario([pause(10)], ['/Users/someone']), deps());

      expect(outcome).toEqual({ kind: 'failed', reason: 'the page shows a private path after step 1', step: 1 });
    });

    it('reports the number of a failing second step', async () => {
      const outcome = await play(scenario([pause(10), { kind: 'click', target: missing }]), deps());

      expect(outcome).toMatchObject({ kind: 'failed', step: 2 });
    });

    it('reports a step that throws as failed at that step', async () => {
      const throwing = deps();
      const outcome = await play(scenario([pause(10)]), {
        ...throwing,
        sleep: (ms) => (ms === LEAD_IN_MS ? Promise.resolve() : Promise.reject(new Error('boom'))),
      });

      expect(outcome).toEqual({ kind: 'failed', reason: 'boom', step: 1 });
    });

    it('fails on the /tmp alias of a private /private/tmp path', async () => {
      document.body.textContent = 'Cache in /tmp/fresher-demo/x-1/Library';
      const outcome = await play(scenario([pause(10)], ['/private/tmp/fresher-demo/x-1']), deps());

      expect(outcome).toMatchObject({ kind: 'failed', step: 1 });
    });

    it('does not take an empty private text for a match', async () => {
      document.body.textContent = 'anything';

      expect((await play(scenario([pause(10)], ['']), deps())).kind).toBe('finished');
    });
  });

  describe('positive cases', () => {
    it('reports a played scenario as finished, timed from the end of the lead-in', async () => {
      const outcome = await play(scenario([pause(300), pause(200)], ['/Users/someone']), deps());

      expect(outcome).toEqual({ elapsedMs: 500, kind: 'finished' });
    });
  });
});

const missing = { name: { kind: 'exact', text: 'Nope' }, role: 'button' } as const;
