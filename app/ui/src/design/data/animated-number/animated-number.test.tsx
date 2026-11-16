import { render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { fakeFrames, fakeReducedMotion, setMotion } from '../../../test/animation';
import { DURATION } from '../../tokens/motion';
import { AnimatedNumber } from './animated-number';

const format = (value: number): string => `${Math.round(value)} GB`;

// The counting copy on screen, hidden from assistive technology.
// eslint-disable-next-line testing-library/no-node-access -- the copy has no role, by design
const counting = (): Element => document.querySelector('[data-numeric] > [aria-hidden]') as Element;

const shown = (): string => counting().textContent;

const renderNumber = (value: number, duration?: 'deliberate' | 'slow'): ReturnType<typeof render> =>
  render(<AnimatedNumber format={format} value={value} {...(duration === undefined ? {} : { duration })} />);

beforeEach(() => {
  fakeReducedMotion(false);
});

afterEach(async () => {
  vi.useRealTimers();
  await setMotion(null);
});

describe('AnimatedNumber', () => {
  describe('negative cases', () => {
    it('a falling value does not count down', () => {
      const frames = fakeFrames();
      const { rerender } = renderNumber(900);

      rerender(<AnimatedNumber format={format} value={100} />);

      expect(shown()).toBe('100 GB');

      frames.advance(DURATION.slow);

      expect(shown()).toBe('100 GB');
    });

    it('the counting text is hidden from assistive technology', () => {
      fakeFrames();
      const { rerender } = renderNumber(0);

      rerender(<AnimatedNumber format={format} value={500} />);

      expect(counting()).toHaveAttribute('aria-hidden', 'true');
      expect(screen.getByText('500 GB')).toHaveClass('u-visually-hidden');
    });

    it('motion off shows the value at once', async () => {
      await setMotion('off');
      fakeFrames();
      const { rerender } = renderNumber(0);

      rerender(<AnimatedNumber format={format} value={500} />);

      expect(shown()).toBe('500 GB');
    });
  });

  describe('positive cases', () => {
    it('the number reaches its value after the duration', () => {
      const frames = fakeFrames();
      const { rerender } = renderNumber(0);

      rerender(<AnimatedNumber format={format} value={500} />);
      frames.advance(16 + DURATION.slow / 2);

      expect(shown()).not.toBe('500 GB');
      expect(shown()).not.toBe('0 GB');

      frames.advance(DURATION.slow);

      expect(shown()).toBe('500 GB');
    });

    it('takes the deliberate duration when asked', () => {
      const frames = fakeFrames();
      const { rerender } = renderNumber(0, 'deliberate');

      rerender(<AnimatedNumber duration="deliberate" format={format} value={500} />);
      frames.advance(16 + DURATION.slow + 16);

      expect(shown()).not.toBe('500 GB');

      frames.advance(DURATION.deliberate);

      expect(shown()).toBe('500 GB');
    });

    it('counts up again after a fall', () => {
      const frames = fakeFrames();
      const { rerender } = renderNumber(900);

      rerender(<AnimatedNumber format={format} value={100} />);
      rerender(<AnimatedNumber format={format} value={300} />);
      frames.advance(16 + DURATION.slow / 2);

      expect(shown()).not.toBe('300 GB');

      frames.advance(DURATION.slow);

      expect(shown()).toBe('300 GB');
    });
  });
});
