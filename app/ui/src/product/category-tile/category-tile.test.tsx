import { fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { MenuItem } from '../../design/primitives/menu/menu';
import type { CategoryTotal } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { DEMO_TOTALS, STORY_DICTIONARY } from '../../test/story-data';
import { CategoryTile, type CategoryTileProps } from './category-tile';

const [CACHES, , IMAGES, VOLUMES] = DEMO_TOTALS as readonly [
  CategoryTotal,
  CategoryTotal,
  CategoryTotal,
  CategoryTotal,
];

// Vitest empties a CSS import, so the stylesheet is read from the disk.
const css = readFileSync(path.join(import.meta.dirname, 'category-tile.module.css'), 'utf8');

const MENU: readonly MenuItem[] = [
  {
    key: 'ignore-category',
    label: 'Ignore all Example caches',
    onSelect: vi.fn(),
  },
];

const tile = (overrides: Partial<CategoryTileProps> = {}): CategoryTileProps => ({
  icon: 'cache',
  menu: MENU,
  onDetails: vi.fn(),
  onToggle: vi.fn(),
  selected: false,
  total: CACHES,
  ...overrides,
});

const LIMITED = { kind: 'limited', text: 'Some of it could not be seen: Full Disk Access.' } as const;

const UNAVAILABLE = {
  kind: 'unavailable',
  text: 'Docker is installed but not running. Start it, then scan again.',
} as const;

const detailsButton = (): HTMLElement => screen.getByRole('button', { name: 'Details' });

beforeEach(() => {
  setCoreDictionary(STORY_DICTIONARY);
});

describe('CategoryTile', () => {
  describe('negative cases', () => {
    it('an unavailable category has no checkbox but keeps Details and the menu', () => {
      const props = tile({ access: UNAVAILABLE });
      render(<CategoryTile {...props} />);

      expect(screen.queryByRole('checkbox')).toBeNull();
      expect(screen.getByText(UNAVAILABLE.text)).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Options for Example caches' })).toBeInTheDocument();

      fireEvent.click(detailsButton());

      expect(props.onDetails).toHaveBeenCalledOnce();
      expect(props.onToggle).not.toHaveBeenCalled();
    });

    it('a tile found while scanning has no checkbox, Details or menu', () => {
      render(<CategoryTile {...tile({ scanning: true })} />);

      expect(screen.queryByRole('checkbox')).toBeNull();
      expect(screen.queryByRole('button', { name: 'Details' })).toBeNull();
      expect(screen.queryByRole('button', { name: 'Options for Example caches' })).toBeNull();
      expect(screen.getByText('Example caches')).toBeInTheDocument();
    });

    it('a tile without onToggle has no checkbox', () => {
      render(<CategoryTile {...tile({ onToggle: undefined, total: IMAGES })} />);

      expect(screen.queryByRole('checkbox')).toBeNull();
      expect(detailsButton()).toBeInTheDocument();
    });

    it('a tile shows no level badge', () => {
      render(<CategoryTile {...tile({ total: VOLUMES })} />);

      expect(screen.queryByText('High')).toBeNull();
      expect(screen.queryByText('Safe')).toBeNull();
    });

    it('has no button for its body and does not move on hover', () => {
      render(<CategoryTile {...tile()} />);

      expect(
        screen.getAllByRole('button').map((button) => button.getAttribute('aria-label') ?? button.textContent),
      ).toStrictEqual(['Details', 'Options for Example caches']);
      expect(css).not.toContain('translate');
    });

    it('the checkbox does not open the details', () => {
      const props = tile();
      render(<CategoryTile {...props} />);

      fireEvent.click(screen.getByRole('checkbox', { name: 'Select Example caches' }));

      expect(props.onDetails).not.toHaveBeenCalled();
    });

    it('an unknown size shows — and never 0', () => {
      render(
        <CategoryTile
          {...tile({
            total: { ...CACHES, sizes: { allocated: 0, completeness: 'unknown', logical: 0, reclaimable: 0 } },
          })}
        />,
      );

      expect(screen.getByText('Size not known')).toBeInTheDocument();
      expect(screen.queryByText(/bytes/u)).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('shows the icon, name, line, size, count, Details and the menu', () => {
      render(<CategoryTile {...tile()} />);

      expect(screen.getByRole('heading', { level: 3, name: 'Example caches' })).toBeInTheDocument();
      expect(screen.getByText('Made-up caches to try Fresher')).toBeInTheDocument();
      expect(document.body).toHaveTextContent(/1\.8 GB.*2 items/u);
      expect(detailsButton()).toHaveAccessibleDescription('Example caches');
      expect(screen.getByRole('button', { name: 'Options for Example caches' })).toBeInTheDocument();
      // eslint-disable-next-line testing-library/no-node-access -- the icon is decorative, without a role
      expect(document.querySelector('[data-card] svg')).not.toBeNull();
    });

    it('the checkbox selects the category', () => {
      const props = tile();
      render(<CategoryTile {...props} />);
      const checkbox = screen.getByRole('checkbox', { name: 'Select Example caches' });

      fireEvent.click(checkbox);

      expect(props.onToggle).toHaveBeenCalledOnce();
      expect(checkbox.tagName).toBe('INPUT');
    });

    it('a selected or mixed tile shows its checkbox state', () => {
      const { rerender } = render(<CategoryTile {...tile({ selected: true })} />);

      expect(screen.getByRole('checkbox')).toBeChecked();

      rerender(<CategoryTile {...tile({ selected: 'mixed' })} />);

      expect(screen.getByRole('checkbox')).toBePartiallyChecked();
    });

    it('the menu offers Ignore all <category>', () => {
      render(<CategoryTile {...tile()} />);

      fireEvent.click(screen.getByRole('button', { name: 'Options for Example caches' }));

      expect(screen.getByRole('menuitem', { name: 'Ignore all Example caches' })).toBeInTheDocument();
    });

    it('a limited tile keeps its partial size and says why', () => {
      render(
        <CategoryTile
          {...tile({
            access: LIMITED,
            total: { ...CACHES, sizes: { ...CACHES.sizes, completeness: 'partial' } },
          })}
        />,
      );

      expect(screen.getByText(/at least 1\.8 GB/u)).toBeInTheDocument();
      expect(screen.getByText(LIMITED.text)).toBeInTheDocument();
      expect(screen.getByRole('checkbox')).toBeInTheDocument();
    });

    it('takes the level of its category for its border and data', () => {
      render(<CategoryTile {...tile({ total: IMAGES })} />);

      // eslint-disable-next-line testing-library/no-node-access -- the tile is found by its marker
      expect(document.querySelector('[data-card]')).toHaveAttribute('data-level', 'medium');
      // eslint-disable-next-line testing-library/no-node-access -- the tile's surface has no role
      expect(document.querySelector('[data-surface]')).toHaveAttribute('data-surface', 'medium');
    });
  });
});
