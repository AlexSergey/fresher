import { afterEach, describe, expect, it, vi } from 'vitest';

import { sandboxBuild } from '../test/sandbox-build';
import { systemWallClock, wallClockOf } from './wall-clock';

const PAGE_MS = 1_795_600_000_000;

afterEach(() => {
  vi.useRealTimers();
});

describe('wallClockOf', () => {
  describe('negative cases', () => {
    it.each([{ kind: 'release' }, { kind: 'development' }] as const)(
      'keeps the page clock for a $kind build, whatever the core says',
      (build) => {
        expect(wallClockOf({ build, now: 1_700_000_000 }, PAGE_MS)).toBe(systemWallClock);
      },
    );
  });

  describe('positive cases', () => {
    it("moves the page clock to a sandbox world's clock, and keeps it running", () => {
      vi.useFakeTimers({ now: PAGE_MS });
      const world = 1_795_000_000;

      const clock = wallClockOf({ build: sandboxBuild(), now: world }, PAGE_MS);

      expect(clock.epochMs()).toBe(world * 1000);
      vi.advanceTimersByTime(90_000);
      expect(clock.epochMs()).toBe(world * 1000 + 90_000);
    });
  });
});
