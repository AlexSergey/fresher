import { fireEvent, render, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { setCoreDictionary } from '../../i18n';
import { DEMO_DICTIONARY, DEMO_LEFT_OUT } from '../../test/demo-scan';
import { forNowCount, keptCount, leftOutEntry } from '../../test/left-out';
import { LeftOutLines } from './left-out-lines';

const MB = 1_000_000;

const sized = (bytes: number): typeof DEMO_LEFT_OUT.sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

const named = (name: string): (typeof DEMO_LEFT_OUT.shown)[number]['title'] => ({
  args: { name: { kind: 'text', value: name } },
  key: 'demo.finding.title',
});

const KEPT = keptCount('demo.caches', 'demo.error.failed', 3, sized(2 * MB));

const items = (): HTMLElement | null => screen.queryByRole('list', { name: 'The items left out' });

const toggle = (name: RegExp): HTMLElement => screen.getByRole('button', { name });

// A count of two caches held back for now while their app is open.
const TWO = forNowCount('demo.caches', 'demo.left-out.open', sized(5 * MB), [
  leftOutEntry(named('editor'), '/Users/someone/Library/Caches/editor', sized(4 * MB)),
  leftOutEntry(named('notes'), '/Users/someone/Library/Caches/notes', sized(MB)),
]);

beforeEach(() => {
  setCoreDictionary({ ...DEMO_DICTIONARY, 'demo.error.failed': 'It is the version you use.' });
});

describe('LeftOutLines', () => {
  describe('negative cases', () => {
    it('shows no line without a left-out', () => {
      render(<LeftOutLines counts={[]} />);

      expect(screen.queryByRole('list')).not.toBeInTheDocument();
    });

    it('shows no item of a line held back for now before its toggle is pressed', () => {
      render(<LeftOutLines counts={[TWO]} />);

      expect(toggle(/^2 items left out \(5 MB\)/u)).toHaveAttribute('aria-expanded', 'false');
      expect(items()).toBeNull();
      expect(screen.queryByText('Example: editor')).not.toBeVisible();
    });

    it('has no toggle on a line held back for now with no item to show', () => {
      render(<LeftOutLines counts={[forNowCount('demo.caches', 'demo.left-out.open', sized(MB), [], 0)]} />);

      expect(screen.queryByRole('button')).toBeNull();
      expect(screen.getByRole('list', { name: 'Left out of this list' })).toHaveTextContent(/^0 items left out/u);
    });

    it('lists no item of a count kept for good and gives it no toggle', () => {
      render(<LeftOutLines counts={[KEPT]} />);

      expect(screen.queryByRole('button')).toBeNull();

      expect(screen.getByRole('list', { name: 'Left out of this list' })).toHaveTextContent(
        '3 items kept (2 MB): It is the version you use.',
      );
      expect(items()).toBeNull();
    });

    it('says no more when the core listed every item', () => {
      render(<LeftOutLines counts={[DEMO_LEFT_OUT]} open />);

      expect(within(items() as HTMLElement).getAllByRole('listitem')).toHaveLength(1);
      expect(screen.queryByText(/^And \d+ more/u)).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('shows the items when the toggle is pressed and hides them when it is pressed again', () => {
      render(<LeftOutLines counts={[TWO]} />);
      const button = toggle(/^2 items left out/u);

      fireEvent.click(button);

      expect(button).toHaveAttribute('aria-expanded', 'true');
      expect(button).toHaveAttribute('aria-controls', items()?.id);
      expect(within(items() as HTMLElement).getAllByRole('listitem')).toHaveLength(2);

      fireEvent.click(button);

      expect(button).toHaveAttribute('aria-expanded', 'false');
      expect(items()).toBeNull();
    });

    it('names the toggle by the whole sentence: the count, the size and the reason', () => {
      render(<LeftOutLines bar counts={[TWO]} />);

      expect(toggle(/./u)).toHaveAccessibleName(
        '2 items left out (5 MB): The app that made it is open. Quit the app, then scan again.',
      );
    });

    it('folds each line on its own', () => {
      const other = forNowCount('demo.caches', 'demo.error.failed', sized(MB), [
        leftOutEntry(named('music'), '/Users/someone/Library/Caches/music', sized(MB)),
      ]);
      render(<LeftOutLines counts={[TWO, other]} />);

      fireEvent.click(toggle(/^1 item left out/u));

      expect(toggle(/^2 items left out/u)).toHaveAttribute('aria-expanded', 'false');
      expect(screen.getAllByRole('list', { name: 'The items left out' })).toHaveLength(1);
      expect(screen.getByText('Example: music')).toBeVisible();
    });

    it('starts open when asked', () => {
      render(<LeftOutLines counts={[TWO]} open />);

      expect(toggle(/^2 items left out/u)).toHaveAttribute('aria-expanded', 'true');
      expect(screen.getByText('Example: editor')).toBeVisible();
    });

    it('shows a reason both kept for good and held back for now on two lines', () => {
      const warn = vi.spyOn(console, 'error').mockImplementation(() => undefined);
      const kept = keptCount('demo.caches', 'demo.left-out.open', 3, sized(2 * MB));
      render(<LeftOutLines counts={[kept, DEMO_LEFT_OUT]} />);

      expect(screen.getByText(/^3 items kept \(2 MB\)/u)).toBeInTheDocument();
      expect(screen.getByText(/^1 item left out \(157\.3 MB\)/u)).toBeInTheDocument();
      // React warns of two lines with one key.
      expect(warn).not.toHaveBeenCalled();
      warn.mockRestore();
    });

    it('shows one line per reason with its count and size, the ones kept for good last', () => {
      render(<LeftOutLines counts={[KEPT, DEMO_LEFT_OUT]} />);

      const list = screen.getByRole('list', { name: 'Left out of this list' });
      const [first] = within(list).getAllByRole('listitem');

      expect(first).toHaveTextContent(
        '1 item left out (157.3 MB): The app that made it is open. Quit the app, then scan again.',
      );
      expect(list).toHaveTextContent(/then scan again\..*3 items kept \(2 MB\): It is the version you use\.$/u);
    });

    it('lists each item held back by its name, its path from the home and its size, in the core order', () => {
      render(<LeftOutLines counts={[TWO]} home="/Users/someone" open />);

      const rows = within(items() as HTMLElement).getAllByRole('listitem');
      expect(rows).toHaveLength(2);
      expect(rows[0]).toHaveTextContent('Example: editor');
      expect(rows[0]).toHaveTextContent('~/Library/Caches/editor');
      expect(rows[0]).toHaveTextContent('4 MB');
      expect(rows[1]).toHaveTextContent('Example: notes');
    });

    it('says how many more items the core counted but did not list', () => {
      const count = forNowCount(
        'demo.caches',
        'demo.left-out.open',
        sized(9 * MB),
        [leftOutEntry(named('editor'), '/a/editor', sized(4 * MB))],
        13,
      );

      render(<LeftOutLines counts={[count]} />);
      fireEvent.click(toggle(/^13 items left out/u));

      expect(within(items() as HTMLElement).getByText('And 12 more items')).toBeVisible();
    });
  });
});
