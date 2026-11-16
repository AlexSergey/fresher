import { renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { useAnimatedNumber } from '.';
import { fakeFrames, fakeReducedMotion, setMotion } from '../../test/animation';

const renderNumber = (target: number): ReturnType<typeof renderHook<number, { target: number }>> =>
  renderHook(({ target: value }) => useAnimatedNumber(value, { duration: 160 }), { initialProps: { target } });

beforeEach(() => {
  fakeReducedMotion(false);
});

afterEach(async () => {
  vi.useRealTimers();
  await setMotion(null);
});

describe('useAnimatedNumber', () => {
  describe('negative cases', () => {
    it('off returns the target at once', async () => {
      await setMotion('off');
      const frames = fakeFrames();
      const { rerender, result } = renderNumber(0);

      rerender({ target: 1000 });

      expect(result.current).toBe(1000);

      frames.advance(16);

      expect(result.current).toBe(1000);
    });

    it('reduced returns the target at once', async () => {
      await setMotion('reduced');
      fakeFrames();
      const { rerender, result } = renderNumber(0);

      rerender({ target: 1000 });

      expect(result.current).toBe(1000);
    });

    it('a target change mid-tween starts from the shown value, not from the old start', () => {
      const frames = fakeFrames();
      const { rerender, result } = renderNumber(0);

      rerender({ target: 1000 });
      // The first frame starts the clock; four more are half of 160 ms.
      frames.advance(16 + 80);
      const midway = result.current;

      expect(midway).toBeGreaterThan(0);
      expect(midway).toBeLessThan(1000);

      rerender({ target: 2000 });
      frames.advance(16);

      expect(result.current).toBe(midway);

      frames.advance(16);

      expect(result.current).toBeGreaterThan(midway);
    });

    it('does not move when the target stays', () => {
      const frames = fakeFrames();
      const { rerender, result } = renderNumber(500);

      rerender({ target: 500 });
      frames.advance(160);

      expect(result.current).toBe(500);
    });
  });

  describe('positive cases', () => {
    it('shows the first target without counting', () => {
      fakeFrames();
      const { result } = renderNumber(42);

      expect(result.current).toBe(42);
    });

    it('reaches the target after the duration', () => {
      const frames = fakeFrames();
      const { rerender, result } = renderNumber(0);

      rerender({ target: 1000 });
      frames.advance(16);

      expect(result.current).toBe(0);

      frames.advance(160);

      expect(result.current).toBe(1000);
    });

    it('stops its frames when unmounted', () => {
      const frames = fakeFrames();
      const { rerender, unmount } = renderNumber(0);

      rerender({ target: 1000 });
      unmount();
      frames.advance(160);

      expect(vi.getTimerCount()).toBe(0);
    });
  });
});
