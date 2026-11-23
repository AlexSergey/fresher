import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type {
  Coverage,
  Finding,
  LargestPage,
  LargestQuery,
  LargestRow,
  Overview,
  Refusal,
} from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, pushEvents, refuse, sampleAgreementAnswers, sampleStorageAnswers } from '../../ipc/mock';
import { type CoreStore, createCoreStore } from '../../state/store';
import { setMotion } from '../../test/animation';
import {
  ACCESS_DICTIONARY,
  accessStatus,
  DEMO_MODULES,
  DEMO_OVERVIEW,
  DEMO_SETTINGS,
  FULL_COVERAGE,
  IDLE,
  LIMITED_COVERAGE,
} from '../../test/demo-scan';
import { LARGEST_DICTIONARY, LARGEST_ROWS, largestPageOf, manyRows, ownFindingOf } from '../../test/largest-scan';
import { AppProviders } from '../app-providers';
import { LargestScreen } from './largest-screen';

const DICTIONARY = { ...LARGEST_DICTIONARY, ...ACCESS_DICTIONARY };

const queryOf = (args: unknown): LargestQuery => (args as { readonly query: LargestQuery }).query;

const overviewOf = (coverage: Coverage = FULL_COVERAGE, changedSince?: number, scan?: number): Overview => ({
  ...DEMO_OVERVIEW,
  lastScan: {
    ...DEMO_OVERVIEW.lastScan,
    coverage,
    ...(scan === undefined ? {} : { scan }),
    ...(changedSince === undefined ? {} : { changedSince }),
  } as NonNullable<Overview['lastScan']>,
});

// The screen over the mocked core; resolves when the core's first answers are in.
const show = async (answers: Partial<CoreAnswers> = {}, overview: Overview = overviewOf()): Promise<CoreStore> => {
  mockCore({
    ...sampleAgreementAnswers,
    getAccessStatus: () => accessStatus([[{ kind: 'all-files' }, 'granted']]),
    getDictionary: () => DICTIONARY,
    getLargest: (args) => largestPageOf(queryOf(args)),
    getModules: () => DEMO_MODULES,
    getOverview: () => overview,
    getSettings: () => DEMO_SETTINGS,
    ...sampleStorageAnswers,
    ...answers,
  });
  const store = createCoreStore();
  render(
    <AppProviders store={store}>
      <LargestScreen />
    </AppProviders>,
  );
  await waitFor(() => {
    expect(store.getState().overview).not.toBeNull();
    expect(store.getState().dictionary).not.toBeNull();
  });

  return store;
};

// The finding ids the mocked core answers for a picked node.
const pickOf = (rows: readonly LargestRow[] = LARGEST_ROWS): ReturnType<typeof vi.fn<(args: unknown) => Finding>> =>
  vi.fn((args: unknown): Finding => {
    const { node } = args as { readonly node: { readonly index: number } };
    const row = rows.find((one) => one.node.id.index === node.index) as LargestRow;

    return ownFindingOf(row, node.index);
  });

const REFUSAL = {
  error: { code: 'not-own-item' },
  text: { args: {}, key: 'core.error.not-own-item' },
} as unknown as Refusal;

const page = (rows: readonly LargestRow[], extra: Partial<LargestPage> = {}): LargestPage => ({
  ...largestPageOf({ kind: null, limit: 100, offset: 0, place: null, unchanged: 'any' }, rows),
  ...extra,
});

beforeEach(async () => {
  await setMotion('off');
});

afterEach(async () => {
  vi.restoreAllMocks();
  await setMotion(null);
});

