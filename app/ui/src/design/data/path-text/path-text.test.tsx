import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { HOVER_DELAY } from '../../primitives/tooltip/tooltip';
import { PathText } from './path-text';

const LONG = '~/Library/Developer/Xcode/DerivedData/Fresher-abcdefghijklmnop/Build/Intermediates.noindex/Index.db';
// One character of the mono font, in pixels.
const CELL = 7;

// The layout a browser would give: the frame's width, and a mono cell of CELL pixels.
let frameWidth = 0;
let resize: () => void = () => undefined;

beforeEach(() => {
  vi.spyOn(HTMLElement.prototype, 'clientWidth', 'get').mockImplementation(() => frameWidth);
  vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(function (this: HTMLElement): DOMRect {
    return new DOMRect(0, 0, this.textContent.length * CELL, 16);
  });
  vi.stubGlobal(
    'ResizeObserver',
    class {
      readonly #callback: () => void;

      constructor(callback: () => void) {
        this.#callback = callback;
      }

      disconnect(): void {
        resize = (): void => undefined;
      }

      observe(): void {
        resize = this.#callback;
      }
    },
  );
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

// The path on screen: the one text hidden from assistive technology that holds a path.
const shownElement = (): HTMLElement =>
  screen.getByText(
    (_, element) => element?.getAttribute('aria-hidden') === 'true' && element.textContent.includes('/'),
  );

const shown = (): string => shownElement().textContent;

describe('PathText', () => {
  describe('negative cases', () => {
    it('a short path is not truncated', () => {
      frameWidth = 400;
      render(<PathText path="~/Library/Caches" />);

      expect(shown()).toBe('~/Library/Caches');
    });

    it('shows the whole path before it can measure', () => {
      frameWidth = 0;
      render(<PathText path={LONG} />);

      expect(shown()).toBe(LONG);
    });

    it('the shown text is hidden from assistive technology', () => {
      frameWidth = 140;
      render(<PathText path={LONG} />);

      expect(shownElement()).toHaveAttribute('aria-hidden', 'true');
      expect(screen.getByText(LONG)).toHaveClass('u-visually-hidden');
    });
  });

  describe('positive cases', () => {
    it('a long path keeps its first folder and its last segment', () => {
      frameWidth = 40 * CELL;
      render(<PathText path={LONG} />);

      expect(shown()).toHaveLength(40);
      expect(shown()).toMatch(/^~\/Library\/.*…\/Index\.db$/u);
    });

    it('measures again when its width changes', () => {
      frameWidth = 40 * CELL;
      render(<PathText path={LONG} />);

      frameWidth = 30 * CELL;
      act(() => {
        resize();
      });

      expect(shown()).toHaveLength(30);
    });

    it('the full path is its accessible text', () => {
      frameWidth = 40 * CELL;
      render(<PathText path={LONG} />);

      expect(screen.getByText(LONG)).toHaveClass('u-visually-hidden');
    });

    it('shows its display text, the full path staying its accessible text', () => {
      frameWidth = 80 * CELL;
      render(<PathText display="~/Library/Caches" path="/Users/a/Library/Caches" />);

      expect(shown()).toBe('~/Library/Caches');
      expect(screen.getByText('/Users/a/Library/Caches')).toHaveClass('u-visually-hidden');
    });

    it('the full path is in the tooltip', () => {
      vi.useFakeTimers();
      frameWidth = 40 * CELL;
      render(<PathText path={LONG} />);

      // eslint-disable-next-line testing-library/no-node-access -- the pointer enters the tooltip's anchor
      fireEvent.pointerEnter(shownElement().parentElement as HTMLElement);
      act(() => {
        vi.advanceTimersByTime(HOVER_DELAY);
      });

      expect(screen.getByRole('tooltip')).toHaveTextContent(LONG);
    });
  });
});
