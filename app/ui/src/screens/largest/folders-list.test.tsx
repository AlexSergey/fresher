import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { FoldersPage, Refusal } from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, refuse, sampleAgreementAnswers } from '../../ipc/mock';
import { type CoreStore, createCoreStore } from '../../state/store';
import { setMotion } from '../../test/animation';
import { DEMO_OVERVIEW } from '../../test/demo-scan';
import { foldersPageOf, LARGEST_DICTIONARY, manyFolders } from '../../test/largest-scan';
import { AppProviders } from '../app-providers';
import { FoldersList } from './folders-list';
import { type LargestFolders } from './use-largest-folders';

// The five folders and three smaller ones, the first page of five.
const PAGE: FoldersPage = foldersPageOf({ limit: 5, offset: 0 }, manyFolders(3));

const STALE: Refusal = { error: { code: 'stale-node' }, text: { args: {}, key: 'core.error.stale-node' } };

const show = async (folders: Partial<LargestFolders>, answers: Partial<CoreAnswers> = {}): Promise<CoreStore> => {
  mockCore({
    ...sampleAgreementAnswers,
    getDictionary: () => LARGEST_DICTIONARY,
    getOverview: () => DEMO_OVERVIEW,
    ...answers,
  });
  const store = createCoreStore();
  render(
    <AppProviders store={store}>
      <FoldersList folders={{ failed: false, more: () => undefined, page: PAGE, ...folders }} />
    </AppProviders>,
  );
  await waitFor(() => {
    expect(store.getState().dictionary).not.toBeNull();
  });

  return store;
};

beforeEach(async () => {
  await setMotion('off');
});

afterEach(async () => {
  vi.restoreAllMocks();
  await setMotion(null);
});

describe('FoldersList', () => {
  describe('negative cases', () => {
    it('shows the empty text with the size and the days', async () => {
      await show({ page: foldersPageOf({ limit: 100, offset: 0 }, []) });

      expect(screen.getByText('No folder of 1 GB or more has gone unchanged for 365 days.')).toBeInTheDocument();
      expect(screen.queryByRole('heading')).not.toBeInTheDocument();
      expect(screen.queryByRole('list')).not.toBeInTheDocument();
    });

    it('says the list could not be read', async () => {
      await show({ failed: true, page: null });

      expect(screen.getByText('The list could not be read.')).toBeInTheDocument();
      expect(screen.queryByRole('list')).not.toBeInTheDocument();
    });

    it('says it reads the list before the first answer', async () => {
      await show({ page: null });

      expect(screen.getByText('Reading the list.')).toBeInTheDocument();
    });

    it('reports a refused reveal', async () => {
      const store = await show({}, { revealNode: refuse(STALE) });

      fireEvent.click(screen.getByRole('button', { name: 'Show in Finder: Archive' }));

      await waitFor(() => {
        expect(store.getState().failure?.kind).toBe('refused');
      });
    });

    it('has no checkbox', async () => {
      await show({});

      expect(screen.queryByRole('checkbox')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('shows the heading, the rule and the tiles', async () => {
      await show({});

      expect(
        screen.getByRole('heading', { level: 2, name: 'Open these in the Finder and decide what to keep' }),
      ).toBeInTheDocument();
      expect(
        screen.getByText('Folders of 1 GB or more where nothing changed for 365 days. Fresher never removes them.'),
      ).toBeInTheDocument();
      expect(screen.getByText('In this list')).toBeInTheDocument();
      expect(
        screen
          .getAllByRole('listitem')
          .slice(-8, -5)
          .map((tile) => tile.textContent),
      ).toStrictEqual(['Folders8', 'Space50 GB', 'Projects5']);
    });

    it('draws the bar of the home', async () => {
      await show({});

      expect(
        screen.getByRole('img', {
          name: "50 GB of your home's 212 GB: Archive 16 GB, old-app 13 GB, Renders 9 GB, datasets 7 GB, old-export 2 GB, 3 more 3 GB",
        }),
      ).toBeInTheDocument();
      expect(screen.getByText('50 GB', { selector: 'b' })).toBeInTheDocument();
    });

    it('shows more rows on Show more', async () => {
      const more = vi.fn();
      await show({ more });

      fireEvent.click(screen.getByRole('button', { name: 'Show more' }));

      expect(more).toHaveBeenCalledOnce();
    });

    it('lists the rows largest first, with no Show more when every row is in', async () => {
      await show({ page: foldersPageOf({ limit: 100, offset: 0 }, manyFolders(0)) });

      const list = screen.getByRole('list', { name: 'Folders left alone, largest first' });

      expect(
        within(list)
          .getAllByRole('listitem')
          .map((row) => within(row).getAllByText(/./u)[0]?.textContent),
      ).toStrictEqual(['Archive', 'old-app', 'Renders', 'datasets', 'old-export']);
      expect(screen.queryByRole('button', { name: 'Show more' })).not.toBeInTheDocument();
    });

    it('reveals a row by its id', async () => {
      const revealNode = vi.fn(() => null);
      await show({}, { revealNode });

      fireEvent.click(screen.getByRole('button', { name: 'Show in Finder: old-app' }));

      await waitFor(() => {
        expect(revealNode).toHaveBeenCalledOnce();
      });
      expect(revealNode.mock.calls[0]).toStrictEqual([{ node: { index: 202, state: 1 } }]);
    });
  });
});
