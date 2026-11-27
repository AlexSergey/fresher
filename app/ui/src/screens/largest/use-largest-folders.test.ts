import { act, renderHook, waitFor } from '@testing-library/react';
import { createElement, type ReactNode } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { FoldersPage, FoldersQuery, Overview } from '../../ipc/bindings.generated';

import { mockCore, refuse } from '../../ipc/mock';
import { CoreContext, type CoreStore, createCoreStore } from '../../state/store';
import { BUSY, DEMO_OVERVIEW, IDLE } from '../../test/demo-scan';
import { foldersPageOf, manyFolders } from '../../test/largest-scan';
import { type LargestFolders, useLargestFolders } from './use-largest-folders';

const queryOf = (args: unknown): FoldersQuery => (args as { readonly query: FoldersQuery }).query;

// The list over 150 folders, paged as the core pages it.
const pagesOf = (args: unknown): FoldersPage => foldersPageOf(queryOf(args), manyFolders(145));

const render = (store: CoreStore): ReturnType<typeof renderHook<LargestFolders, unknown>> =>
  renderHook(() => useLargestFolders(), {
    wrapper: ({ children }: { readonly children: ReactNode }) => createElement(CoreContext, { value: store }, children),
  });

// A store that knows a finished scan.
const storeOf = (overview: Overview = DEMO_OVERVIEW): CoreStore => {
  const store = createCoreStore();
  store.dispatch({ overview, type: 'overview-loaded' });

  return store;
};

// The list is shown: the first page answered.
const shown = async (result: { readonly current: LargestFolders }): Promise<void> => {
  await waitFor(() => {
    expect(result.current.page?.rows).toHaveLength(100);
  });
};

afterEach(() => {
  vi.restoreAllMocks();
});

describe('useLargestFolders', () => {
  describe('negative cases', () => {
    it('asks nothing before a scan', async () => {
      const asked = vi.fn(pagesOf);
      mockCore({ getLargestFolders: asked });

      const { result } = render(storeOf(IDLE));
      await act(async () => {
        await Promise.resolve();
      });

      expect(asked).not.toHaveBeenCalled();
      expect(result.current.page).toBeNull();
    });

    it('asks nothing while a scan runs', async () => {
      const asked = vi.fn(pagesOf);
      mockCore({ getLargestFolders: asked });
      const store = storeOf();
      const { result } = render(store);
      await shown(result);

      act(() => {
        store.dispatch({ modules: [], scan: 2, type: 'scan-started' });
      });
      await act(async () => {
        await Promise.resolve();
      });

      expect(result.current.page).toBeNull();
      expect(asked).toHaveBeenCalledOnce();
    });

    it('reports a refusal and keeps no page', async () => {
      mockCore({ getLargestFolders: refuse(BUSY) });
      const store = storeOf();

      const { result } = render(store);
      await waitFor(() => {
        expect(result.current.failed).toBe(true);
      });

      expect(result.current.page).toBeNull();
      expect(store.getState().failure?.kind).toBe('refused');
    });
  });

  describe('positive cases', () => {
    it('asks the first page when a scan ends', async () => {
      const asked = vi.fn(pagesOf);
      mockCore({ getLargestFolders: asked });

      const { result } = render(storeOf());
      await shown(result);

      expect(queryOf(asked.mock.calls[0]?.[0])).toStrictEqual({ limit: 100, offset: 0 });
      expect(result.current.page?.total).toBe(150);
    });

    it('adds the next page on more', async () => {
      const asked = vi.fn(pagesOf);
      mockCore({ getLargestFolders: asked });
      const { result } = render(storeOf());
      await shown(result);

      act(() => {
        result.current.more();
        result.current.more();
      });
      await waitFor(() => {
        expect(result.current.page?.rows).toHaveLength(150);
      });

      expect(asked).toHaveBeenCalledTimes(2);
      expect(queryOf(asked.mock.calls[1]?.[0])).toStrictEqual({ limit: 100, offset: 100 });
      expect(result.current.page?.rows[100]?.node.name).toBe('folder-96');
    });

    it('asks again after a cleanup removed something', async () => {
      const asked = vi.fn(pagesOf);
      mockCore({ getLargestFolders: asked });
      const store = storeOf();
      const { result } = render(store);
      await shown(result);

      act(() => {
        store.dispatch({ findings: [1], scan: 1, type: 'findings-removed' });
      });

      await waitFor(() => {
        expect(asked).toHaveBeenCalledTimes(2);
      });
      expect(queryOf(asked.mock.calls[1]?.[0]).offset).toBe(0);
    });

    it('asks again after an ignore list changed', async () => {
      const asked = vi.fn(pagesOf);
      mockCore({ getLargestFolders: asked });
      const store = storeOf();
      const { result } = render(store);
      await shown(result);

      act(() => {
        store.dispatch({ key: 'core.ignore.items', type: 'setting-changed', value: { kind: 'ids', value: ['a'] } });
      });

      await waitFor(() => {
        expect(asked).toHaveBeenCalledTimes(2);
      });
      expect(queryOf(asked.mock.calls[1]?.[0]).offset).toBe(0);
    });
  });
});
