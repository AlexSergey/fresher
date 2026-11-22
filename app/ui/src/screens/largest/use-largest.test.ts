import { act, renderHook, waitFor } from '@testing-library/react';
import { createElement, type ReactNode } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { LargestPage, LargestQuery, Overview } from '../../ipc/bindings.generated';

import { mockCore, refuse } from '../../ipc/mock';
import { CoreContext, type CoreStore, createCoreStore } from '../../state/store';
import { BUSY, DEMO_OVERVIEW, IDLE } from '../../test/demo-scan';
import { largestPageOf, manyRows } from '../../test/largest-scan';
import { type Largest, NO_FILTER, useLargest } from './use-largest';

type Asked = (args: unknown) => LargestPage | Promise<LargestPage>;

const queryOf = (args: unknown): LargestQuery => (args as { readonly query: LargestQuery }).query;

// The list over 250 rows, paged as the core pages it.
const pagesOf: Asked = (args) => largestPageOf(queryOf(args), manyRows(250));

const scanned = (scan: number): Overview => ({
  ...DEMO_OVERVIEW,
  lastScan: { ...DEMO_OVERVIEW.lastScan, scan } as NonNullable<Overview['lastScan']>,
});

const render = (store: CoreStore): ReturnType<typeof renderHook<Largest, unknown>> =>
  renderHook(() => useLargest(NO_FILTER), {
    wrapper: ({ children }: { readonly children: ReactNode }) => createElement(CoreContext, { value: store }, children),
  });

// A store that knows a finished scan.
const storeOf = (overview: Overview = DEMO_OVERVIEW): CoreStore => {
  const store = createCoreStore();
  store.dispatch({ overview, type: 'overview-loaded' });

  return store;
};

const deferred = (): { readonly promise: Promise<LargestPage>; readonly resolve: (value: LargestPage) => void } => {
  let resolve: (value: LargestPage) => void = () => undefined;
  const promise = new Promise<LargestPage>((done) => {
    resolve = done;
  });

  return { promise, resolve };
};

afterEach(() => {
  vi.restoreAllMocks();
});

