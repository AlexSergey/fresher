import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { LargestRow, OwnRow } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { LARGEST_DICTIONARY, LARGEST_ROWS } from '../../test/largest-scan';
import { OwnItemRow } from './own-item-row';

const [IMAGE, , PART, , ARCHIVE] = LARGEST_ROWS as readonly [
  LargestRow,
  LargestRow,
  LargestRow,
  LargestRow,
  LargestRow,
];

const FOLDER: OwnRow = {
  explained: { allocated: 3_100_000_000, completeness: 'complete', logical: 0, reclaimable: 0 },
  kind: 'folder',
  node: { ...IMAGE.node, kind: 'folder', name: 'Developer' },
  offer: { kind: 'withheld', reason: 'holds-explained' },
  opens: true,
};

// Whether the row keeps the empty place of a checkbox: a decoration with no role to query by.
const keepsCheckboxPlace = (): boolean =>
  // eslint-disable-next-line testing-library/no-node-access -- the place is empty decoration with no role to query by
  document.querySelector('[class*="noCheckbox"]') !== null;

const inList = (row: React.JSX.Element): void => {
  render(<div role="list">{row}</div>);
};

beforeEach(() => {
  setCoreDictionary(LARGEST_DICTIONARY);
});

describe('OwnItemRow', () => {
  describe('negative cases', () => {
    it('a withheld row has no checkbox and says why', () => {
      inList(<OwnItemRow checked={false} onToggle={vi.fn()} row={PART} />);

      expect(screen.queryByRole('checkbox')).not.toBeInTheDocument();
      expect(screen.getByText('Fresher could not read all of it, so it cannot tell what it holds.')).toBeVisible();
    });

    it('a row with no time says nothing of when', () => {
      inList(<OwnItemRow checked={false} onToggle={vi.fn()} row={ARCHIVE} />);

      expect(screen.queryByText(/Changed/u)).not.toBeInTheDocument();
    });

    it('a folder that does not open has no Open button', () => {
      inList(<OwnItemRow checked={false} row={{ ...FOLDER, opens: false }} />);

      expect(screen.queryByRole('button', { name: 'Open' })).not.toBeInTheDocument();
    });

    it('a row without a toggle has no checkbox', () => {
      inList(<OwnItemRow checked={false} row={IMAGE} />);

      expect(screen.queryByRole('checkbox')).not.toBeInTheDocument();
    });

    it('a Browse row has no checkbox', () => {
      inList(<OwnItemRow onReveal={vi.fn()} row={FOLDER} />);

      expect(screen.queryByRole('checkbox')).not.toBeInTheDocument();
    });

    it('a Browse row has no empty checkbox place', () => {
      inList(<OwnItemRow onReveal={vi.fn()} row={FOLDER} />);

      expect(keepsCheckboxPlace()).toBe(false);
    });

    it('a folder says nothing of Scan when Scan explains none of it', () => {
      inList(<OwnItemRow checked={false} row={{ ...FOLDER, explained: { ...FOLDER.explained, allocated: 0 } }} />);

      expect(screen.queryByText(/Can be freed on Scan/u)).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('an offered row shows its name, kind, place and size', () => {
      inList(<OwnItemRow checked={false} onToggle={vi.fn()} row={IMAGE} />);

      expect(screen.getByText('ubuntu-24.04.dmg')).toBeVisible();
      expect(screen.getByText('Disk image ·')).toBeVisible();
      expect(screen.getByText(/^· Changed /u)).toBeVisible();
      expect(screen.getByText('~/Downloads', { selector: '[aria-hidden]' })).toBeInTheDocument();
      expect(screen.getAllByText('12 GB')).toHaveLength(2);
    });

    it('an archive names the folder beside it', () => {
      inList(<OwnItemRow checked={false} onToggle={vi.fn()} row={ARCHIVE} />);

      expect(
        screen.getByText('A folder named project-backup beside it may hold what was unpacked from it.'),
      ).toBeVisible();
    });

    it('the checkbox is labelled by the row', () => {
      const onToggle = vi.fn();
      inList(<OwnItemRow checked onToggle={onToggle} row={IMAGE} />);

      const box = screen.getByRole('checkbox', { name: 'Choose ubuntu-24.04.dmg, 12 GB' });
      expect(box).toBeChecked();

      fireEvent.click(box);

      expect(onToggle).toHaveBeenCalledOnce();
    });

    it('a folder shows what Scan explains of it and opens', () => {
      const onOpen = vi.fn();
      inList(<OwnItemRow checked={false} onOpen={onOpen} row={FOLDER} />);

      expect(screen.getByText(/Folder · Can be freed on Scan: 3.1 GB/u)).toBeVisible();

      fireEvent.click(screen.getByRole('button', { name: 'Open' }));

      expect(onOpen).toHaveBeenCalledOnce();
    });

    it('a Browse row shows why it does not open', () => {
      inList(
        <OwnItemRow
          onReveal={vi.fn()}
          row={{ ...FOLDER, offer: { kind: 'withheld', reason: 'trash' }, opens: false }}
        />,
      );

      expect(screen.getByText(/Trash/u)).toBeVisible();
      expect(screen.queryByRole('button', { name: 'Open' })).not.toBeInTheDocument();
    });

    it('a Browse row calls onReveal from Show in Finder', () => {
      const onReveal = vi.fn();
      inList(<OwnItemRow onReveal={onReveal} row={FOLDER} />);

      fireEvent.click(screen.getByRole('button', { name: 'Show in Finder: Developer' }));

      expect(onReveal).toHaveBeenCalledOnce();
      expect(screen.getByText('Show in Finder')).toBeVisible();
    });

    it('a withheld Files row keeps the empty checkbox place', () => {
      inList(<OwnItemRow row={PART} />);

      expect(keepsCheckboxPlace()).toBe(true);
    });

    it('a Files row still has its checkbox', () => {
      inList(<OwnItemRow checked={false} onToggle={vi.fn()} row={IMAGE} />);

      expect(screen.getByRole('checkbox')).toBeVisible();
    });

    it('a withheld folder keeps its Open button and says why', () => {
      inList(<OwnItemRow checked={false} onOpen={vi.fn()} row={FOLDER} />);

      expect(screen.getByText('Part of it is on Scan. Open it to choose the rest.')).toBeVisible();
      expect(screen.getByRole('button', { name: 'Open' })).toBeVisible();
    });
  });
});
