import { fireEvent, render, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type * as i18n from '../../i18n';

import { setCoreDictionary } from '../../i18n';
import { STORAGE_DICTIONARY } from '../../test/storage-map';
import { DiskCard, type DiskCardProps } from './disk-card';
import { DISK_VOLUME, DISK_VOLUME_CHANGED, DISK_VOLUME_NOTHING } from './story-data';

// The free line as the dictionary words it; a test may word it without the free size.
const freeLine = vi.hoisted(() => ({ override: undefined as string | undefined }));

vi.mock('../../i18n', async (importOriginal) => {
  const original = await importOriginal<typeof i18n>();

  return {
    ...original,
    t: (...args: Parameters<typeof i18n.t>): string =>
      args[0] === 'ui.disk-card.free-of' && freeLine.override !== undefined ? freeLine.override : original.t(...args),
  };
});

const mountCard = (overrides: Partial<DiskCardProps> = {}): DiskCardProps => {
  const props: DiskCardProps = {
    failure: null,
    onScan: vi.fn(),
    phase: 'finished',
    volume: DISK_VOLUME,
    ...overrides,
  };

  render(<DiskCard {...props} />);

  return props;
};

// The ring's drawing has no roles: its arcs are found by their markers.
/* eslint-disable testing-library/no-node-access -- the ring's arcs are decoration, without roles */
const arcs = (): readonly Element[] => [...document.querySelectorAll('[data-ring="arcs"] [data-tone]')];
const arcOf = (tone: string): Element | undefined => arcs().find((arc) => arc.getAttribute('data-tone') === tone);
/* eslint-enable testing-library/no-node-access */

const rowOf = (name: string): HTMLElement => {
  const row = screen.getAllByRole('listitem').find((item) => within(item).queryByText(name) !== null);

  if (row === undefined) {
    throw new Error(`no row for ${name}`);
  }

  return row;
};

beforeEach(() => {
  freeLine.override = undefined;
  setCoreDictionary({ ...STORAGE_DICTIONARY, 'demo.error.failed': 'The demo module could not build its findings.' });
});

describe('DiskCard', () => {
  describe('negative cases', () => {
    it('a part of zero size draws no arc but keeps its legend row', () => {
      mountCard({ volume: DISK_VOLUME_NOTHING });

      expect(arcOf('can-be-freed')).toBeUndefined();
      expect(arcs()).toHaveLength(3);
      expect(rowOf('Can be freed')).toBeInTheDocument();
    });

    it('before a scan Can be freed reads Not scanned yet and the centre is Scan', () => {
      const { onScan } = mountCard({ phase: 'idle' });

      expect(within(rowOf('Can be freed')).getByText('Not scanned yet')).toBeInTheDocument();
      expect(screen.queryByText('can be freed')).not.toBeInTheDocument();
      expect(arcs()).toHaveLength(0);

      fireEvent.click(screen.getByRole('button', { name: 'Scan' }));

      expect(onScan).toHaveBeenCalledTimes(1);
    });

    it('a compact card holds no Scan before a scan and is named by its volume', () => {
      mountCard({ compact: true, phase: 'idle' });

      expect(screen.queryByRole('button', { name: 'Scan' })).toBeNull();
      expect(screen.getByRole('region', { name: 'Macintosh HD' })).toBeInTheDocument();
    });

    it('a failed scan shows the core text and Try again, no legend', () => {
      const { onScan } = mountCard({ failure: { args: {}, key: 'demo.error.failed' }, phase: 'failed' });

      expect(screen.getByText('The demo module could not build its findings.')).toBeInTheDocument();
      expect(screen.queryByRole('list', { name: 'Parts of the disk' })).not.toBeInTheDocument();

      fireEvent.click(screen.getByRole('button', { name: 'Try again' }));

      expect(onScan).toHaveBeenCalledTimes(1);
    });

    it('shows no disk-changed note while the disk did not change', () => {
      mountCard();

      expect(screen.queryByText(/^The disk changed during the scan/u)).not.toBeInTheDocument();
    });

    it('shows no explanation until an i is pressed', () => {
      mountCard();

      expect(screen.getByRole('button', { name: 'What is macOS?' })).toHaveAttribute('aria-expanded', 'false');
      expect(screen.getByText(/which macOS looks after/u)).not.toBeVisible();
    });

    it('a free line without the free size shows plain, nothing bold', () => {
      freeLine.override = 'Free space unknown';
      mountCard();

      expect(screen.getByText('Free space unknown')).toBeInTheDocument();
      expect(screen.queryByText('109 GB', { selector: 'b' })).not.toBeInTheDocument();
    });

    it('Free has no purgeable note when the core gives none', () => {
      mountCard({ volume: DISK_VOLUME_NOTHING });

      fireEvent.click(screen.getByRole('button', { name: 'What is Free?' }));

      expect(screen.queryByText(/macOS can free about/u)).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('the legend lists the five parts in the core order with their names and sizes', () => {
      mountCard();

      const rows = screen.getAllByRole('listitem');

      expect(rows.map((row) => row.getAttribute('data-key'))).toEqual([
        'can-be-freed',
        'files-and-apps',
        'mac-os',
        'system-space',
        'free',
      ]);
      expect(rows[0]).toHaveTextContent('Can be freed3.8 GB');
      expect(rows[1]).toHaveTextContent('Your files and apps353 GB');
      expect(rows[4]).toHaveTextContent('Free109 GB');
    });

    it('draws the four parts as arcs and leaves Free as the track', () => {
      mountCard();

      expect(arcs().map((arc) => arc.getAttribute('data-tone'))).toEqual([
        'files-and-apps',
        'mac-os',
        'system-space',
        'can-be-freed',
      ]);
    });

    it("the ring's name reads every part", () => {
      mountCard();

      expect(screen.getByRole('img', { name: /^Macintosh HD: / })).toHaveAccessibleName(
        'Macintosh HD: Can be freed 3.8 GB, Your files and apps 353 GB, macOS 23 GB, System space 5.2 GB, Free 109 GB',
      );
    });

    it('shows what can be freed in the centre', () => {
      mountCard();

      expect(screen.getAllByText('3.8 GB').length).toBeGreaterThan(1);
      expect(screen.getByText('can be freed')).toBeInTheDocument();
    });

    it('shows the name and the free space of the disk', () => {
      mountCard();

      expect(screen.getAllByText('Macintosh HD').length).toBeGreaterThan(0);
      expect(screen.getByText('109 GB', { selector: 'b' })).toBeInTheDocument();
      expect(screen.getByText(/free of 494 GB/u)).toBeInTheDocument();
    });

    it('pointing at a row highlights its arc, and the other way', () => {
      mountCard();

      fireEvent.pointerEnter(rowOf('macOS'));

      expect(arcOf('mac-os')).not.toHaveAttribute('data-dimmed');
      expect(arcs().filter((arc) => arc.hasAttribute('data-dimmed'))).toHaveLength(3);

      fireEvent.pointerLeave(rowOf('macOS'));

      expect(arcs().filter((arc) => arc.hasAttribute('data-dimmed'))).toHaveLength(0);

      fireEvent.pointerEnter(arcOf('system-space') as Element);

      expect(rowOf('System space')).toHaveAttribute('data-highlighted');

      fireEvent.pointerLeave(arcOf('system-space') as Element);

      expect(rowOf('System space')).not.toHaveAttribute('data-highlighted');
    });

    it('the i of a part shows the core description', () => {
      mountCard();

      const about = screen.getByRole('button', { name: 'What is macOS?' });

      fireEvent.click(about);

      expect(about).toHaveAttribute('aria-expanded', 'true');
      expect(screen.getByText(/which macOS looks after/u)).toBeVisible();
    });

    it("Free's i adds the purgeable note", () => {
      mountCard();

      fireEvent.click(screen.getByRole('button', { name: 'What is Free?' }));

      expect(screen.getByText('Space you can still use for new files.')).toBeVisible();
      expect(screen.getByText(/macOS can free about 18 GB more/u)).toBeVisible();
    });

    it('shows the disk-changed note under the legend', () => {
      mountCard({ volume: DISK_VOLUME_CHANGED });

      expect(screen.getByText(/^The disk changed during the scan/u)).toBeInTheDocument();
    });

    it('compact draws the smaller ring', () => {
      mountCard({ compact: true });

      expect(screen.getByRole('img', { name: /^Macintosh HD: / })).toHaveAttribute('width', '86');
    });

    it('the full card draws the 180 px ring', () => {
      mountCard();

      expect(screen.getByRole('img', { name: /^Macintosh HD: / })).toHaveAttribute('width', '164');
    });
  });
});
