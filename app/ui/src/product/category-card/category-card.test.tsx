import { fireEvent, render, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { CategoryTotal } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { DEMO_TOTALS, STORY_DICTIONARY } from '../scan-hero/story-data';
import { CategoryCard, type CategoryCardProps } from './category-card';

const [CACHES, , , VOLUMES] = DEMO_TOTALS as readonly [CategoryTotal, CategoryTotal, CategoryTotal, CategoryTotal];

const card = (overrides: Partial<CategoryCardProps> = {}): CategoryCardProps => ({
  onOpen: vi.fn(),
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

const openButton = (): HTMLElement => screen.getByRole('button', { name: /^Open Example caches/u });

// The share bar is decoration, without a role.
// eslint-disable-next-line testing-library/no-node-access -- found by its marker
const shareBar = (): HTMLElement | null => document.querySelector('[data-share]');

beforeEach(() => {
  setCoreDictionary(STORY_DICTIONARY);
});

describe('CategoryCard', () => {
  describe('negative cases', () => {
    it('an unavailable category is shown disabled, not hidden', () => {
      render(<CategoryCard {...card({ access: UNAVAILABLE })} />);

      expect(openButton()).toHaveAttribute('aria-disabled', 'true');
      expect(openButton()).not.toBeDisabled();
    });

    it('an unavailable category cannot be selected', () => {
      const props = card({ access: UNAVAILABLE });
      render(<CategoryCard {...props} />);

      fireEvent.click(openButton());

      expect(screen.queryByRole('checkbox')).toBeNull();
      expect(props.onOpen).not.toHaveBeenCalled();
      expect(props.onToggle).not.toHaveBeenCalled();
    });

    it('a ready category shows no access text', () => {
      render(<CategoryCard {...card()} />);

      expect(openButton()).not.toHaveAttribute('aria-disabled');
      expect(screen.queryByText(UNAVAILABLE.text)).toBeNull();
      expect(screen.queryByRole('tooltip')).toBeNull();
    });

    it('a limited category is not disabled', () => {
      const props = card({ access: LIMITED });
      render(<CategoryCard {...props} />);

      fireEvent.click(openButton());

      expect(openButton()).not.toHaveAttribute('aria-disabled');
      expect(props.onOpen).toHaveBeenCalledOnce();
      expect(screen.getByRole('checkbox')).toBeInTheDocument();
    });

    it('a card never shows its level by colour alone', () => {
      render(<CategoryCard {...card({ total: VOLUMES })} />);
      const details = screen.getByRole('button', { name: /^Open Example volumes/u });

      expect(within(details).getByText('High')).toBeInTheDocument();
      // eslint-disable-next-line testing-library/no-node-access -- the level's icon is decorative, without a role
      expect(details.querySelectorAll('svg')).not.toHaveLength(0);
    });

    it('names the card from what it shows, with nothing read twice', () => {
      render(<CategoryCard {...card()} />);

      // Label in Name (WCAG 2.5.3): an `aria-label` would hide the visible text from the name.
      expect(openButton()).not.toHaveAttribute('aria-label');
      expect(openButton()).not.toHaveAttribute('aria-describedby');
      expect(openButton().textContent.match(/Example caches/gu)).toHaveLength(2);
      expect(openButton()).not.toHaveAccessibleName(/Example caches.*Example caches/u);
    });

    it('the checkbox does not open the category', () => {
      const props = card();
      render(<CategoryCard {...props} />);

      fireEvent.click(screen.getByRole('checkbox', { name: 'Select Example caches' }));

      expect(props.onOpen).not.toHaveBeenCalled();
      expect(props.onToggle).toHaveBeenCalledOnce();
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

      expect(within(openButton()).getByText('Size not known')).toBeInTheDocument();
      expect(within(openButton()).queryByText(/bytes/u)).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('a limited category shows a partial size and says why on focus', () => {
      render(
        <CategoryCard
          {...card({
            access: LIMITED,
            total: { ...CACHES, sizes: { ...CACHES.sizes, completeness: 'partial' } },
          })}
        />,
      );

      expect(within(openButton()).getByText(/at least 1\.8 GB/u)).toBeInTheDocument();

      fireEvent.focusIn(openButton());

      expect(screen.getByRole('tooltip')).toHaveTextContent(LIMITED.text);
      expect(openButton()).toHaveAccessibleDescription(LIMITED.text);
    });

    it('an unavailable category names what is missing and reads it as its description', () => {
      render(<CategoryCard {...card({ access: UNAVAILABLE })} />);

      expect(within(openButton()).getByText(UNAVAILABLE.text)).toBeInTheDocument();
      expect(openButton()).toHaveAccessibleDescription(UNAVAILABLE.text);
    });

    it('opens on click and Enter', () => {
      const props = card();
      render(<CategoryCard {...props} />);

      fireEvent.click(openButton());

      expect(props.onOpen).toHaveBeenCalledOnce();
      // A native button: the browser turns Enter and Space into a click.
      expect(openButton()).toHaveAttribute('type', 'button');
      expect(openButton().tagName).toBe('BUTTON');
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

    it('reads the category: its name, what it is, its level, size and count', () => {
      render(<CategoryCard {...card()} />);

      expect(openButton()).toHaveAccessibleName(
        /^Open Example caches Example data: made-up cache folders.* Safe 1\.8 GB 2 items$/u,
      );
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

    it('renders its actions', () => {
      render(<CategoryCard {...card({ actions: <button type="button">Ignore</button> })} />);

      expect(screen.getByRole('button', { name: 'Ignore' })).toBeInTheDocument();
    });
  });
});
