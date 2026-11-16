import { describe, expect, it, vi } from 'vitest';

import { systemClock } from './clock';

describe('systemClock', () => {
  describe('negative cases', () => {
    it('does not run a cancelled timer', () => {
      vi.useFakeTimers();
      const run = vi.fn();

      systemClock.setTimeout(run, 100)();
      vi.advanceTimersByTime(200);

      expect(run).not.toHaveBeenCalled();
      vi.useRealTimers();
    });
  });

  describe('positive cases', () => {
    it('runs a timer once it falls due', () => {
      vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout', 'performance'] });
      const run = vi.fn();
      const start = systemClock.now();

      systemClock.setTimeout(run, 100);
      vi.advanceTimersByTime(100);

      expect(run).toHaveBeenCalledOnce();
      expect(systemClock.now() - start).toBe(100);
      vi.useRealTimers();
    });
  });
});
