import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { fakeFrames, fakeReducedMotion, recordAnimations, setMotion } from '../../../test/animation';
import { ListRow } from '../list-row/list-row';
import { VirtualList, type VirtualListProps } from './virtual-list';

const ROW = 44;
// Ten rows fit the view.
const VIEW = 440;

// ResizeObserver as the browser calls it: once on observe, with the viewport's height.
const fakeViewport = (height: number): void => {
  vi.stubGlobal(
    'ResizeObserver',
    class {
      private readonly onResize: ResizeObserverCallback;

      constructor(onResize: ResizeObserverCallback) {
        this.onResize = onResize;
      }

      disconnect(): void {
        // Nothing to stop.
      }

      observe(): void {
        this.onResize([{ contentRect: { height } } as ResizeObserverEntry], this as unknown as ResizeObserver);
      }
    },
  );
};

const keyOf = (index: number): string => `k${String(index)}`;
const itemOf = (index: number): string => `Item ${String(index)}`;

type ListProps = Partial<VirtualListProps<string>> & { readonly count: number };

const list = (props: ListProps): React.JSX.Element => (
  <VirtualList
    getItem={itemOf}
    getKey={keyOf}
    label="Findings"
    renderRow={(_index, item, row) => <ListRow {...row} title={item ?? 'Loading'} />}
    rowHeight={ROW}
    {...props}
  />
);

const rows = (): readonly HTMLElement[] => screen.queryAllByRole('listitem');
const rowNamed = (title: string): HTMLElement => {
  const found = rows().find((candidate) => candidate.textContent === title);

  if (found === undefined) {
    throw new Error(`No row ${title}`);
  }

  return found;
};
const viewport = (): HTMLElement => screen.getByRole('list', { name: 'Findings' });

const press = (key: string): void => {
  // eslint-disable-next-line testing-library/no-node-access -- a key goes to whatever has the focus, as in a browser
  fireEvent.keyDown(document.activeElement ?? document.body, { key });
};

// The row's motion layer: what glides and what enters.
// eslint-disable-next-line testing-library/no-node-access -- the layer is presentation, it has no role to query by
const layerOf = (row: HTMLElement): HTMLElement | null => row.parentElement;

const focusRow = (title: string): void => {
  act(() => {
    rowNamed(title).focus();
  });
};

