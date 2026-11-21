import { act, renderHook, waitFor } from '@testing-library/react';
import { type ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';

import type { StorageMap } from '../ipc/bindings.generated';

import { mockCore, sampleStorageAnswers } from '../ipc/mock';
import { DEMO_MAP_SCANNING, SAMPLE_MEANING, SAMPLE_STORAGE_MAP } from '../test/storage-map';
import { useStorageMeaning } from './storage';
import { CoreContext, type CoreStore, createCoreStore } from './store';

const settle = async (): Promise<void> => {
  await new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
};

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

describe('the storage meaning', () => {
  describe('negative cases', () => {
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
      mockCore({ getStorageMeaning: () => calls('meaning') as never });
      const store = withMap(SAMPLE_STORAGE_MAP);

      renderHook(() => useStorageMeaning(null), { wrapper: wrapperOf(store) });
      await settle();

      expect(calls).not.toHaveBeenCalled();
    });
  });

  describe('positive cases', () => {
    it('the meaning view is asked again with a new state', async () => {
      const getMeaning = vi.fn(() => []);
      mockCore({ ...sampleStorageAnswers, getStorageMeaning: getMeaning });
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
      mockCore({ ...sampleStorageAnswers, getStorageMeaning: getMeaning });
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

    it('during a scan the meaning view follows the bar', async () => {
      const getMeaning = vi.fn(() => []);
      mockCore({ ...sampleStorageAnswers, getStorageMeaning: getMeaning });
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
