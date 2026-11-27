import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { FolderRow as FolderRowData } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { FOLDERS, LARGEST_DICTIONARY } from '../../test/largest-scan';
import { FolderRow } from './folder-row';

const ROWS: readonly FolderRowData[] = FOLDERS.rows;
const [PLAIN, PROJECT] = ROWS as readonly [FolderRowData, FolderRowData];

const inList = (row: React.JSX.Element): void => {
  render(<div role="list">{row}</div>);
};

beforeEach(() => {
  setCoreDictionary(LARGEST_DICTIONARY);
});

describe('FolderRow', () => {
  describe('negative cases', () => {
    it('shows no checkbox', () => {
      inList(<FolderRow onReveal={vi.fn()} row={PLAIN} />);

      expect(screen.queryByRole('checkbox')).not.toBeInTheDocument();
    });

    it('shows no project badge for a folder that is no project', () => {
      inList(<FolderRow onReveal={vi.fn()} row={PLAIN} />);

      expect(screen.queryByText('Project')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('shows the name, the place and the size', () => {
      inList(<FolderRow onReveal={vi.fn()} row={PROJECT} />);

      expect(screen.getByText('old-app')).toBeVisible();
      expect(screen.getByTitle('~/Code')).toBeVisible();
      // The size is drawn once and read once: the first match is the drawn one.
      expect(screen.getAllByText('13 GB')[0]).toBeVisible();
    });

    it('says the months it was left alone', () => {
      inList(<FolderRow onReveal={vi.fn()} row={PROJECT} />);

      expect(screen.getByText('Not changed for 14 months. Open it in the Finder to decide.')).toBeVisible();
    });

    it('says the years it was left alone', () => {
      inList(<FolderRow onReveal={vi.fn()} row={PLAIN} />);

      expect(screen.getByText('Not changed for 3 years. Open it in the Finder to decide.')).toBeVisible();
    });

    it('says one month in the singular', () => {
      inList(<FolderRow onReveal={vi.fn()} row={{ ...PLAIN, leftFor: { count: 1, unit: 'months' } }} />);

      expect(screen.getByText('Not changed for 1 month. Open it in the Finder to decide.')).toBeVisible();
    });

    it('shows the project badge', () => {
      inList(<FolderRow onReveal={vi.fn()} row={PROJECT} />);

      expect(screen.getByText('Project')).toBeVisible();
    });

    it('calls onReveal from Show in Finder, named after the folder', () => {
      const onReveal = vi.fn();
      inList(<FolderRow onReveal={onReveal} row={PLAIN} />);

      fireEvent.click(screen.getByRole('button', { name: 'Show in Finder: Archive' }));

      expect(onReveal).toHaveBeenCalledOnce();
    });
  });
});
