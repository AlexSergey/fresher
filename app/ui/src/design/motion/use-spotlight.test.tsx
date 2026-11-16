import { fireEvent, render, screen } from '@testing-library/react';
import { type JSX, useRef } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { useSpotlight } from '.';
import { fakeFrames, fakeReducedMotion, setMotion } from '../../test/animation';

const Card = (): JSX.Element => {
  const ref = useRef<HTMLDivElement>(null);
  useSpotlight(ref);

  return <div ref={ref}>Card</div>;
};

const card = (): HTMLElement => screen.getByText('Card');

beforeEach(() => {
  fakeReducedMotion(false);
  vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue({ left: 100, top: 50 } as DOMRect);
});

afterEach(async () => {
  vi.useRealTimers();
  await setMotion(null);
});

describe('useSpotlight', () => {
  describe('negative cases', () => {
    it('writes nothing under off', async () => {
      await setMotion('off');
      const frames = fakeFrames();
      render(<Card />);

      fireEvent.pointerMove(card(), { clientX: 130, clientY: 70 });
      frames.advance(16);

      expect(card().style.getPropertyValue('--spot-x')).toBe('');
    });

    it('writes nothing before the next frame', () => {
      fakeFrames();
      render(<Card />);

      fireEvent.pointerMove(card(), { clientX: 130, clientY: 70 });

      expect(card().style.getPropertyValue('--spot-x')).toBe('');
    });

    it('stops listening when unmounted', () => {
      const frames = fakeFrames();
      const { unmount } = render(<Card />);
      const element = card();

      fireEvent.pointerMove(element, { clientX: 130, clientY: 70 });
      unmount();
      frames.advance(16);

      expect(element.style.getPropertyValue('--spot-x')).toBe('');
    });
  });

  describe('positive cases', () => {
    it('writes once per frame', () => {
      const frames = fakeFrames();
      const setProperty = vi.spyOn(CSSStyleDeclaration.prototype, 'setProperty');
      render(<Card />);

      fireEvent.pointerMove(card(), { clientX: 110, clientY: 60 });
      fireEvent.pointerMove(card(), { clientX: 130, clientY: 70 });
      frames.advance(16);

      expect(setProperty).toHaveBeenCalledTimes(2);
      expect(card().style.getPropertyValue('--spot-x')).toBe('30px');
      expect(card().style.getPropertyValue('--spot-y')).toBe('20px');
    });
  });
});
