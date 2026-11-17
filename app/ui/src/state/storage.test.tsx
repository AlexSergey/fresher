import { act, renderHook, waitFor } from '@testing-library/react';
import { type ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';

import type { Descent, NodeId, PlaceView, StorageMap } from '../ipc/bindings.generated';

import { CommandFailed } from '../ipc';
import { mockCore, refuse, sampleStorageAnswers } from '../ipc/mock';
import { BUSY } from '../test/demo-scan';
import { DEMO_MAP_SCANNING, EMPTY_PLACE, SAMPLE_MEANING, SAMPLE_STORAGE_MAP } from '../test/storage-map';
import { createPlaceSource, useStorageMeaning, useStoragePlace, useStorageRoots } from './storage';
import { CoreContext, type CoreStore, createCoreStore } from './store';

const ROOT: NodeId = { index: 0, state: 1 };
const OLD: NodeId = { index: 7, state: 0 };

const STALE = {
  error: { code: 'stale-node' },
  text: { args: {}, key: 'core.error.stale-node' },
} as const;

const viewOf = (offset: number, rows: number, total: number): PlaceView => ({
  offset,
  path: [],
  rows: Array.from({ length: rows }, (_, at) => ({
    explained: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 },
    node: {
      id: { index: offset + at, state: 1 },
      kind: 'file',
      name: `row ${offset + at}`,
      sizes: { allocated: 1, completeness: 'complete', logical: 1, reclaimable: 0 },
    },
  })) as unknown as PlaceView['rows'],
  total,
});

const settle = async (): Promise<void> => {
  await new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
};

const sourceOf = (
  getPlace: (node: NodeId, descent: Descent, offset: number, limit: number) => Promise<PlaceView>,
  maxPages = 10,
): ReturnType<typeof createPlaceSource> =>
  createPlaceSource(getPlace, { maxPages, pageSize: 100, phase: 'a', root: () => ROOT });

const wrapperOf =
  (store: CoreStore) =>
  ({ children }: { readonly children: ReactNode }): ReactNode => <CoreContext value={store}>{children}</CoreContext>;

const withMap = (map: StorageMap): CoreStore => {
  const store = createCoreStore();
  store.dispatch({ storageMap: map, type: 'storage-map-loaded' });

  return store;
};

// The map with the first volume's first part (its Explained) at `gigabytes`, the phase as it was: an ignore, a restore
// or a scan's progress.
const withParts = (map: StorageMap, gigabytes: number): StorageMap => ({
  ...map,
  volumes: map.volumes.map((volume, at) =>
    at === 0
      ? {
          ...volume,
          parts: volume.parts.map((part, index) =>
            index === 0 ? { ...part, sizes: { ...part.sizes, allocated: gigabytes * 1_000_000_000 } } : part,
          ),
        }
      : volume,
  ),
});

