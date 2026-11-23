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

      rerender(
        bar({
          summary: {
            count: 0,
            highest: null,
            sizes: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 },
          },
        }),
      );

      expect(screen.queryByRole('region', { name: 'Selection' })).not.toBeInTheDocument();
      expect(status()).toBeEmptyDOMElement();
    });

    it('the highest level is shown with its icon and word', () => {
      render(bar());

      const region = screen.getByRole('region', { name: 'Selection' });

      expect(region).toHaveTextContent(/Highest danger level:\s*High/u);
      // eslint-disable-next-line testing-library/no-node-access -- the badge's icon is decoration beside its word
      expect(region.querySelector('[data-level="high"] svg')).not.toBeNull();
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

    it('the highest level has its meaning as a description when it has the focus', () => {
      render(bar());

      // eslint-disable-next-line testing-library/no-node-access -- the hinted element has no role to query by
      const level = screen.getByText('High').closest('[tabindex]') as HTMLElement;
      act(() => {
        level.focus();
      });

      expect(level).toHaveAccessibleDescription('May hold the only copy of your data. Removing it can lose work.');
    });

    it('the selection bar shows count, size and highest level', () => {
      render(bar());

      const region = screen.getByRole('region', { name: 'Selection' });

      expect(region).toHaveTextContent('128 items selected');
      expect(screen.getByText('4.2 GB', { selector: '.u-visually-hidden' })).toBeInTheDocument();
      expect(region).toHaveTextContent('High');
    });

    it('a partial total reads as a lower bound', () => {
      render(
        bar({
          summary: {
            count: 1,
            highest: 'safe',
            sizes: { allocated: 640_000_000, completeness: 'partial', logical: 1, reclaimable: 640_000_000 },
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
