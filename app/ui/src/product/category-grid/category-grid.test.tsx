import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { CategoryTotal } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { fakeReducedMotion, recordAnimations, setMotion } from '../../test/animation';
import { DEMO_TOTALS, DEMO_TOTALS_FOUND, STORY_DICTIONARY } from '../scan-hero/story-data';
import { CategoryGrid, type CategoryGridProps } from './category-grid';

const CARD = 100;

// happy-dom has no layout: each cell is placed by its place in the list, 100 px apart.
const placeCellsInOrder = (): void => {
  vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(function (this: HTMLElement): DOMRect {
    // eslint-disable-next-line testing-library/no-node-access -- a cell's place is its index among its siblings
    const siblings = this.tagName === 'LI' ? [...(this.parentElement?.children ?? [])] : [];
    const top = Math.max(siblings.indexOf(this), 0) * CARD;

    return { bottom: top + CARD, height: CARD, left: 0, right: 300, top, width: 300, x: 0, y: top } as DOMRect;
  });
};

const props = (overrides: Partial<CategoryGridProps> = {}): CategoryGridProps => ({
  found: DEMO_TOTALS_FOUND,
  onOpen: vi.fn(),
  onToggle: vi.fn(),
  scanning: false,
  selected: () => false,
  totals: DEMO_TOTALS,
  ...overrides,
});

const sized = (category: string, allocated: number): CategoryTotal => {
  const total = DEMO_TOTALS.find((each) => each.category === category) ?? DEMO_TOTALS[0];

  return {
    category,
    count: total?.count ?? 1,
    highestDanger: total?.highestDanger ?? 'safe',
    sizes: { allocated, completeness: 'complete', logical: allocated, reclaimable: allocated },
  };
};

const names = (): readonly string[] =>
  within(screen.getByRole('list', { name: 'Categories' }))
    .getAllByRole('button', { name: /^Open / })
    // The card's visible name, which its accessible name already reads as "Open …".
    .map((button) => within(button).getAllByText(/./u, { selector: '[aria-hidden="true"]' })[0]?.textContent ?? '');

const wait = (ms: number): void => {
  act(() => {
    vi.advanceTimersByTime(ms);
  });
};

beforeEach(() => {
  setCoreDictionary(STORY_DICTIONARY);
  fakeReducedMotion(false);
  placeCellsInOrder();
});

afterEach(async () => {
  vi.useRealTimers();
  await setMotion(null);
});

describe('CategoryGrid', () => {
  describe('negative cases', () => {
    it('cards do not reorder more than once per move duration during a scan', () => {
      vi.useFakeTimers();
      const first = [sized('demo.caches', 300), sized('demo.downloads', 200)];
      const { rerender } = render(<CategoryGrid {...props({ scanning: true, totals: first })} />);

      rerender(
        <CategoryGrid
          {...props({ scanning: true, totals: [sized('demo.caches', 300), sized('demo.downloads', 400)] })}
        />,
      );

      expect(names()).toStrictEqual(['Example downloads', 'Example caches']);

      rerender(
        <CategoryGrid
          {...props({ scanning: true, totals: [sized('demo.caches', 500), sized('demo.downloads', 400)] })}
        />,
      );
      wait(100);

      expect(names()).toStrictEqual(['Example downloads', 'Example caches']);

      wait(220);

      expect(names()).toStrictEqual(['Example caches', 'Example downloads']);
    });

    it('a new card joins at the end until the next move', () => {
      vi.useFakeTimers();
      const { rerender } = render(
        <CategoryGrid
          {...props({ scanning: true, totals: [sized('demo.caches', 300), sized('demo.downloads', 200)] })}
        />,
      );

      rerender(
        <CategoryGrid
          {...props({ scanning: true, totals: [sized('demo.caches', 300), sized('demo.downloads', 400)] })}
        />,
      );
      rerender(
        <CategoryGrid
          {...props({
            scanning: true,
            totals: [sized('demo.caches', 300), sized('demo.downloads', 400), sized('demo.images', 900)],
          })}
        />,
      );

      expect(names()).toStrictEqual(['Example downloads', 'Example caches', 'Example images']);

      wait(320);

      expect(names()).toStrictEqual(['Example images', 'Example downloads', 'Example caches']);
    });

    it('motion off reorders without animation', async () => {
      await setMotion('off');
      const animations = recordAnimations();
      const { rerender } = render(<CategoryGrid {...props()} />);

      rerender(
        <CategoryGrid {...props({ totals: [...DEMO_TOTALS.slice(0, 3), sized('demo.volumes', 9_000_000_000)] })} />,
      );

      expect(names()[0]).toBe('Example volumes');
      expect(animations).toHaveLength(0);
    });

    it('a category of no size has no share bar', () => {
      render(
        <CategoryGrid {...props({ found: { allocated: 0, completeness: 'unknown', logical: 0, reclaimable: 0 } })} />,
      );

      // eslint-disable-next-line testing-library/no-node-access -- the share bar is decoration, without a role
      expect(document.querySelectorAll('[data-share]')).toHaveLength(0);
    });
  });

  describe('positive cases', () => {
    it('cards are sorted largest first', () => {
      render(<CategoryGrid {...props({ totals: DEMO_TOTALS.toReversed() })} />);

      expect(names()).toStrictEqual(['Example caches', 'Example downloads', 'Example images', 'Example volumes']);
    });

    it('a card that grows past another glides to its place', () => {
      const animations = recordAnimations();
      const { rerender } = render(<CategoryGrid {...props()} />);

      rerender(
        <CategoryGrid {...props({ totals: [...DEMO_TOTALS.slice(0, 3), sized('demo.volumes', 9_000_000_000)] })} />,
      );
      const volumes = animations.find(({ element }) => element.textContent.includes('Example volumes'));

      // It was the fourth card and is the first: it starts three cards lower and glides up.
      expect(volumes?.keyframes[0]).toStrictEqual({ transform: 'translate(0px, 300px)' });
    });

    it('opens and selects a card by its category', () => {
      const onOpen = vi.fn();
      const onToggle = vi.fn();
      render(<CategoryGrid {...props({ onOpen, onToggle, selected: (category) => category === 'demo.images' })} />);

      fireEvent.click(screen.getByRole('button', { name: /^Open Example downloads/u }));
      fireEvent.click(screen.getByRole('checkbox', { name: 'Select Example caches' }));

      expect(onOpen).toHaveBeenCalledWith('demo.downloads');
      expect(onToggle).toHaveBeenCalledWith('demo.caches');
      expect(screen.getByRole('checkbox', { name: 'Select Example images' })).toBeChecked();
    });
  });
});
