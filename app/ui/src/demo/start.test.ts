import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { DemoPlay } from '../ipc/bindings.generated';

import { createCoreStore } from '../state/store';

const ipc = vi.hoisted(() => ({
  demoFinished: vi.fn<(outcome: unknown) => Promise<void>>(() => Promise.resolve()),
  demoPlay: vi.fn<() => Promise<DemoPlay | null>>(),
}));
vi.mock('../ipc', () => ipc);

const SCENARIO: DemoPlay = {
  private: [],
  scenario: {
    name: 'x',
    steps: [{ kind: 'pause', ms: 100 }],
    window: { height: 700, width: 1000 },
    world: 'w',
  },
};

const pointer = (): Element | null => document.querySelector('[data-demo-player]');

beforeEach(() => {
  vi.useFakeTimers();
  ipc.demoFinished.mockClear();
});

afterEach(() => {
  vi.useRealTimers();
  vi.resetModules();
  vi.doUnmock('./player');
  delete document.documentElement.dataset.demo;
  document.body.replaceChildren();
});

describe('startDemo', () => {
  describe('negative cases', () => {
    it('does nothing without a scenario', async () => {
      ipc.demoPlay.mockResolvedValue(null);
      const { startDemo } = await import('./start');
      const store = createCoreStore();
      await startDemo(store);

      expect(store.getState().demo).toBe(false);
      expect(pointer()).toBeNull();
      expect(document.documentElement.dataset.demo).toBeUndefined();
      expect(ipc.demoFinished).not.toHaveBeenCalled();
    });

    it('ignores a shell that refuses demo_play', async () => {
      ipc.demoPlay.mockRejectedValue(new Error('unknown command'));
      const { startDemo } = await import('./start');
      await startDemo(createCoreStore());

      expect(ipc.demoFinished).not.toHaveBeenCalled();
    });

    it('reports a play that throws as failed', async () => {
      ipc.demoPlay.mockResolvedValue(SCENARIO);
      vi.doMock('./player', () => ({ play: (): Promise<never> => Promise.reject(new Error('broke')) }));
      const { startDemo } = await import('./start');
      await startDemo(createCoreStore());

      expect(ipc.demoFinished).toHaveBeenCalledWith({ kind: 'failed', reason: 'broke', step: 1 });
      expect(pointer()).toBeNull();
    });

    it('logs a refused outcome instead of rejecting', async () => {
      ipc.demoPlay.mockResolvedValue(SCENARIO);
      ipc.demoFinished.mockRejectedValueOnce(new Error('refused'));
      const logged = vi.spyOn(console, 'error').mockImplementation(() => undefined);
      const { startDemo } = await import('./start');
      const running = startDemo(createCoreStore());
      await vi.advanceTimersByTimeAsync(2000);

      await expect(running).resolves.toBeUndefined();
      expect(logged).toHaveBeenCalledOnce();
      logged.mockRestore();
    });
  });

  describe('positive cases', () => {
    it('plays while data-demo is set, then reports the outcome and removes the pointer', async () => {
      ipc.demoPlay.mockResolvedValue(SCENARIO);
      const { startDemo } = await import('./start');
      const store = createCoreStore();
      const running = startDemo(store);
      await vi.advanceTimersByTimeAsync(1);

      expect(document.documentElement.dataset.demo).toBe('playing');
      expect(store.getState().demo).toBe(true);
      expect(pointer()).not.toBeNull();

      await vi.advanceTimersByTimeAsync(2000);
      await running;

      expect(ipc.demoFinished).toHaveBeenCalledWith({ elapsedMs: expect.any(Number) as number, kind: 'finished' });
      expect(pointer()).toBeNull();
    });
  });
});