describe('LargestScreen', () => {
  describe('negative cases', () => {
    it('before a scan it asks to scan first', async () => {
      const getLargest = vi.fn();
      await show({ getLargest }, IDLE);

      expect(await screen.findByText('Scan first to see your largest files.')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Scan' })).toBeInTheDocument();
      expect(getLargest).not.toHaveBeenCalled();
    });

    it('while a scan runs it says the list comes back', async () => {
      await show();
      act(() => {
        pushEvents({ modules: [], scan: 2, type: 'scan-started' });
      });

      expect(await screen.findByText('Scanning. The list comes back when the scan ends.')).toBeInTheDocument();
      expect(screen.queryByRole('list', { name: 'Largest files, largest first' })).not.toBeInTheDocument();
    });

    it('an empty answer says the minimum size', async () => {
      await show({ getLargest: () => page([]) });

      expect(await screen.findByText('Nothing here is 100 MB or larger.')).toBeInTheDocument();
      expect(screen.queryByRole('list', { name: 'Largest files, largest first' })).not.toBeInTheDocument();
    });

    it('a list that fits one page has no Show more and no truncated note', async () => {
      await show();

      await screen.findByText('ubuntu-24.04.dmg');

      expect(screen.queryByRole('button', { name: 'Show more' })).not.toBeInTheDocument();
      expect(screen.queryByText('Only the 10,000 largest are listed.')).not.toBeInTheDocument();
      expect(screen.queryByText(/A cleanup changed the disk/u)).not.toBeInTheDocument();
    });

    it('says it reads the list before the first answer', async () => {
      await show({ getLargest: () => new Promise<LargestPage>(() => undefined) });

      expect(await screen.findByText('Reading the list.')).toBeInTheDocument();
    });

    it('an empty list has no count in the header', async () => {
      await show({ getLargest: () => page([]) });

      await screen.findByText('Nothing here is 100 MB or larger.');

      expect(screen.queryByText(/^0 items/u)).not.toBeInTheDocument();
    });
  });

  describe('negative cases (selection)', () => {
    it('no row is ticked when the screen opens', async () => {
      const store = await show();
      await screen.findByText('ubuntu-24.04.dmg');

      for (const box of screen.getAllByRole('checkbox')) {
        expect(box).not.toBeChecked();
      }
      expect(store.getState().selection.findings.size).toBe(0);
      expect(screen.queryByRole('region', { name: /selected/iu })).not.toBeInTheDocument();
    });

    it('a withheld row cannot be ticked', async () => {
      await show();
      await screen.findByText('Photos Library.photoslibrary');

      expect(screen.queryByRole('checkbox', { name: /Photos Library/u })).not.toBeInTheDocument();
      expect(screen.getAllByRole('checkbox')).toHaveLength(4);
    });

    it('a refused pick leaves the row unticked', async () => {
      const store = await show({ pickOwnItem: refuse(REFUSAL) });

      fireEvent.click(await screen.findByRole('checkbox', { name: /ubuntu-24.04.dmg/u }));

      await waitFor(() => {
        expect(store.getState().failure).not.toBeNull();
      });
      expect(screen.getByRole('checkbox', { name: /ubuntu-24.04.dmg/u })).not.toBeChecked();
      expect(store.getState().selection.findings.size).toBe(0);
    });

    it('there is no select all', async () => {
      await show();
      await screen.findByText('ubuntu-24.04.dmg');

      expect(screen.queryByRole('checkbox', { name: /select all/iu })).not.toBeInTheDocument();
      expect(screen.queryByRole('button', { name: /select all/iu })).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('the rows come largest first as the core sent them', async () => {
      await show();

      const list = await screen.findByRole('list', { name: 'Largest files, largest first' });

      expect(
        within(list)
          .getAllByRole('listitem')
          .map((row) => within(row).getAllByText(/\.(dmg|pvm|photoslibrary|mov|zip)$/u)[0]?.textContent),
      ).toStrictEqual(LARGEST_ROWS.map(({ node }) => node.name));
    });

    it('shows the heading and the count with the size', async () => {
      await show();

      expect(await screen.findByRole('heading', { level: 1, name: 'Largest files and folders' })).toBeInTheDocument();
      expect(screen.getByText('5 items, 29.3 GB')).toBeInTheDocument();
    });

    it('show more asks the next page', async () => {
      const getLargest = vi.fn((args: unknown) => largestPageOf(queryOf(args), manyRows(150)));
      await show({ getLargest });
      await screen.findByText('file-1.bin');

      fireEvent.click(screen.getByRole('button', { name: 'Show more' }));

      await screen.findByText('file-150.bin');
      expect(queryOf(getLargest.mock.calls[1]?.[0]).offset).toBe(100);
      expect(screen.queryByRole('button', { name: 'Show more' })).not.toBeInTheDocument();
    });

    it('the coverage notices show what was not seen', async () => {
      await show({}, overviewOf(LIMITED_COVERAGE));

      expect(await screen.findByRole('region', { name: 'Full Disk Access' })).toBeInTheDocument();
    });

    it('the truncated note shows', async () => {
      await show({ getLargest: () => page(LARGEST_ROWS, { truncated: true }) });

      expect(await screen.findByText('Only the 10,000 largest are listed.')).toBeInTheDocument();
    });

    it('the out-of-date note shows after a cleanup', async () => {
      await show({}, overviewOf(FULL_COVERAGE, 7));

      expect(
        await screen.findByText('A cleanup changed the disk since this scan. Scan again for exact sizes.'),
      ).toBeInTheDocument();
    });

    it('the list is asked again when findings were removed', async () => {
      const getLargest = vi.fn((args: unknown) => largestPageOf(queryOf(args)));
      await show({ getLargest });
      await screen.findByText('ubuntu-24.04.dmg');

      act(() => {
        pushEvents({ findings: [1], scan: 1, type: 'findings-removed' });
      });

      await waitFor(() => {
        expect(getLargest).toHaveBeenCalledTimes(2);
      });
    });

    it('ticking a row picks it and puts its finding in the selection', async () => {
      const pickOwnItem = pickOf();
      const store = await show({ pickOwnItem });

      fireEvent.click(await screen.findByRole('checkbox', { name: /ubuntu-24.04.dmg/u }));

      await waitFor(() => {
        expect(screen.getByRole('checkbox', { name: /ubuntu-24.04.dmg/u })).toBeChecked();
      });
      expect(pickOwnItem).toHaveBeenCalledTimes(1);
      expect(pickOwnItem.mock.calls[0]?.[0]).toStrictEqual({ node: { index: 101, state: 1 } });
      expect([...store.getState().selection.findings.values()].map(({ category }) => category)).toStrictEqual([
        'core.own-file',
      ]);
    });

    it('unticking a row takes it out without asking the core', async () => {
      const pickOwnItem = pickOf();
      const store = await show({ pickOwnItem });
      fireEvent.click(await screen.findByRole('checkbox', { name: /ubuntu-24.04.dmg/u }));
      await waitFor(() => {
        expect(screen.getByRole('checkbox', { name: /ubuntu-24.04.dmg/u })).toBeChecked();
      });

      fireEvent.click(screen.getByRole('checkbox', { name: /ubuntu-24.04.dmg/u }));

      await waitFor(() => {
        expect(screen.getByRole('checkbox', { name: /ubuntu-24.04.dmg/u })).not.toBeChecked();
      });
      expect(pickOwnItem).toHaveBeenCalledTimes(1);
      expect(store.getState().selection.findings.size).toBe(0);
    });

    it('a filter asks the first page again', async () => {
      const getLargest = vi.fn((args: unknown) => largestPageOf(queryOf(args)));
      await show({ getLargest });
      fireEvent.change(await screen.findByRole('combobox', { name: 'Not changed for' }), {
        target: { value: 'year' },
      });

      await waitFor(() => {
        expect(getLargest).toHaveBeenCalledTimes(2);
      });
      expect(queryOf(getLargest.mock.calls[1]?.[0])).toStrictEqual({
        kind: null,
        limit: 100,
        offset: 0,
        place: null,
        unchanged: 'year',
      });
    });

    it('the filters list the core kinds and places with their counts', async () => {
      const folder = { ...LARGEST_ROWS[0]?.node, name: 'Downloads' } as LargestRow['node'];
      await show({
        getLargest: () => ({
          ...page(LARGEST_ROWS),
          kinds: [
            { count: 2, kind: 'video' },
            { count: 1, kind: 'archive' },
          ],
          places: [{ count: 3, folder }],
        }),
      });

      const kinds = await screen.findByRole('combobox', { name: 'Kind' });
      const places = screen.getByRole('combobox', { name: 'Place' });

      expect(
        within(kinds)
          .getAllByRole('option')
          .map((option) => option.textContent),
      ).toStrictEqual(['Any kind', 'Video (2)', 'Archive (1)']);
      expect(
        within(places)
          .getAllByRole('option')
          .map((option) => option.textContent),
      ).toStrictEqual(['Anywhere in your home', '~/Downloads (3)']);
      expect(
        within(screen.getByRole('combobox', { name: 'Not changed for' }))
          .getAllByRole('option')
          .map((option) => option.textContent),
      ).toContain('Not changed for a year');
    });

    it('the selection bar shows the picked total', async () => {
      await show({ pickOwnItem: pickOf() });

      fireEvent.click(await screen.findByRole('checkbox', { name: /ubuntu-24.04.dmg/u }));

      expect(await screen.findByText(/1 item/u)).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /Review/u })).toBeInTheDocument();
    });

    it('a new scan drops the chosen place, whose folder is gone', async () => {
      const folder = { ...LARGEST_ROWS[0]?.node, name: 'Downloads' } as LargestRow['node'];
      const getLargest = vi.fn((args: unknown) => ({
        ...largestPageOf(queryOf(args)),
        places: [{ count: 3, folder }],
      }));
      const store = await show({ getLargest });
      fireEvent.change(await screen.findByRole('combobox', { name: 'Place' }), {
        target: { value: `${String(folder.id.state)}:${String(folder.id.index)}` },
      });
      await waitFor(() => {
        expect(queryOf(getLargest.mock.calls.at(-1)?.[0]).place).toStrictEqual(folder.id);
      });

      act(() => {
        store.dispatch({ overview: overviewOf(FULL_COVERAGE, undefined, 2), type: 'overview-loaded' });
      });

      await waitFor(() => {
        expect(queryOf(getLargest.mock.calls.at(-1)?.[0]).place).toBeNull();
      });
      expect(screen.getByRole('combobox', { name: 'Place' })).toHaveValue('any');
    });
  });
});
