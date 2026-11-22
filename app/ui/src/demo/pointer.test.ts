import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { createPointer, GLIDE_MS } from './pointer';

const element = (): HTMLElement | null => document.querySelector<HTMLElement>('[data-demo-player]');

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
  document.body.replaceChildren();
});

describe('createPointer', () => {
  describe('negative cases', () => {
    it('leaves nothing on the page once removed', () => {
      createPointer(document).remove();

      expect(element()).toBeNull();
    });

    it('does not resolve a glide before its time', async () => {
      const pointer = createPointer(document);
      let done = false;
      void pointer.moveTo(10, 20).then(() => {
        done = true;
      });
      await vi.advanceTimersByTimeAsync(GLIDE_MS - 1);

      expect(done).toBe(false);
    });
  });

  describe('positive cases', () => {
    it('is a marked element that takes no event', () => {
      createPointer(document);

      expect(element()).not.toBeNull();
    });

    it('moves to the point and resolves after the glide', async () => {
      const pointer = createPointer(document);
      const glide = pointer.moveTo(10, 20);
      await vi.advanceTimersByTimeAsync(GLIDE_MS);
      await glide;

      expect(element()?.style.transform).toBe('translate(10px, 20px)');
    });

    it('resolves a glide at once when the transition ends', async () => {
      const pointer = createPointer(document);
      const glide = pointer.moveTo(1, 2);
      element()?.dispatchEvent(new Event('transitionend'));

      await expect(glide).resolves.toBeUndefined();
    });

    it('presses and releases in 160 ms', async () => {
      const pointer = createPointer(document);
      const press = pointer.press();
      await vi.advanceTimersByTimeAsync(160);
      await press;

      expect(element()?.querySelector('svg')?.classList.length).toBe(1);
    });
  });
});
