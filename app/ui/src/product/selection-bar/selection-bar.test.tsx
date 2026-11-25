import { act, fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { Announcer } from '../../design/feedback/announcer/announcer';
import { setCoreDictionary } from '../../i18n';
import { fakeReducedMotion, recordAnimations, setMotion } from '../../test/animation';
import { FINDING_DICTIONARY } from '../../test/finding-fixtures';
import { SelectionBar, type SelectionBarProps } from './selection-bar';

const css = readFileSync(path.join(import.meta.dirname, 'selection-bar.module.css'), 'utf8');

const MANY: SelectionBarProps['summary'] = {
  count: 128,
  highest: 'high',
  sizes: { allocated: 4_200_000_000, completeness: 'complete', logical: 4_200_000_000, reclaimable: 4_200_000_000 },
  toCheck: { categories: 0 },
};

const NOTHING_GOES: SelectionBarProps['summary'] = {
  count: 0,
  highest: null,
  sizes: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 },
  toCheck: { categories: 0 },
};

const bar = (props: Partial<SelectionBarProps> = {}): React.JSX.Element => (
  <Announcer>
    <SelectionBar onClear={vi.fn()} onReview={vi.fn()} summary={MANY} {...props} />
  </Announcer>
);

const status = (): HTMLElement => screen.getByRole('status');

beforeEach(async () => {
  setCoreDictionary(FINDING_DICTIONARY);
  fakeReducedMotion(false);
  await setMotion('off');
});

afterEach(async () => {
  await setMotion(null);
});

describe('SelectionBar', () => {
  describe('negative cases', () => {
    it('the selection bar is absent when nothing is selected', () => {
      const { rerender } = render(bar({ summary: null }));

      expect(screen.queryByRole('region', { name: 'Selection' })).not.toBeInTheDocument();

      rerender(bar({ summary: NOTHING_GOES }));

      expect(screen.queryByRole('region', { name: 'Selection' })).not.toBeInTheDocument();
      expect(status()).toBeEmptyDOMElement();
    });

    it('says nothing is to check when no category is', () => {
      render(bar());

      expect(screen.getByRole('region', { name: 'Selection' })).not.toHaveTextContent('to check in Review');
    });

    it('shows no danger level', () => {
      render(bar());

      const region = screen.getByRole('region', { name: 'Selection' });

      expect(region).not.toHaveTextContent('High');
      // eslint-disable-next-line testing-library/no-node-access -- a level badge would carry its level as data
      expect(region.querySelector('[data-level]')).toBeNull();
    });

    it('is announced once when it appears, not on every change', () => {
      const { rerender } = render(bar({ summary: null }));

      rerender(bar());

      expect(status()).toHaveTextContent('128 items selected, 4.2 GB in all');

      rerender(bar({ summary: { ...MANY, count: 129 } }));

      expect(status()).toHaveTextContent('128 items selected, 4.2 GB in all');
    });

    it('leaves with a fade, keeping what it showed until it is gone', async () => {
      await setMotion(null);
      const animations = recordAnimations();
      const { rerender } = render(bar());

      rerender(bar({ summary: null }));

      expect(screen.getByRole('region', { name: 'Selection' })).toHaveTextContent('128 items selected');
      expect(animations.at(-1)?.keyframes).toStrictEqual([{ opacity: 1 }, { opacity: 0 }]);

      await animations.at(-1)?.finish();

      expect(screen.queryByRole('region', { name: 'Selection' })).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('the total has a description when it has the focus', () => {
      render(bar());

      // eslint-disable-next-line testing-library/no-node-access -- the hinted element has no role to query by
      const total = screen.getByText('4.2 GB', { selector: '.u-visually-hidden' }).parentElement as HTMLElement;
      act(() => {
        total.focus();
      });

      expect(total).toHaveAccessibleDescription('The space on the disk that everything you selected takes.');
    });

    it('shows while only categories to check are selected, and leads with how many', () => {
      render(bar({ summary: { ...NOTHING_GOES, toCheck: { categories: 2 } } }));

      const region = screen.getByRole('region', { name: 'Selection' });

      expect(region).toHaveTextContent(/^2 to check in Review/u);
      expect(region).not.toHaveTextContent('items selected');
      expect(status()).toHaveTextContent('2 to check in Review');
      expect(screen.getByRole('button', { name: 'Review' })).toBeVisible();
    });

    it('a change of the categories to check alone is announced', () => {
      const { rerender } = render(bar());

      rerender(bar({ summary: { ...MANY, toCheck: { categories: 1 } } }));

      expect(status()).toHaveTextContent('128 items selected, 4.2 GB in all · 1 to check in Review');
      expect(screen.getByRole('region', { name: 'Selection' })).toHaveTextContent('· 1 to check in Review');
    });

    it('the selection bar shows count, size, Clear and Review', () => {
      render(bar());

      const region = screen.getByRole('region', { name: 'Selection' });

      expect(region).toHaveTextContent('128 items selected');
      expect(screen.getByText('4.2 GB', { selector: '.u-visually-hidden' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Clear' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Review' })).toBeInTheDocument();
    });

    it('floats on its own panel, 18 px above the bottom', () => {
      expect(css).toMatch(/bottom: 18px;/u);
      expect(css).toMatch(/background: var\(--surface-floating\);/u);
      expect(css).toMatch(/border: 1px solid var\(--border-solid\);/u);
      expect(css).toMatch(/box-shadow: var\(--shadow-floating\);/u);
    });

    it('a partial total reads as a lower bound', () => {
      render(
        bar({
          summary: {
            count: 1,
            highest: 'safe',
            sizes: { allocated: 640_000_000, completeness: 'partial', logical: 1, reclaimable: 640_000_000 },
            toCheck: { categories: 0 },
          },
        }),
      );

      expect(screen.getByRole('region', { name: 'Selection' })).toHaveTextContent('1 item selected');
      expect(screen.getByText('at least 640 MB', { selector: '.u-visually-hidden' })).toBeInTheDocument();
      expect(screen.getByText('640 MB+')).toBeInTheDocument();
    });

    it('Review and Clear call their handlers', () => {
      const onClear = vi.fn();
      const onReview = vi.fn();
      render(bar({ onClear, onReview }));

      fireEvent.click(screen.getByRole('button', { name: 'Review' }));
      fireEvent.click(screen.getByRole('button', { name: 'Clear' }));

      expect(onReview).toHaveBeenCalledOnce();
      expect(onClear).toHaveBeenCalledOnce();
    });

    it('rises and fades in, its total counting over 200 ms', () => {
      expect(css).toMatch(/animation: selection-bar-in var\(--duration-base\) var\(--ease-enter\)/u);
      expect(css).toMatch(/transform: translateY\(var\(--distance-3\)\)/u);
      expect(css).not.toMatch(/--ease-settle/u);
    });
  });
});
