import { fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Sizes } from '../../ipc/bindings.generated';

import { DURATION } from '../../design/tokens/motion';
import { setCoreDictionary } from '../../i18n';
import { fakeFrames, fakeReducedMotion, setMotion } from '../../test/animation';
import { ResultSummary, type ResultSummaryProps } from './result-summary';

const FREED: Sizes = { allocated: 2_250_000_000, completeness: 'complete', logical: 2_200_000_000 };

const SKIPPED: ResultSummaryProps['skipped'] = [
  {
    reason: { args: {}, key: 'test.skipped.in-use' },
    title: { args: { name: { kind: 'text', value: 'editor-cache' } }, key: 'test.skipped.title' },
  },
  {
    reason: { args: {}, key: 'test.skipped.gone' },
    title: { args: { name: { kind: 'text', value: 'installer.dmg' } }, key: 'test.skipped.title' },
  },
];

const renderSummary = (props: Partial<ResultSummaryProps> = {}): ReturnType<typeof render> =>
  render(
    <ResultSummary
      freed={FREED}
      method="trash"
      moved={12}
      onDone={vi.fn()}
      onOpenTrash={vi.fn()}
      skipped={[]}
      {...props}
    />,
  );

// The counting copy of the freed size, hidden from assistive technology.
const counting = (): string =>
  // eslint-disable-next-line testing-library/no-node-access -- the copy has no role, by design
  document.querySelector('[data-numeric] > [aria-hidden]')?.textContent ?? '';

// eslint-disable-next-line testing-library/no-node-access -- the wave is decoration, hidden from assistive technology
const wave = (): Element | null => document.querySelector('[data-light-wave]');

beforeEach(() => {
  fakeReducedMotion(false);
  setCoreDictionary({
    'test.skipped.gone': 'It was already gone.',
    'test.skipped.in-use': 'An app is using it.',
    'test.skipped.title': 'Example: {name}',
  });
});

afterEach(async () => {
  vi.useRealTimers();
  setCoreDictionary({});
  await setMotion(null);
});

describe('ResultSummary', () => {
  describe('negative cases', () => {
    it('the freed number does not count under reduced motion', async () => {
      await setMotion('reduced');
      fakeFrames();
      renderSummary();

      expect(counting()).toBe('2.3 GB');
      expect(wave()).not.toBeInTheDocument();
    });

    it('the light wave plays once', () => {
      fakeFrames();
      const { rerender } = renderSummary();
      const first = wave();

      expect(first).toBeInTheDocument();

      fireEvent.animationEnd(first as Element);

      expect(wave()).not.toBeInTheDocument();

      rerender(<ResultSummary freed={FREED} method="trash" moved={13} onDone={vi.fn()} skipped={[]} />);

      expect(wave()).not.toBeInTheDocument();
    });

    it('the ambient light turns back from green after the wave', () => {
      fakeFrames();
      renderSummary();
      // eslint-disable-next-line testing-library/no-node-access -- the light is decoration
      const safe = (): Element | null => document.querySelector('[data-tone="safe"]');

      expect(safe()).toHaveAttribute('data-shown');

      fireEvent.animationEnd(wave() as Element);

      expect(safe()).not.toHaveAttribute('data-shown');
    });

    it('Open Trash is absent after a permanent delete', () => {
      renderSummary({ method: 'permanent' });

      expect(screen.queryByRole('button', { name: 'Open Trash' })).not.toBeInTheDocument();
      expect(screen.getByText('Deleted 12 items permanently')).toBeInTheDocument();
    });

    it('Open Trash is absent when nothing opens it', () => {
      render(<ResultSummary freed={FREED} method="trash" moved={1} onDone={vi.fn()} skipped={[]} />);

      expect(screen.queryByRole('button', { name: 'Open Trash' })).not.toBeInTheDocument();
      expect(screen.getByText('Moved 1 item to the Trash')).toBeInTheDocument();
    });

    it('a skipped item always shows its reason', () => {
      renderSummary({ skipped: SKIPPED });

      const list = screen.getByRole('list', { name: '2 items were skipped' });
      const items = within(list).getAllByRole('listitem');

      expect(items).toHaveLength(2);
      expect(items[0]).toHaveTextContent('Example: editor-cache');
      expect(items[0]).toHaveTextContent('An app is using it.');
      expect(items[1]).toHaveTextContent('Example: installer.dmg');
      expect(items[1]).toHaveTextContent('It was already gone.');
    });

    it('shows no skipped list when nothing was skipped', () => {
      renderSummary();

      expect(screen.queryByRole('list')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('the freed size reaches its value', () => {
      const frames = fakeFrames();
      renderSummary();

      expect(counting()).toBe('0 bytes');

      // One frame hands the number its value; the count starts on the next.
      frames.advance(16);
      frames.advance(16 + DURATION.deliberate / 2);

      expect(counting()).not.toBe('2.3 GB');

      frames.advance(DURATION.deliberate);

      expect(counting()).toBe('2.3 GB');
      expect(screen.getByText('2.3 GB', { selector: '.u-visually-hidden' })).toBeInTheDocument();
    });

    it('marks a partial freed size as a lower bound', async () => {
      await setMotion('off');
      renderSummary({ freed: { ...FREED, completeness: 'partial' } });

      expect(counting()).toBe('2.3 GB+');
    });

    it('Done calls onDone', () => {
      const onDone = vi.fn();
      renderSummary({ onDone });

      fireEvent.click(screen.getByRole('button', { name: 'Done' }));

      expect(onDone).toHaveBeenCalledOnce();
    });

    it('Open Trash calls onOpenTrash after a move to the Trash', () => {
      const onOpenTrash = vi.fn();
      renderSummary({ onOpenTrash });

      fireEvent.click(screen.getByRole('button', { name: 'Open Trash' }));

      expect(onOpenTrash).toHaveBeenCalledOnce();
      expect(screen.getByText('Moved 12 items to the Trash')).toBeInTheDocument();
    });

    it('is a region named by its title', () => {
      renderSummary();

      expect(screen.getByRole('region', { name: 'Freed' })).toBeInTheDocument();
    });
  });
});
