import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { type JSX, useState } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { NodeId, NodeInfo, OwnPlace, OwnRow, Refusal } from '../../ipc/bindings.generated';

import { mockCore, refuse, sampleAgreementAnswers } from '../../ipc/mock';
import { type CoreStore, createCoreStore } from '../../state/store';
import { setMotion } from '../../test/animation';
import { DEMO_OVERVIEW } from '../../test/demo-scan';
import { LARGEST_DICTIONARY } from '../../test/largest-scan';
import { AppProviders } from '../app-providers';
import { BrowseView } from './browse-view';

const node = (index: number, name: string, kind: NodeInfo['kind'] = 'folder'): NodeInfo => ({
  files: 0,
  folders: 0,
  id: { index, state: 1 },
  kind,
  modified: null,
  name,
  notScanned: null,
  owner: null,
  sizes: { allocated: 5_000_000_000, completeness: 'complete', logical: 5_000_000_000, reclaimable: 0 },
});

const HOME = node(1, 'someone');
const MOVIES = node(2, 'Movies');
const TRIPS = node(3, 'Trips');

const row = (info: NodeInfo, opens: boolean): OwnRow => ({
  explained: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 },
  kind: info.kind === 'folder' ? 'folder' : 'file',
  node: info,
  offer: { kind: 'offered' },
  opens,
});

const PLACES: Readonly<Record<number, OwnPlace>> = {
  1: { offset: 0, path: [HOME], rows: [row(MOVIES, true), row(node(4, 'notes.pkg', 'collapsed'), false)], total: 2 },
  2: { offset: 0, path: [HOME, MOVIES], rows: [row(TRIPS, true)], total: 1 },
  3: { offset: 0, path: [HOME, MOVIES, TRIPS], rows: [], total: 0 },
};

const STALE: Refusal = { error: { code: 'stale-node' }, text: { args: {}, key: 'core.error.stale-node' } };

const asked: { folder: NodeId | null; offset: number }[] = [];

const answer = (args: unknown): OwnPlace => {
  const { folder, offset } = args as { readonly folder: NodeId | null; readonly offset: number };
  asked.push({ folder, offset });

  return PLACES[folder?.index ?? 1] as OwnPlace;
};

// The view with the screen's folder state around it, under a heading like the screen's.
const Host = (): JSX.Element => {
  const [folder, setFolder] = useState<NodeId | null>(null);

  return (
    <main>
      <h1>Largest</h1>
      <BrowseView folder={folder} onFolder={setFolder} />
    </main>
  );
};

const show = async (
  reply: (args: unknown) => unknown = answer,
  more: Readonly<Record<string, (args: unknown) => unknown>> = {},
): Promise<CoreStore> => {
  mockCore({
    ...sampleAgreementAnswers,
    ...more,
    getDictionary: () => LARGEST_DICTIONARY,
    getLargestPlace: reply,
    getOverview: () => DEMO_OVERVIEW,
  });
  const store = createCoreStore();
  render(
    <AppProviders store={store}>
      <Host />
    </AppProviders>,
  );
  await waitFor(() => {
    expect(store.getState().dictionary).not.toBeNull();
  });

  return store;
};

beforeEach(async () => {
  asked.length = 0;
  await setMotion('off');
});

afterEach(async () => {
  vi.restoreAllMocks();
  await setMotion(null);
});

