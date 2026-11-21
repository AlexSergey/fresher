import { fireEvent, render, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import type { CategoryTotal, MapPart, MeaningGroup, VolumeMap } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { SAMPLE_STORAGE_MAP, STORAGE_DICTIONARY } from '../../test/storage-map';
import { explainedTone } from './parts';
import { StorageMap } from './storage-map';

const GB = 1_000_000_000;

const SAMPLE = SAMPLE_STORAGE_MAP.volumes[0] as VolumeMap;

const FREE: MapPart['sizes'] = { allocated: 1, completeness: 'complete', logical: 1, reclaimable: 1 };

const part = (
  kind: MapPart['kind'],
  gigabytes: number,
  completeness: MapPart['sizes']['completeness'] = 'complete',
): MapPart => ({
  kind,
  sizes: { allocated: gigabytes * GB, completeness, logical: gigabytes * GB, reclaimable: gigabytes * GB },
});

const withParts = (parts: readonly MapPart[], overrides: Partial<VolumeMap> = {}): VolumeMap => ({
  ...SAMPLE,
  parts: [...parts],
  ...overrides,
});

const renderMap = (volume: VolumeMap = SAMPLE): void => {
  render(<StorageMap explained="low" scanning={false} volume={volume} />);
};

// The lines of the map, by the part each names.
const lines = (): readonly HTMLElement[] =>
  within(screen.getByRole('list', { name: 'Parts of Macintosh HD' })).getAllByRole('listitem');

const lineOf = (name: string): HTMLElement => {
  const line = lines().find((item) => within(item).queryByText(name, { exact: true }) !== null);

  if (line === undefined) {
    throw new Error(`No line names ${name}`);
  }

  return line;
};

const openDetails = (name: string): HTMLElement => {
  const line = lineOf(name);
  fireEvent.click(within(line).getByRole('button', { name: 'Show details' }));

  return line;
};

beforeEach(() => {
  setCoreDictionary(STORAGE_DICTIONARY);
});

describe('StorageMap', () => {
  describe('negative cases', () => {
    it('a zero part has no line', () => {
      renderMap(withParts([part({ kind: 'explained' }, 10), part({ kind: 'mac-os' }, 0), part({ kind: 'free' }, 5)]));

      expect(lines()).toHaveLength(2);
      expect(screen.queryByText('macOS')).toBeNull();
    });

    it('no part but Explained offers a cleanup', () => {
      renderMap();

      for (const line of lines()) {
        fireEvent.click(within(line).getByRole('button', { name: 'Show details' }));
      }

      expect(screen.queryByRole('checkbox')).toBeNull();
      expect(screen.getAllByText(/Freed if removed/u)).toHaveLength(1);
      expect(lineOf('Explained')).toHaveTextContent('Freed if removed: 58 GB');
      expect(screen.queryByRole('button', { name: /clean|remove|trash/iu })).toBeNull();
    });

    it('the remainder lists its reasons, not a size per reason', () => {
      renderMap(
        withParts([
          part(
            {
              kind: 'remainder',
              outsideScan: false,
              reasons: [
                { folders: 3, kind: 'unreadable', needs: { kind: 'all-files' } },
                { kind: 'snapshots-and-bookkeeping' },
                { kind: 'changed-during-scan' },
              ],
            },
            120,
          ),
          part({ kind: 'free' }, 10),
        ]),
      );

      const line = openDetails('Not measured');
      const reasons = within(line).getAllByRole('listitem');

      expect(reasons.map((reason) => reason.textContent)).toStrictEqual([
        'Folders Fresher could not read. Allowing access lets the next scan measure them. 3 folders could not be ' +
          'read. Full Disk Access would let the scan read them.',
        "Local snapshots and the file system's own records, which macOS looks after.",
        'Files changed on the disk while the scan ran.',
      ]);
      for (const reason of reasons) {
        expect(reason).not.toHaveTextContent(/\d\s?(?:B|kB|MB|GB|TB)\b/u);
      }
    });

    it('Explained ignores the categories of another volume', () => {
      const totals: readonly CategoryTotal[] = [
        { category: 'demo.caches', count: 1, highestDanger: 'safe', sizes: SAMPLE.parts[0]?.sizes ?? FREE },
        { category: 'demo.volumes', count: 1, highestDanger: 'high', sizes: SAMPLE.parts[0]?.sizes ?? FREE },
      ];
      const groups: readonly MeaningGroup[] = [{ kind: 'category', value: { category: 'demo.caches', sizes: FREE } }];

      expect(explainedTone(totals, groups)).toBe('safe');
      expect(explainedTone(totals, null)).toBe('high');
    });

    it('no text says Other', () => {
      renderMap(
        withParts([
          ...SAMPLE.parts.slice(0, -1),
          part({ kind: 'other-volume', name: '' }, 1),
          part({ kind: 'free' }, 3),
        ]),
      );

      for (const line of lines()) {
        fireEvent.click(within(line).getByRole('button', { name: 'Show details' }));
      }

      expect(document.body).not.toHaveTextContent(/\bOther\b/u);
      expect(screen.getAllByText('Another volume').length).toBeGreaterThan(0);
    });
  });

  describe('positive cases', () => {
    it('names the volume by its name and kind', () => {
      renderMap();

      expect(screen.getByRole('heading', { level: 2, name: 'Macintosh HD · internal' })).toBeInTheDocument();
    });

    it('the parts add up to the bar', () => {
      renderMap();

      const segments = [
        // eslint-disable-next-line testing-library/no-node-access -- the segments are decoration with no role to query by
        ...screen.getByRole('img', { name: /^Space on Macintosh HD/u }).querySelectorAll<HTMLElement>('[data-tone]'),
      ];
      const widths = segments.map((segment) => Number.parseFloat(segment.style.width));

      expect(widths).toHaveLength(SAMPLE.parts.length);
      expect(widths.reduce((sum, width) => sum + width, 0)).toBeCloseTo(100, 1);
    });

    it('every part is drawn in its own tone, Explained in the one passed', () => {
      const kinds = ['your-folders', 'mac-os', 'swap-and-sleep', 'disk-own', 'free'] as const;

      renderMap(
        withParts([
          part({ kind: 'explained' }, 10),
          ...kinds.slice(0, 3).map((kind) => part({ kind }, 5)),
          part({ kind: 'other-volume', name: 'Backups' }, 5),
          ...kinds.slice(3).map((kind) => part({ kind }, 5)),
        ]),
      );

      const tones = [
        // eslint-disable-next-line testing-library/no-node-access -- the segments are decoration with no role to query by
        ...screen.getByRole('img', { name: /^Space on Macintosh HD/u }).querySelectorAll<HTMLElement>('[data-tone]'),
      ].map((segment) => segment.dataset.tone);

      expect(tones).toStrictEqual([
        'low',
        'your-folders',
        'mac-os',
        'swap-and-sleep',
        'other-volume',
        'disk-own',
        'free',
      ]);
    });

    it('a partial part shows a plus', () => {
      renderMap(withParts([part({ kind: 'your-folders' }, 42, 'partial'), part({ kind: 'free' }, 8)]));

      expect(lineOf('Your folders')).toHaveTextContent('42 GB+');
      expect(within(lineOf('Your folders')).getByText('at least 42 GB')).toBeInTheDocument();
    });

    it('while scanning, a partial size still reads as a lower bound', () => {
      render(
        <StorageMap
          explained="safe"
          scanning
          volume={withParts([part({ kind: 'your-folders' }, 42, 'partial'), part({ kind: 'free' }, 8)])}
        />,
      );

      expect(within(lineOf('Your folders')).getByText('at least 42 GB')).toBeInTheDocument();
    });

    it('freed if removed is on demand', () => {
      renderMap();
      const line = lineOf('Explained');

      expect(within(line).getByText(/Freed if removed/u)).not.toBeVisible();

      fireEvent.click(within(line).getByRole('button', { name: 'Show details' }));

      expect(within(line).getByText('Freed if removed: 58 GB')).toBeVisible();
      expect(within(line).getByRole('button', { name: 'Show details' })).toHaveAttribute('aria-expanded', 'true');
    });

    it('purgeable is beside Free', () => {
      renderMap();

      expect(lineOf('Free')).toHaveTextContent('macOS can free about 18 GB more on its own when space runs low');
      expect(lineOf('Free')).toHaveTextContent('109 GB');
    });

    it('a disk that changed during the scan says so', () => {
      renderMap({ ...SAMPLE, diskChanged: true });

      expect(screen.getByText(/The disk changed during the scan/u)).toBeVisible();
    });

    it('a collapsible volume shows its name until it is opened', () => {
      render(<StorageMap collapsible explained="safe" scanning={false} volume={SAMPLE} />);
      const toggle = screen.getByRole('button', { name: 'Macintosh HD · internal' });

      expect(screen.queryByRole('list', { name: 'Parts of Macintosh HD' })).toBeNull();

      fireEvent.click(toggle);

      expect(screen.getByRole('list', { name: 'Parts of Macintosh HD' })).toBeVisible();
    });
  });
});