describe('the storage views', () => {
  describe('negative cases', () => {
    it('a stale place reopens the root', async () => {
      const asked: NodeId[] = [];
      const source = sourceOf((node) => {
        asked.push(node);

        return node.state === OLD.state ? Promise.reject(new CommandFailed(STALE)) : Promise.resolve(viewOf(0, 3, 3));
      });

      source.show(OLD);
      await settle();

      expect(asked).toEqual([OLD, ROOT]);
      expect(source.node()).toEqual(ROOT);
      expect(source.total()).toBe(3);
      expect(source.failure()).toBeNull();
    });

    it('a stale place reopens the root of its own volume', async () => {
      const second: NodeId = { index: 5, state: 1 };
      const [first] = SAMPLE_STORAGE_MAP.volumes;
      const map = {
        ...SAMPLE_STORAGE_MAP,
        volumes: [first, { ...first, name: 'Data', roots: [second] }],
      } as StorageMap;
      const asked: NodeId[] = [];
      mockCore({
        ...sampleStorageAnswers,
        getStoragePlace: (args) => {
          const { node } = args as { readonly node: NodeId };
          asked.push(node);

          return node.state === OLD.state ? refuse(STALE)() : viewOf(0, 1, 1);
        },
      });
      const { result } = renderHook(() => useStoragePlace(OLD, 1), { wrapper: wrapperOf(withMap(map)) });

      await waitFor(() => {
        expect(result.current.node).toEqual(second);
      });
      expect(asked).toEqual([OLD, second]);
    });

    it('an older meaning answer does not replace a newer one', async () => {
      const waiting: ((groups: readonly never[]) => void)[] = [];
      mockCore({
        ...sampleStorageAnswers,
        getStorageMeaning: () =>
          new Promise((resolve) => {
            waiting.push(resolve);
          }),
      });
      const store = withMap(SAMPLE_STORAGE_MAP);
      const { result } = renderHook(() => useStorageMeaning(0), { wrapper: wrapperOf(store) });
      await waitFor(() => {
        expect(waiting).toHaveLength(1);
      });
      act(() => {
        store.dispatch({
          storageMap: { ...SAMPLE_STORAGE_MAP, phase: { kind: 'complete', state: 2 } },
          type: 'storage-map-loaded',
        });
      });
      await waitFor(() => {
        expect(waiting).toHaveLength(2);
      });

      await act(async () => {
        waiting[1]?.([]);
        await settle();
        waiting[0]?.(SAMPLE_MEANING as never);
        await settle();
      });

      expect(result.current.groups).toEqual([]);
    });

    it('no view is asked before it is shown', async () => {
      const calls = vi.fn();
      mockCore({
        getStorageMeaning: () => calls('meaning') as never,
        getStoragePlace: () => calls('place') as never,
      });
      const store = withMap(SAMPLE_STORAGE_MAP);

      renderHook(() => useStoragePlace(null, 0), { wrapper: wrapperOf(store) });
      renderHook(() => useStorageMeaning(null), { wrapper: wrapperOf(store) });
      await settle();

      expect(calls).not.toHaveBeenCalled();
    });

    it('a volume with one root has no list of roots', () => {
      mockCore(sampleStorageAnswers);
      const { result } = renderHook(() => useStorageRoots(0), { wrapper: wrapperOf(withMap(SAMPLE_STORAGE_MAP)) });

      expect(result.current).toBeNull();
    });

    it('a busy place is a failure, not rows', async () => {
      const source = sourceOf(() => Promise.reject(new CommandFailed(BUSY)));

      source.show(ROOT);
      await settle();

      expect(source.failure()).toEqual({ kind: 'refused', refusal: BUSY });
      expect(source.total()).toBeUndefined();
    });

    it('an eleventh page drops the least recently used', async () => {
      const source = sourceOf((_node, _descent, offset) => Promise.resolve(viewOf(offset, 100, 2000)));
      source.show(ROOT);
      await settle();

      for (let page = 1; page <= 10; page += 1) {
        source.ensure(page * 100, page * 100);
        await settle();
      }

      expect(source.held()).toBe(1000);
      expect(source.rowAt(0)).toBeUndefined();
      expect(source.rowAt(100)).toBeDefined();
      expect(source.rowAt(1000)).toBeDefined();
    });
  });

  describe('positive cases', () => {
    it('a next page is of the folder the descent settled on', async () => {
      const settled: NodeId = { index: 3, state: 1 };
      const asked: [NodeId, Descent, number][] = [];
      const source = sourceOf((node, descent, offset) => {
        asked.push([node, descent, offset]);
        const view = viewOf(offset, 100, 300);

        return Promise.resolve(offset === 0 ? { ...view, path: [{ id: ROOT }, { id: settled }] as never } : view);
      });

      source.show(ROOT);
      await settle();
      source.ensure(150, 160);
      await settle();

      expect(asked).toEqual([
        [ROOT, 'through', 0],
        [settled, 'stay', 100],
      ]);
    });

    it('a breadcrumb step asks the folder as it is', async () => {
      const asked: Descent[] = [];
      const source = sourceOf((_node, descent) => {
        asked.push(descent);

        return Promise.resolve(viewOf(0, 1, 1));
      });

      source.show(ROOT);
      await settle();
      source.show(ROOT, 'stay');
      await settle();

      expect(asked).toEqual(['through', 'stay']);
    });

    it('the roots of a volume that holds several are each described', async () => {
      const second: NodeId = { index: 5, state: 1 };
      const [first] = SAMPLE_STORAGE_MAP.volumes;
      const map = { ...SAMPLE_STORAGE_MAP, volumes: [{ ...first, roots: [ROOT, second] }] } as StorageMap;
      mockCore({
        ...sampleStorageAnswers,
        getStoragePlace: (args) => {
          const { node } = args as { readonly node: NodeId };

          return { ...EMPTY_PLACE, path: [{ id: node, name: `root ${String(node.index)}` }] };
        },
      });
      const { result } = renderHook(() => useStorageRoots(0), { wrapper: wrapperOf(withMap(map)) });

      expect(result.current).toBeUndefined();
      await waitFor(() => {
        expect(result.current?.map(({ name }) => name)).toEqual(['root 0', 'root 5']);
      });
    });

    it('a new state asks the view again', async () => {
      const getPlace = vi.fn(() => Promise.resolve(viewOf(0, 2, 2)));
      mockCore({
        ...sampleStorageAnswers,
        getStoragePlace: getPlace,
      });
      const store = withMap(SAMPLE_STORAGE_MAP);
      renderHook(() => useStoragePlace(ROOT, 0), { wrapper: wrapperOf(store) });
      await waitFor(() => {
        expect(getPlace).toHaveBeenCalledTimes(1);
      });

      act(() => {
        store.dispatch({
          storageMap: { ...SAMPLE_STORAGE_MAP, phase: { kind: 'complete', state: 2 } },
          type: 'storage-map-loaded',
        });
      });

      await waitFor(() => {
        expect(getPlace).toHaveBeenCalledTimes(2);
      });
    });

    it('the meaning view is asked again with a new state', async () => {
      const getMeaning = vi.fn(() => []);
      mockCore({ ...sampleStorageAnswers, getStorageMeaning: getMeaning, getStoragePlace: () => EMPTY_PLACE });
      const store = withMap(SAMPLE_STORAGE_MAP);
      const { result } = renderHook(() => useStorageMeaning(0), { wrapper: wrapperOf(store) });
      await waitFor(() => {
        expect(result.current.groups).toEqual([]);
      });

      act(() => {
        store.dispatch({
          storageMap: { ...SAMPLE_STORAGE_MAP, phase: { kind: 'complete', state: 2 } },
          type: 'storage-map-loaded',
        });
      });

      await waitFor(() => {
        expect(getMeaning).toHaveBeenCalledTimes(2);
      });
    });

    it('the meaning view is asked again after an ignore changes the parts in the same state', async () => {
      const getMeaning = vi.fn(() => []);
      mockCore({ ...sampleStorageAnswers, getStorageMeaning: getMeaning, getStoragePlace: () => EMPTY_PLACE });
      const store = withMap(SAMPLE_STORAGE_MAP);
      renderHook(() => useStorageMeaning(0), { wrapper: wrapperOf(store) });
      await waitFor(() => {
        expect(getMeaning).toHaveBeenCalledTimes(1);
      });

      act(() => {
        store.dispatch({ storageMap: withParts(SAMPLE_STORAGE_MAP, 60), type: 'storage-map-loaded' });
      });

      await waitFor(() => {
        expect(getMeaning).toHaveBeenCalledTimes(2);
      });
    });

    it('the place view is asked again after an ignore changes the parts in the same state', async () => {
      const getPlace = vi.fn(() => Promise.resolve(viewOf(0, 2, 2)));
      mockCore({ ...sampleStorageAnswers, getStoragePlace: getPlace });
      const store = withMap(SAMPLE_STORAGE_MAP);
      renderHook(() => useStoragePlace(ROOT, 0), { wrapper: wrapperOf(store) });
      await waitFor(() => {
        expect(getPlace).toHaveBeenCalledTimes(1);
      });

      act(() => {
        store.dispatch({ storageMap: withParts(SAMPLE_STORAGE_MAP, 60), type: 'storage-map-loaded' });
      });

      await waitFor(() => {
        expect(getPlace).toHaveBeenCalledTimes(2);
      });
    });

    it('during a scan the meaning view follows the bar', async () => {
      const getMeaning = vi.fn(() => []);
      mockCore({ ...sampleStorageAnswers, getStorageMeaning: getMeaning, getStoragePlace: () => EMPTY_PLACE });
      const store = withMap(DEMO_MAP_SCANNING);
      renderHook(() => useStorageMeaning(0), { wrapper: wrapperOf(store) });
      await waitFor(() => {
        expect(getMeaning).toHaveBeenCalledTimes(1);
      });

      act(() => {
        store.dispatch({ storageMap: withParts(DEMO_MAP_SCANNING, 3), type: 'storage-map-loaded' });
      });

      await waitFor(() => {
        expect(getMeaning).toHaveBeenCalledTimes(2);
      });
    });
  });
});