describe('BrowseView', () => {
  describe('negative cases', () => {
    it('reports a refused reveal', async () => {
      const store = await show(answer, { revealNode: refuse(STALE) });

      fireEvent.click(await screen.findByRole('button', { name: 'Show in Finder: Movies' }));

      await waitFor(() => {
        expect(store.getState().failure?.kind).toBe('refused');
      });
    });

    it('has no checkbox', async () => {
      await show();
      await screen.findByText('Movies');

      expect(screen.queryByRole('checkbox')).not.toBeInTheDocument();
    });

    it('a folder that does not open has no Open button', async () => {
      await show();

      await screen.findByText('notes.pkg');

      // Only Movies opens: one Open button for the two rows.
      expect(screen.getAllByRole('button', { name: 'Open' })).toHaveLength(1);
    });

    it('the last step of the way is not a button', async () => {
      await show();
      fireEvent.click(await screen.findByRole('button', { name: 'Open' }));

      const way = await screen.findByRole('navigation', { name: 'Where you are' });

      expect(within(way).queryByRole('button', { name: 'Movies' })).not.toBeInTheDocument();
      expect(within(way).getByText('Movies')).toHaveAttribute('aria-current', 'page');
    });
  });

  describe('positive cases', () => {
    it('reveals a row by its id', async () => {
      const revealNode = vi.fn(() => null);
      await show(answer, { revealNode });

      fireEvent.click(await screen.findByRole('button', { name: 'Show in Finder: Movies' }));

      await waitFor(() => {
        expect(revealNode).toHaveBeenCalledOnce();
      });
      expect(revealNode.mock.calls[0]).toStrictEqual([{ node: MOVIES.id }]);
    });

    it('opens a folder', async () => {
      await show();

      fireEvent.click(await screen.findByRole('button', { name: 'Open' }));

      await screen.findByText('Trips');
      expect(asked.at(-1)).toStrictEqual({ folder: MOVIES.id, offset: 0 });
    });

    it('the home opens first', async () => {
      await show();

      await screen.findByText('Movies');

      expect(asked[0]).toStrictEqual({ folder: null, offset: 0 });
      expect(within(screen.getByRole('navigation', { name: 'Where you are' })).getByText('Home')).toBeInTheDocument();
    });

    it('opening a folder shows its rows and the way back', async () => {
      await show();
      fireEvent.click(await screen.findByRole('button', { name: 'Open' }));

      await screen.findByText('Trips');

      expect(asked[1]).toStrictEqual({ folder: MOVIES.id, offset: 0 });
      const way = screen.getByRole('navigation', { name: 'Where you are' });

      expect(within(way).getByRole('button', { name: 'Home' })).toBeInTheDocument();
    });

    it('a step of the way opens that folder', async () => {
      await show();
      fireEvent.click(await screen.findByRole('button', { name: 'Open' }));
      await screen.findByText('Trips');
      fireEvent.click(screen.getByRole('button', { name: 'Open' }));
      await screen.findByRole('navigation', { name: 'Where you are' });
      await waitFor(() => {
        expect(screen.getByText('Trips', { selector: '[aria-current]' })).toBeInTheDocument();
      });

      fireEvent.click(screen.getByRole('button', { name: 'Movies' }));

      await waitFor(() => {
        expect(screen.getByText('Movies', { selector: '[aria-current]' })).toBeInTheDocument();
      });
      expect(asked.at(-1)).toStrictEqual({ folder: MOVIES.id, offset: 0 });
    });

    it('a folder of an older scan falls back to the home', async () => {
      const stale = {
        error: { code: 'stale-node' },
        text: { args: {}, key: 'core.error.stale-node' },
      } as unknown as Refusal;
      await show((args) => {
        const { folder } = args as { readonly folder: NodeId | null };

        return folder?.index === 2 ? refuse(stale)() : answer(args);
      });
      fireEvent.click(await screen.findByRole('button', { name: 'Open' }));

      await waitFor(() => {
        expect(asked.at(-1)).toStrictEqual({ folder: null, offset: 0 });
      });
      expect(await screen.findByText('notes.pkg')).toBeInTheDocument();
    });

    it('asks the folder again when findings were removed', async () => {
      const store = await show();
      await screen.findByText('Movies');

      act(() => {
        store.dispatch({ findings: [1], scan: 1, type: 'findings-removed' });
      });

      await waitFor(() => {
        expect(asked).toHaveLength(2);
      });
    });

    it("the focus moves to the view's heading", async () => {
      await show();

      fireEvent.click(await screen.findByRole('button', { name: 'Open' }));

      await waitFor(() => {
        expect(screen.getByRole('heading', { level: 1 })).toHaveFocus();
      });
    });
  });
});
