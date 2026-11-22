import { act, renderHook, waitFor } from '@testing-library/react';
import { createElement, type ReactNode } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { TrashLine, TrashView } from '../../ipc/bindings.generated';

import { mockCore, refuse } from '../../ipc/mock';
import { CoreContext, type CoreStore, createCoreStore } from '../../state/store';
import { cleanupEventsOf, DEMO_PLAN, MOVED_REPORT } from '../../test/cleanup-plan';
import { DEMO_OVERVIEW } from '../../test/demo-scan';
import { openTrashLine, useTrash } from './use-trash';

const sizes = { allocated: 100, completeness: 'complete', logical: 100, reclaimable: 100 } as const;

const line = (index: number): TrashLine => ({ fromCleanups: sizes, goneBy: null, index, volume: null, waiting: sizes });

const view = (...indexes: readonly number[]): TrashView => ({ trashes: indexes.map(line) });

const render = (store: CoreStore): ReturnType<typeof renderHook<readonly TrashLine[], unknown>> =>
  renderHook(() => useTrash(), {
    wrapper: ({ children }: { readonly children: ReactNode }) => createElement(CoreContext, { value: store }, children),
  });

const deferred = (): { readonly promise: Promise<TrashView>; readonly resolve: (value: TrashView) => void } => {
  let resolve: (value: TrashView) => void = () => undefined;
  const promise = new Promise<TrashView>((done) => {
    resolve = done;
  });

  return { promise, resolve };
};

afterEach(() => {
  vi.restoreAllMocks();
});

describe('useTrash', () => {
  describe('negative cases', () => {
    it('gives no line and dispatches nothing when the answer fails', async () => {
      const asked = vi.fn(refuse({ code: 'unknown-trash' } as never));
      mockCore({ getTrash: asked });
      const store = createCoreStore();
      const dispatch = vi.spyOn(store, 'dispatch');

      const { result } = render(store);
      await waitFor(() => {
        expect(asked).toHaveBeenCalledOnce();
      });
      await act(async () => {
        await Promise.resolve();
      });

      expect(result.current).toEqual([]);
      expect(dispatch).not.toHaveBeenCalled();
    });

    it('drops the late answer of an older request', async () => {
      const first = deferred();
      const second = deferred();
      const answers = [first.promise, second.promise];
      mockCore({ getTrash: () => answers.shift() });
      const { result } = render(createCoreStore());

      act(() => {
        window.dispatchEvent(new Event('focus'));
      });
      await act(async () => {
        second.resolve(view(2));
        await second.promise;
      });
      await act(async () => {
        first.resolve(view(1));
        await first.promise;
      });

      expect(result.current.map(({ index }) => index)).toEqual([2]);
    });

    it('stops listening to the focus when it unmounts', async () => {
      const asked = vi.fn(() => view());
      mockCore({ getTrash: asked });
      const { unmount } = render(createCoreStore());
      await waitFor(() => {
        expect(asked).toHaveBeenCalledOnce();
      });

      unmount();
      window.dispatchEvent(new Event('focus'));

      expect(asked).toHaveBeenCalledOnce();
    });

    it('a refused Trash that cannot be opened goes to the frame', async () => {
      mockCore({ openTrash: refuse({ code: 'unknown-trash' } as never) });
      const store = createCoreStore();
      const dispatch = vi.spyOn(store, 'dispatch');

      openTrashLine(store)(3);

      await waitFor(() => {
        expect(dispatch).toHaveBeenCalledWith(expect.objectContaining({ type: 'command-failed' }));
      });
    });
  });

  describe('positive cases', () => {
    it('asks when it mounts and gives the lines', async () => {
      mockCore({ getTrash: () => view(0, 1) });

      const { result } = render(createCoreStore());

      await waitFor(() => {
        expect(result.current).toHaveLength(2);
      });
    });

    it('asks again when the window gets focus', async () => {
      const asked = vi.fn(() => view(0));
      mockCore({ getTrash: asked });
      render(createCoreStore());
      await waitFor(() => {
        expect(asked).toHaveBeenCalledOnce();
      });

      act(() => {
        window.dispatchEvent(new Event('focus'));
      });

      await waitFor(() => {
        expect(asked).toHaveBeenCalledTimes(2);
      });
    });

    it('asks again after a new scan', async () => {
      const asked = vi.fn(() => view(0));
      mockCore({ getTrash: asked });
      const store = createCoreStore();
      render(store);
      await waitFor(() => {
        expect(asked).toHaveBeenCalledTimes(1);
      });

      act(() => {
        store.dispatch({ overview: DEMO_OVERVIEW, type: 'overview-loaded' });
      });

      await waitFor(() => {
        expect(asked).toHaveBeenCalledTimes(2);
      });
    });

    it('asks again after a cleanup finished', async () => {
      const asked = vi.fn(() => view(0));
      mockCore({ getTrash: asked });
      const store = createCoreStore();
      render(store);
      await waitFor(() => {
        expect(asked).toHaveBeenCalledTimes(1);
      });
      const { finished, started } = cleanupEventsOf(DEMO_PLAN, MOVED_REPORT);

      act(() => {
        store.dispatch(started);
        store.dispatch(finished);
      });

      await waitFor(() => {
        expect(asked).toHaveBeenCalledTimes(2);
      });
    });

    it('opens the Trash of the index', async () => {
      const opened = vi.fn(() => null);
      mockCore({ openTrash: opened });

      openTrashLine(createCoreStore())(1);

      await waitFor(() => {
        expect(opened).toHaveBeenCalledWith({ index: 1 });
      });
    });
  });
});
