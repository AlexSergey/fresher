import { render, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

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

beforeEach(() => {
  setCoreDictionary({ ...DEMO_DICTIONARY, 'demo.error.failed': 'It is the version you use.' });
});

describe('LeftOutLines', () => {
  describe('negative cases', () => {
    it('shows no line without a left-out', () => {
      render(<LeftOutLines counts={[]} />);

      expect(screen.queryByRole('list')).not.toBeInTheDocument();
    });

    it('lists no item of a count kept for good', () => {
      render(<LeftOutLines counts={[KEPT]} />);

      expect(screen.getByRole('list', { name: 'Left out of this list' })).toHaveTextContent(
        '3 items kept (2 MB): It is the version you use.',
      );
      expect(items()).toBeNull();
    });

    it('says no more when the core listed every item', () => {
      render(<LeftOutLines counts={[DEMO_LEFT_OUT]} />);

      expect(within(items() as HTMLElement).getAllByRole('listitem')).toHaveLength(1);
      expect(screen.queryByText(/^And \d+ more/u)).toBeNull();
    });
  });

  describe('positive cases', () => {
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
      const count = forNowCount('demo.caches', 'demo.left-out.open', sized(5 * MB), [
        leftOutEntry(named('editor'), '/Users/someone/Library/Caches/editor', sized(4 * MB)),
        leftOutEntry(named('notes'), '/Users/someone/Library/Caches/notes', sized(MB)),
      ]);

      render(<LeftOutLines counts={[count]} home="/Users/someone" />);

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

      expect(within(items() as HTMLElement).getByText('And 12 more items')).toBeVisible();
    });
  });
});
