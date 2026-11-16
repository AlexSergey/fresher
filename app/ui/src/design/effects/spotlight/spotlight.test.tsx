import { fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { type JSX } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { fakeFrames, fakeReducedMotion } from '../../../test/animation';
import { useSpotlightProps } from './spotlight';

const css = readFileSync(path.join(import.meta.dirname, 'spotlight.module.css'), 'utf8');

const Card = (): JSX.Element => {
  const spotlight = useSpotlightProps();

  return (
    <div {...spotlight} role="group">
      Xcode caches
    </div>
  );
};

const card = (): HTMLElement => screen.getByRole('group');

beforeEach(() => {
  fakeReducedMotion(false);
  vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue({ left: 100, top: 50 } as DOMRect);
});

afterEach(() => {
  vi.useRealTimers();
});

describe('Spotlight', () => {
  describe('negative cases', () => {
    it('the spotlight writes nothing without a pointer', () => {
      const frames = fakeFrames();
      render(<Card />);

      frames.advance(64);

      expect(card().style.getPropertyValue('--spot-x')).toBe('');
      expect(card().style.getPropertyValue('--spot-y')).toBe('');
    });

    it('shows on hover only, not on keyboard focus', () => {
      expect(css).not.toMatch(/:focus/u);
      expect(css).toMatch(/\.spotlight:hover::before,\s*\.spotlight\[data-hover\]::before \{/u);
    });

    it('is not drawn under reduced motion', () => {
      expect(css).toMatch(/:root\[data-motion='reduced'\] \.spotlight::before \{\s*display: none;/u);
      expect(css).toMatch(
        /@media \(prefers-reduced-motion: reduce\) \{\s*:root:not\(\[data-motion='full'\], \[data-motion='off'\]\) \.spotlight::before \{\s*display: none;/u,
      );
    });
  });

  describe('positive cases', () => {
    it('adds no element: the card spreads a class and a ref', () => {
      render(<Card />);

      expect(card().className).not.toBe('');
      expect(card()).toHaveTextContent(/^Xcode caches$/u);
    });

    it('the spotlight follows the pointer once per frame', () => {
      const frames = fakeFrames();
      const setProperty = vi.spyOn(CSSStyleDeclaration.prototype, 'setProperty');
      render(<Card />);

      fireEvent.pointerMove(card(), { clientX: 110, clientY: 60 });
      fireEvent.pointerMove(card(), { clientX: 220, clientY: 130 });
      frames.advance(16);

      expect(setProperty).toHaveBeenCalledTimes(2);
      expect(card().style.getPropertyValue('--spot-x')).toBe('120px');
      expect(card().style.getPropertyValue('--spot-y')).toBe('80px');
      expect(css).toMatch(/transform: translate\(var\(--spot-x, 0\), var\(--spot-y, 0\)\);/u);
    });
  });
});
