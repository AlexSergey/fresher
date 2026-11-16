import { act, fireEvent, render, screen } from '@testing-library/react';
import { type ReactNode } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { ScrollArea } from './scroll-area';

// The scroll area's content is this tall, and the area shows this much of it.
const layOut = ({ shown, tall }: { readonly shown: number; readonly tall: number }): void => {
  vi.spyOn(HTMLElement.prototype, 'scrollHeight', 'get').mockReturnValue(tall);
  vi.spyOn(HTMLElement.prototype, 'clientHeight', 'get').mockReturnValue(shown);
};

const show = (children: ReactNode, screenKey?: string): ReturnType<typeof render> =>
  render(
    <ScrollArea label="Findings" {...(screenKey === undefined ? {} : { screenKey })}>
      {children}
    </ScrollArea>,
  );

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('ScrollArea', () => {
  describe('negative cases', () => {
    it('a scroll area that does not overflow is not focusable', () => {
      layOut({ shown: 400, tall: 400 });
      show(<p>Short</p>);

      expect(screen.queryByRole('region')).not.toBeInTheDocument();
      // eslint-disable-next-line testing-library/no-node-access -- an area that is not a region has no role to query by
      expect(document.querySelector('[tabindex]')).toBeNull();
    });

    it('an overflowing area with a focusable child leaves the focus to the child', () => {
      layOut({ shown: 400, tall: 1200 });
      show(<button type="button">Scan</button>);

      expect(screen.queryByRole('region')).not.toBeInTheDocument();
    });

    it('a new screen key starts at the top', () => {
      layOut({ shown: 400, tall: 1200 });
      const { rerender } = show(<p>Long</p>, 'negative-a');
      const element = screen.getByRole('region', { name: 'Findings' });

      element.scrollTop = 300;
      fireEvent.scroll(element);
      rerender(
        <ScrollArea label="Findings" screenKey="negative-b">
          <p>Long</p>
        </ScrollArea>,
      );

      expect(element.scrollTop).toBe(0);
    });
  });

  describe('positive cases', () => {
    it('an overflowing area without a focusable child takes the focus, named', () => {
      layOut({ shown: 400, tall: 1200 });
      show(<p>Long</p>);

      expect(screen.getByRole('region', { name: 'Findings' })).toHaveAttribute('tabindex', '0');
    });

    it('measures again when its size changes', () => {
      let resized: () => void = () => undefined;
      vi.stubGlobal(
        'ResizeObserver',
        class {
          constructor(callback: () => void) {
            resized = callback;
          }

          disconnect(): void {}

          observe(): void {}
        },
      );
      layOut({ shown: 400, tall: 400 });
      show(<p>Grows</p>);

      layOut({ shown: 400, tall: 900 });
      act(() => {
        resized();
      });

      expect(screen.getByRole('region', { name: 'Findings' })).toBeInTheDocument();
    });

    it('restores the scroll of a screen when its key comes back', () => {
      layOut({ shown: 400, tall: 1200 });
      const { rerender } = show(<p>Long</p>, 'positive-a');
      const element = screen.getByRole('region', { name: 'Findings' });

      element.scrollTop = 300;
      fireEvent.scroll(element);
      rerender(
        <ScrollArea label="Findings" screenKey="positive-b">
          <p>Long</p>
        </ScrollArea>,
      );
      rerender(
        <ScrollArea label="Findings" screenKey="positive-a">
          <p>Long</p>
        </ScrollArea>,
      );

      expect(element.scrollTop).toBe(300);
    });
  });
});
