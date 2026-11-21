import { fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { MenuItem } from '../../design/primitives/menu/menu';
import type { CategoryTotal } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { DEMO_TOTALS, STORY_DICTIONARY } from '../scan-hero/story-data';
import { CategoryCard, type CategoryCardProps } from './category-card';

const [CACHES, , , VOLUMES] = DEMO_TOTALS as readonly [CategoryTotal, CategoryTotal, CategoryTotal, CategoryTotal];

// Vitest empties a CSS import, so the stylesheet is read from the disk.
const css = readFileSync(path.join(import.meta.dirname, 'category-card.module.css'), 'utf8');

const MENU: readonly MenuItem[] = [
  {
    description: 'Hides it from the results and never removes it.',
    key: 'ignore-category',
    label: 'Ignore this category',
    onSelect: vi.fn(),
  },
  {
    description: 'Hides everything this module finds.',
    key: 'ignore-module',
    label: 'Ignore Demo module',
    onSelect: vi.fn(),
  },
];

const card = (overrides: Partial<CategoryCardProps> = {}): CategoryCardProps => ({
  menu: MENU,
  onDetails: vi.fn(),
  onToggle: vi.fn(),
  selected: false,
  share: 0.5,
  total: CACHES,
  ...overrides,
});

const LIMITED = { kind: 'limited', text: 'Some of it could not be seen: Full Disk Access.' } as const;

const UNAVAILABLE = {
  kind: 'unavailable',
  text: 'Docker is installed but not running. Start it, then scan again.',
} as const;

const detailsButton = (): HTMLElement => screen.getByRole('button', { name: 'Details' });

// The share bar is decoration, without a role.
// eslint-disable-next-line testing-library/no-node-access -- found by its marker
const shareBar = (): HTMLElement | null => document.querySelector('[data-share]');

beforeEach(() => {
  setCoreDictionary(STORY_DICTIONARY);
});

describe('CategoryCard', () => {
  describe('negative cases', () => {
    it('has no button for its body', () => {
      render(<CategoryCard {...card()} />);

      expect(
        screen.getAllByRole('button').map((button) => button.getAttribute('aria-label') ?? button.textContent),
      ).toStrictEqual(['Details', 'Options for Example caches']);
      expect(screen.getByRole('heading', { level: 3, name: 'Example caches' })).toBeInTheDocument();
    });

    it('does not move on hover', () => {
      expect(css).not.toContain('translateY');
      expect(css).not.toContain('--scale-press-card');
    });

    it('an unavailable card has no checkbox but keeps Details and the menu', () => {
      const props = card({ access: UNAVAILABLE });
      render(<CategoryCard {...props} />);

      expect(screen.queryByRole('checkbox')).toBeNull();
      expect(screen.getByText(UNAVAILABLE.text)).toBeInTheDocument();
      expect(detailsButton()).toBeEnabled();
      expect(screen.getByRole('button', { name: 'Options for Example caches' })).toBeInTheDocument();

      fireEvent.click(detailsButton());

      expect(props.onDetails).toHaveBeenCalledOnce();
      expect(props.onToggle).not.toHaveBeenCalled();
    });

    it('a ready card shows no access text', () => {
      render(<CategoryCard {...card()} />);

      expect(screen.queryByText(UNAVAILABLE.text)).toBeNull();
      expect(screen.queryByText(LIMITED.text)).toBeNull();
    });

    it('the checkbox does not open the details', () => {
      const props = card();
      render(<CategoryCard {...props} />);

      fireEvent.click(screen.getByRole('checkbox', { name: 'Select Example caches' }));

      expect(props.onDetails).not.toHaveBeenCalled();
      expect(props.onToggle).toHaveBeenCalledOnce();
    });

    it('a card never shows its level by colour alone', () => {
      render(<CategoryCard {...card({ total: VOLUMES })} />);

      expect(screen.getByText('High')).toBeInTheDocument();
      // eslint-disable-next-line testing-library/no-node-access -- the level's icon is decorative, without a role
      expect(document.querySelectorAll('svg')).not.toHaveLength(0);
    });

    it('a zero share draws no bar', () => {
      render(<CategoryCard {...card({ share: 0 })} />);

      expect(shareBar()).toBeNull();
    });

    it('a share above the whole is drawn as the whole', () => {
      render(<CategoryCard {...card({ share: 1.5 })} />);

      expect(shareBar()?.style.getPropertyValue('--share')).toBe('1');
    });

    it('an unknown size shows — and never 0', () => {
      render(
        <CategoryCard
          {...card({
            total: { ...CACHES, sizes: { allocated: 0, completeness: 'unknown', logical: 0, reclaimable: 0 } },
          })}
        />,
      );

      expect(screen.getByText('Size not known')).toBeInTheDocument();
      expect(screen.queryByText(/bytes/u)).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('Details calls onDetails, described by the name', () => {
      const props = card();
      render(<CategoryCard {...props} />);

      expect(detailsButton()).toHaveAccessibleDescription('Example caches');

      fireEvent.click(detailsButton());

      expect(props.onDetails).toHaveBeenCalledOnce();
    });

    it('the menu offers Ignore this category and Ignore <module>, each with its sentence', () => {
      render(<CategoryCard {...card()} />);

      fireEvent.click(screen.getByRole('button', { name: 'Options for Example caches' }));

      expect(screen.getByRole('menuitem', { name: 'Ignore this category' })).toHaveAccessibleDescription(
        'Hides it from the results and never removes it.',
      );
      expect(screen.getByRole('menuitem', { name: 'Ignore Demo module' })).toHaveAccessibleDescription(
        'Hides everything this module finds.',
      );
    });

    it('a limited card keeps its partial size and says why', () => {
      render(
        <CategoryCard
          {...card({
            access: LIMITED,
            total: { ...CACHES, sizes: { ...CACHES.sizes, completeness: 'partial' } },
          })}
        />,
      );

      expect(screen.getByText(/at least 1\.8 GB/u)).toBeInTheDocument();
      expect(screen.getByText(LIMITED.text)).toBeInTheDocument();
      expect(screen.getByRole('checkbox')).toBeInTheDocument();
    });

    it("shows the whole category's size", () => {
      render(<CategoryCard {...card()} />);

      expect(document.body).toHaveTextContent(/1\.8 GB.*2 items/u);
    });

    it('shows the name, what it is and its level', () => {
      render(<CategoryCard {...card()} />);

      expect(screen.getByText('Example caches')).toBeInTheDocument();
      expect(screen.getByText(/Example data: made-up cache folders/u)).toBeInTheDocument();
      expect(screen.getByText('Safe')).toBeInTheDocument();
    });

    it('selects with the checkbox and Space', () => {
      const props = card();
      render(<CategoryCard {...props} />);
      const checkbox = screen.getByRole('checkbox', { name: 'Select Example caches' });

      // A native checkbox: the browser turns Space into a click.
      fireEvent.click(checkbox);

      expect(props.onToggle).toHaveBeenCalledOnce();
      expect(checkbox.tagName).toBe('INPUT');
    });

    it('a selected or mixed card shows its checkbox state', () => {
      const { rerender } = render(<CategoryCard {...card({ selected: true })} />);

      expect(screen.getByRole('checkbox')).toBeChecked();

      rerender(<CategoryCard {...card({ selected: 'mixed' })} />);

      expect(screen.getByRole('checkbox')).toBePartiallyChecked();
    });

    it('the share bar scales to the share', () => {
      render(<CategoryCard {...card({ share: 0.25 })} />);

      expect(shareBar()?.style.getPropertyValue('--share')).toBe('0.25');
    });
  });
});