describe('useLargest', () => {
  describe('negative cases', () => {
    it('asks nothing before a scan', async () => {
      const asked = vi.fn(pagesOf);
      mockCore({ getLargest: asked });

      const { result } = render(storeOf(IDLE));
      await act(async () => {
        await Promise.resolve();
      });

      expect(asked).not.toHaveBeenCalled();
      expect(result.current.page).toBeNull();
    });

    it('shows no list while a scan runs and asks again when it ends', async () => {
      const asked = vi.fn(pagesOf);
      mockCore({ getLargest: asked });
      const store = storeOf();
      const { result } = render(store);
      await waitFor(() => {
        expect(result.current.page).not.toBeNull();
      });

      act(() => {
        store.dispatch({ modules: [], scan: 2, type: 'scan-started' });
      });

      expect(result.current.page).toBeNull();
      expect(asked).toHaveBeenCalledOnce();

      act(() => {
        store.dispatch({ overview: scanned(2), type: 'overview-loaded' });
        store.dispatch({ kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 2, type: 'scan-finished' });
      });
      await waitFor(() => {
        expect(result.current.page).not.toBeNull();
      });

      expect(asked).toHaveBeenCalledTimes(2);
    });

    it('says it failed and hands the refusal to the store when the core refuses', async () => {
      mockCore({ getLargest: refuse(BUSY) });
      const store = storeOf();

      const { result } = render(store);
      await waitFor(() => {
        expect(result.current.failed).toBe(true);
      });

      expect(result.current.page).toBeNull();
      expect(store.getState().failure?.kind).toBe('refused');
    });

    it('drops the late answer of an older list', async () => {
      const first = deferred();
      const second = deferred();
      const answers = [first.promise, second.promise];
      mockCore({ getLargest: () => answers.shift() });
      const store = storeOf();
      const { result } = render(store);

      act(() => {
        store.dispatch({ findings: [1], scan: 1, type: 'findings-removed' });
      });
      await act(async () => {
        second.resolve(
          largestPageOf({ kind: null, limit: 100, offset: 0, place: null, unchanged: 'any' }, manyRows(2)),
        );
        await second.promise;
      });
      await act(async () => {
        first.resolve(largestPageOf({ kind: null, limit: 100, offset: 0, place: null, unchanged: 'any' }, manyRows(9)));
        await first.promise;
      });

      expect(result.current.page?.total).toBe(2);
    });

    it('a double Show more lands the page once', async () => {
      const asked = vi.fn(pagesOf);
      mockCore({ getLargest: asked });
      const { result } = render(storeOf());
      await waitFor(() => {
        expect(result.current.page?.rows).toHaveLength(100);
      });

      act(() => {
        result.current.more();
        result.current.more();
      });
      await waitFor(() => {
        expect(result.current.page?.rows).toHaveLength(200);
      });

      expect(asked).toHaveBeenCalledTimes(2);
      expect(new Set(result.current.page?.rows.map(({ node }) => node.name)).size).toBe(200);
    });

    it('a failed Show more keeps the rows already in', async () => {
      const answers = [(args: unknown): LargestPage => largestPageOf(queryOf(args), manyRows(250)), refuse(BUSY)];
      mockCore({ getLargest: (args) => answers.shift()?.(args) });
      const store = storeOf();
      const { result } = render(store);
      await waitFor(() => {
        expect(result.current.page?.rows).toHaveLength(100);
      });

      act(() => {
        result.current.more();
      });
      await waitFor(() => {
        expect(store.getState().failure?.kind).toBe('refused');
      });

      expect(result.current.failed).toBe(false);
      expect(result.current.page?.rows).toHaveLength(100);
    });

    it('asks for no more when every row is in', async () => {
      const asked = vi.fn((args: unknown) => largestPageOf(queryOf(args), manyRows(3)));
      mockCore({ getLargest: asked });
      const { result } = render(storeOf());
      await waitFor(() => {
        expect(result.current.page).not.toBeNull();
      });

      act(() => {
        result.current.more();
      });

      expect(asked).toHaveBeenCalledOnce();
    });
  });

  describe('positive cases', () => {
    it('asks the first page of 100 with no filter', async () => {
      const asked = vi.fn(pagesOf);
      mockCore({ getLargest: asked });

      const { result } = render(storeOf());
      await waitFor(() => {
        expect(result.current.page?.rows).toHaveLength(100);
      });

      expect(queryOf(asked.mock.calls[0]?.[0])).toStrictEqual({
        kind: null,
        limit: 100,
        offset: 0,
        place: null,
        unchanged: 'any',
      });
      expect(result.current.page?.total).toBe(250);
    });

    it('asks the next offset and keeps the rows it has', async () => {
      const asked = vi.fn(pagesOf);
      mockCore({ getLargest: asked });
      const { result } = render(storeOf());
      await waitFor(() => {
        expect(result.current.page?.rows).toHaveLength(100);
      });

      act(() => {
        result.current.more();
      });
      await waitFor(() => {
        expect(result.current.page?.rows).toHaveLength(200);
      });

      expect(queryOf(asked.mock.calls[1]?.[0]).offset).toBe(100);
      expect(result.current.page?.rows[100]?.node.name).toBe('file-101.bin');
    });

    it('asks again from the start when a cleanup removed findings', async () => {
      const asked = vi.fn(pagesOf);
      mockCore({ getLargest: asked });
      const store = storeOf();
      const { result } = render(store);
      await waitFor(() => {
        expect(result.current.page?.rows).toHaveLength(100);
      });
      act(() => {
        result.current.more();
      });
      await waitFor(() => {
        expect(result.current.page?.rows).toHaveLength(200);
      });

      act(() => {
        store.dispatch({ findings: [1], scan: 1, type: 'findings-removed' });
      });
      await waitFor(() => {
        expect(asked).toHaveBeenCalledTimes(3);
      });

      expect(queryOf(asked.mock.calls[2]?.[0]).offset).toBe(0);
      await waitFor(() => {
        expect(result.current.page?.rows).toHaveLength(100);
      });
    });
  });
});
