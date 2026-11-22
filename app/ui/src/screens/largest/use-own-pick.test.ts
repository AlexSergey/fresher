import { act, renderHook, waitFor } from '@testing-library/react';
import { createElement, type ReactNode } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { mockCore, refuse } from '../../ipc/mock';
import { CoreContext, type CoreStore, createCoreStore } from '../../state/store';
import { DEMO_OVERVIEW } from '../../test/demo-scan';
import { LARGEST_ROWS, ownFindingOf } from '../../test/largest-scan';
import { useOwnPick } from './use-own-pick';

const ROW = LARGEST_ROWS[0] as (typeof LARGEST_ROWS)[number];

const render = (store: CoreStore): ReturnType<typeof renderHook<ReturnType<typeof useOwnPick>, unknown>> =>
  renderHook(() => useOwnPick(), {
    wrapper: ({ children }: { readonly children: ReactNode }) => createElement(CoreContext, { value: store }, children),
  });

const storeOf = (): CoreStore => {
  const store = createCoreStore();
  store.dispatch({ overview: DEMO_OVERVIEW, type: 'overview-loaded' });

  return store;
};

afterEach(() => {
  vi.restoreAllMocks();
});

describe('useOwnPick', () => {
  describe('negative cases', () => {
    it('a refused pick leaves the row unticked', async () => {
      mockCore({
        pickOwnItem: refuse({
          error: { code: 'not-own-item' },
          text: { args: {}, key: 'core.error.not-own-item' },
        } as never),
      });
      const store = storeOf();
      const { result } = render(store);

      act(() => {
        result.current.toggle(ROW);
      });

      await waitFor(() => {
        expect(store.getState().failure).not.toBeNull();
      });
      expect(result.current.ticked(ROW)).toBe(false);
      expect(store.getState().selection.findings.size).toBe(0);
    });

    it('a second click while the core picks asks once', () => {
      const pickOwnItem = vi.fn(() => new Promise<never>(() => undefined));
      mockCore({ pickOwnItem });
      const { result } = render(storeOf());

      act(() => {
        result.current.toggle(ROW);
        result.current.toggle(ROW);
      });

      expect(pickOwnItem).toHaveBeenCalledTimes(1);
    });

    it('a pick that answers after a new scan began is dropped', async () => {
      let answer: (value: ReturnType<typeof ownFindingOf>) => void = () => undefined;
      mockCore({
        pickOwnItem: () =>
          new Promise((resolve) => {
            answer = resolve;
          }),
      });
      const store = storeOf();
      const { result } = render(store);
      act(() => {
        result.current.toggle(ROW);
      });

      act(() => {
        store.dispatch({ modules: [], scan: 2, type: 'scan-started' } as never);
      });
      await act(async () => {
        answer(ownFindingOf(ROW, 7));
        await Promise.resolve();
      });

      expect(store.getState().selection.findings.size).toBe(0);
    });

    it('a pick that answers after the selection was cleared is dropped', async () => {
      let answer: (value: ReturnType<typeof ownFindingOf>) => void = () => undefined;
      mockCore({
        pickOwnItem: () =>
          new Promise((resolve) => {
            answer = resolve;
          }),
      });
      const store = storeOf();
      store.dispatch({ finding: ownFindingOf(ROW, 8), type: 'toggle-finding' });
      const { result } = render(store);
      act(() => {
        result.current.toggle(ROW);
      });

      act(() => {
        store.dispatch({ type: 'clear' });
      });
      await act(async () => {
        answer(ownFindingOf(ROW, 7));
        await Promise.resolve();
      });

      expect(store.getState().selection.findings.size).toBe(0);
    });
  });

  describe('positive cases', () => {
    it('picks once, then unticks and ticks again with the kept finding', async () => {
      const pickOwnItem = vi.fn(() => ownFindingOf(ROW, 7));
      mockCore({ pickOwnItem });
      const store = storeOf();
      const { result } = render(store);

      act(() => {
        result.current.toggle(ROW);
      });
      await waitFor(() => {
        expect(result.current.ticked(ROW)).toBe(true);
      });
      act(() => {
        result.current.toggle(ROW);
      });

      expect(result.current.ticked(ROW)).toBe(false);
      expect(pickOwnItem).toHaveBeenCalledTimes(1);
    });

    it('a row the core says is picked is ticked when its finding is in the selection', () => {
      mockCore({});
      const store = storeOf();
      store.dispatch({ finding: ownFindingOf(ROW, 7), type: 'toggle-finding' });
      const { result } = render(store);

      expect(result.current.ticked({ ...ROW, picked: 7 })).toBe(true);
      expect(result.current.ticked({ ...ROW, picked: 8 })).toBe(false);
    });
  });
});