beforeEach(() => {
  fakeViewport(VIEW);
  fakeReducedMotion(false);
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

describe('VirtualList', () => {
  describe('negative cases', () => {
    it('renders no row for a count of 0', () => {
      const onRangeChange = vi.fn();
      render(list({ count: 0, onRangeChange }));

      expect(viewport()).toBeInTheDocument();
      expect(rows()).toHaveLength(0);
      expect(onRangeChange).not.toHaveBeenCalled();
    });

    it('renders no more than the visible rows and the overscan for 100,000', () => {
      render(list({ count: 100_000 }));

      // Ten in view, six below (none above the first).
      expect(rows()).toHaveLength(16);
      expect(rows().at(-1)).toHaveTextContent('Item 15');
    });

    it('does not move focus past the last row', () => {
      render(list({ count: 5 }));
      focusRow('Item 0');

      press('End');
      press('ArrowDown');

      expect(rowNamed('Item 4')).toHaveFocus();
      expect(rowNamed('Item 4')).toHaveAttribute('tabindex', '0');
    });

    it('does not move focus above the first row', () => {
      render(list({ count: 5 }));
      focusRow('Item 0');

      press('ArrowUp');
      press('PageUp');

      expect(rowNamed('Item 0')).toHaveFocus();
    });

    it('a disabled row does not toggle', () => {
      const onToggle = vi.fn();
      render(
        list({
          count: 3,
          onToggle,
          renderRow: (index, item, props) => <ListRow {...props} disabled={index === 1} title={item ?? ''} />,
        }),
      );
      focusRow('Item 1');

      press(' ');

      expect(onToggle).not.toHaveBeenCalled();
    });

    it('Space on a control inside a row is left to the control', () => {
      const onToggle = vi.fn();
      render(
        list({
          count: 3,
          onToggle,
          renderRow: (_index, item, props) => (
            <ListRow {...props} leading={<button type="button">Pick</button>} title={item ?? ''} />
          ),
        }),
      );
      const [pick] = screen.getAllByRole('button', { name: 'Pick' });

      act(() => {
        pick?.focus();
      });
      press(' ');

      expect(onToggle).not.toHaveBeenCalled();
    });

    it('an arrow on a control inside a row does not move the list', () => {
      render(
        list({
          count: 3,
          renderRow: (_index, item, props) => (
            <ListRow {...props} leading={<button type="button">Pick</button>} title={item ?? ''} />
          ),
        }),
      );
      const [pick] = screen.getAllByRole('button', { name: 'Pick' });

      act(() => {
        pick?.focus();
      });
      press('ArrowDown');

      expect(pick).toHaveFocus();
    });

    it('keys on an empty list do nothing', () => {
      const onToggle = vi.fn();
      render(list({ count: 0, onToggle }));

      fireEvent.keyDown(viewport(), { key: 'ArrowDown' });
      fireEvent.keyDown(viewport(), { key: ' ' });

      expect(onToggle).not.toHaveBeenCalled();
    });

    it('a scroll pending when the list goes away is dropped', () => {
      const frames = fakeFrames();
      const { unmount } = render(list({ count: 100 }));

      fireEvent.scroll(viewport());
      unmount();
      frames.advance(16);

      expect(screen.queryByRole('list')).not.toBeInTheDocument();
    });

    it('a focused key that left the view keeps the focus at its index', () => {
      const { rerender } = render(list({ count: 5 }));
      focusRow('Item 2');

      rerender(
        list({ count: 5, getItem: (index) => `Other ${String(index)}`, getKey: (index) => `other${String(index)}` }),
      );

      expect(rowNamed('Other 2')).toHaveAttribute('tabindex', '0');
    });

    it('a row the user left takes no focus back when the list renders again', async () => {
      const { rerender } = render(
        <>
          {list({ count: 5 })}
          <button type="button">Elsewhere</button>
        </>,
      );
      focusRow('Item 1');
      act(() => {
        rowNamed('Item 1').blur();
      });
      await Promise.resolve();

      rerender(
        <>
          {list({ count: 5, getKey: (index) => `other${String(index)}` })}
          <button type="button">Elsewhere</button>
        </>,
      );

      expect(document.body).toHaveFocus();

      focusRow('Item 2');
      act(() => {
        screen.getByRole('button', { name: 'Elsewhere' }).focus();
      });
      rerender(
        <>
          {list({ count: 5 })}
          <button type="button">Elsewhere</button>
        </>,
      );

      expect(screen.getByRole('button', { name: 'Elsewhere' })).toHaveFocus();
    });

    it('a row whose page is not loaded gets undefined', () => {
      const renderRow = vi.fn<VirtualListProps<string>['renderRow']>((_index, item, props) => (
        <ListRow {...props} title={item ?? 'Loading'} />
      ));
      render(list({ count: 6, getItem: (index) => (index < 3 ? itemOf(index) : undefined), renderRow }));

      expect(renderRow).toHaveBeenCalledWith(3, undefined, expect.objectContaining({ 'aria-posinset': 4 }));
      expect(rows().filter((candidate) => candidate.textContent === 'Loading')).toHaveLength(3);
    });

    it('keys that move nothing do nothing', () => {
      render(list({ count: 5 }));
      focusRow('Item 2');

      press('a');

      expect(rowNamed('Item 2')).toHaveFocus();
    });
  });

  describe('positive cases', () => {
    it('scrolling shows the rows at that offset', () => {
      const frames = fakeFrames();
      render(list({ count: 100_000 }));

      viewport().scrollTop = 500 * ROW;
      fireEvent.scroll(viewport());
      fireEvent.scroll(viewport());
      frames.advance(16);

      expect(rowNamed('Item 500')).toBeInTheDocument();
      expect(rows()[0]).toHaveTextContent('Item 494');
      expect(rows().some((candidate) => candidate.textContent === 'Item 0')).toBe(false);
    });

    it('reports the visible range for paging', () => {
      const frames = fakeFrames();
      const onRangeChange = vi.fn();
      render(list({ count: 100_000, onRangeChange }));

      expect(onRangeChange).toHaveBeenLastCalledWith(0, 15);

      viewport().scrollTop = 500 * ROW;
      fireEvent.scroll(viewport());
      frames.advance(16);

      // Rows 500 to 509 are in view; six more on each side are rendered and need their page too.
      expect(onRangeChange).toHaveBeenLastCalledWith(494, 515);
    });

    it('rows carry setsize and posinset', () => {
      render(list({ count: 100_000 }));

      expect(rowNamed('Item 0')).toHaveAttribute('aria-setsize', '100000');
      expect(rowNamed('Item 0')).toHaveAttribute('aria-posinset', '1');
      expect(rowNamed('Item 7')).toHaveAttribute('aria-posinset', '8');
    });

    it('has one tab stop, the first row at the start', () => {
      render(list({ count: 20 }));

      expect(rows().filter((candidate) => candidate.tabIndex === 0)).toStrictEqual([rowNamed('Item 0')]);
    });

    it('arrow keys move the focus and scroll it into view', () => {
      render(list({ count: 100 }));
      focusRow('Item 0');

      press('ArrowDown');

      expect(rowNamed('Item 1')).toHaveFocus();
      expect(viewport().scrollTop).toBe(0);

      press('PageDown');

      expect(rowNamed('Item 11')).toHaveFocus();
      expect(viewport().scrollTop).toBe(2 * ROW);

      press('End');

      expect(rowNamed('Item 99')).toHaveFocus();
      expect(viewport().scrollTop).toBe(100 * ROW - VIEW);

      press('Home');

      expect(rowNamed('Item 0')).toHaveFocus();
      expect(viewport().scrollTop).toBe(0);
    });

    it('Space toggles the focused row', () => {
      const onToggle = vi.fn();
      render(list({ count: 10, onToggle }));
      focusRow('Item 3');

      press(' ');

      expect(onToggle).toHaveBeenCalledExactlyOnceWith(3);
    });

    it('the focused row survives a re-render by its key', () => {
      const { rerender } = render(list({ count: 5 }));
      focusRow('Item 2');

      // A new item arrives above: the focused item is now the fourth row.
      rerender(
        list({
          count: 6,
          getItem: (index) => (index === 0 ? 'New' : itemOf(index - 1)),
          getKey: (index) => (index === 0 ? 'new' : keyOf(index - 1)),
        }),
      );

      expect(rowNamed('Item 2')).toHaveAttribute('tabindex', '0');
      expect(rowNamed('Item 2')).toHaveFocus();

      press('ArrowDown');

      expect(rowNamed('Item 3')).toHaveFocus();
    });

    it('a focused row whose page arrives hands the focus to its loaded row', () => {
      const loading = {
        getItem: (): undefined => undefined,
        getKey: (index: number): string => `loading${String(index)}`,
      };
      const { rerender } = render(list({ count: 5, ...loading }));
      focusRow('Loading');

      rerender(list({ count: 5 }));

      expect(rowNamed('Item 0')).toHaveFocus();

      press('ArrowDown');

      expect(rowNamed('Item 1')).toHaveFocus();
    });

    it('a reordered row plays a translation from its old index', () => {
      const animations = recordAnimations();
      const getKey = (index: number): string => keyOf(index);
      const { rerender } = render(list({ count: 3, getKey }));

      const swapped = (index: number): number => [1, 0, 2][index] ?? index;
      rerender(
        list({ count: 3, getItem: (index) => itemOf(swapped(index)), getKey: (index) => keyOf(swapped(index)) }),
      );

      const moved = animations.find((animation) => animation.element.textContent === 'Item 1');

      expect(moved?.keyframes).toStrictEqual([
        { transform: `translate(0px, ${String(ROW)}px)` },
        { transform: 'none' },
      ]);
      expect(animations.find((animation) => animation.element.textContent === 'Item 2')).toBeUndefined();
    });

    it('motion off moves rows without animation', async () => {
      await setMotion('off');
      const animations = recordAnimations();
      const { rerender } = render(list({ count: 3 }));

      rerender(list({ count: 3, getItem: (index) => itemOf(2 - index), getKey: (index) => keyOf(2 - index) }));

      expect(animations).toHaveLength(0);
      expect(rows().map((candidate) => candidate.textContent)).toStrictEqual(['Item 2', 'Item 1', 'Item 0']);
      expect(rows().some((candidate) => layerOf(candidate)?.hasAttribute('data-entering'))).toBe(false);

      await setMotion(null);
    });

    it('rows new in view enter with the stagger, rows scrolled into view do not', () => {
      const frames = fakeFrames();
      const { rerender } = render(list({ count: 2 }));

      expect(layerOf(rowNamed('Item 1'))).toHaveAttribute('data-entering');
      expect(layerOf(rowNamed('Item 1'))?.style.getPropertyValue('--i')).toBe('1');

      rerender(list({ count: 100 }));

      expect(layerOf(rowNamed('Item 2'))).toHaveAttribute('data-entering');

      viewport().scrollTop = 50 * ROW;
      fireEvent.scroll(viewport());
      frames.advance(16);

      expect(layerOf(rowNamed('Item 55'))).not.toHaveAttribute('data-entering');
    });
  });
});
