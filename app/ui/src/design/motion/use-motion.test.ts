import { renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { readMotion, useMotion } from '.';
import { fakeReducedMotion, setMotion } from '../../test/animation';

afterEach(async () => {
  await setMotion(null);
});

describe('useMotion', () => {
  describe('negative cases', () => {
    it('an unknown data-motion value is full', async () => {
      fakeReducedMotion(false);
      await setMotion('fast');

      const { result } = renderHook(() => useMotion());

      expect(result.current).toBe('full');
    });

    it('an unknown data-motion value leaves the system setting in charge', async () => {
      fakeReducedMotion(true);
      await setMotion('fast');

      expect(readMotion()).toBe('reduced');
    });
  });

  describe('positive cases', () => {
    it('is full by default', () => {
      fakeReducedMotion(false);

      expect(readMotion()).toBe('full');
    });

    it('follows the media query and the attribute', async () => {
      const system = fakeReducedMotion(false);
      const { result } = renderHook(() => useMotion());

      system.change(true);

      expect(result.current).toBe('reduced');

      await setMotion('off');

      expect(result.current).toBe('off');

      await setMotion('full');

      expect(result.current).toBe('full');

      await setMotion(null);

      expect(result.current).toBe('reduced');
    });

    it('stops listening when unmounted', () => {
      const system = fakeReducedMotion(false);
      const { unmount } = renderHook(() => useMotion());

      expect(system.listeners()).toBe(1);

      unmount();

      expect(system.listeners()).toBe(0);
    });
  });
});
