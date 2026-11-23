import { act, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { CoreAction } from './reduce';

import { mockCore, pushEvents, sampleAgreementAnswers, sampleStorageAnswers } from '../ipc/mock';
import { DEMO_DICTIONARY, IDLE, SCAN_STARTED } from '../test/demo-scan';
import { SAMPLE_STORAGE_MAP } from '../test/storage-map';
import { connectCore } from './connect';
import { CoreContext, createCoreStore, useCore } from './store';

const Status = (): string => useCore((state) => state.status.state);

describe('the core store', () => {
  describe('negative cases', () => {
    it('refuses to be read outside its context', () => {
      vi.spyOn(console, 'error').mockImplementation(() => {});

      expect(() => render(<Status />)).toThrow('useCore needs a CoreContext');
    });

    it('tells no one when an action changes nothing', () => {
      const store = createCoreStore();
      const listener = vi.fn();
      store.subscribe(listener);

      store.dispatch({ type: 'clear' });

      expect(listener).not.toHaveBeenCalled();
    });

    it('connects a store to the core only once', async () => {
      let subscriptions = 0;
      mockCore({ ...sampleAgreementAnswers, getDictionary: () => DEMO_DICTIONARY, getOverview: () => IDLE });
      const store = createCoreStore();
      const counting = store.subscribe(() => {
        subscriptions += 1;
      });

      connectCore(store);
      connectCore(store);
      await vi.waitFor(() => {
        expect(store.getState().dictionary).toBe(DEMO_DICTIONARY);
      });
      pushEvents(SCAN_STARTED);

      expect(store.getState().scan).toBe(1);
      expect(subscriptions).toBeGreaterThan(0);
      counting();
    });

    it('asks for the overview only once the events are subscribed', async () => {
      const subscribedAtOverview: boolean[] = [];
      let subscribed = false;
      mockCore({
        ...sampleAgreementAnswers,
        getDictionary: () => DEMO_DICTIONARY,
        getOverview: () => {
          subscribedAtOverview.push(subscribed);

          return IDLE;
        },
        subscribe: async () => {
          await Promise.resolve();
          subscribed = true;

          return null;
        },
      });

      connectCore(createCoreStore());

      await vi.waitFor(() => {
        expect(subscribedAtOverview).toEqual([true]);
      });
    });

    it('keeps a scan that starts while the overview is on its way', async () => {
      mockCore({
        ...sampleAgreementAnswers,
        getDictionary: () => DEMO_DICTIONARY,
        getOverview: () => {
          pushEvents(SCAN_STARTED);

          return IDLE;
        },
        ...sampleStorageAnswers,
      });
      const store = createCoreStore();
      const actions: CoreAction['type'][] = [];

      connectCore({
        ...store,
        dispatch: (action) => {
          actions.push(action.type);
          store.dispatch(action);
        },
      });
      await vi.waitFor(() => {
        expect(actions).toContain('overview-loaded');
      });

      expect(
        actions.filter(
          (type) => type !== 'agreement-loaded' && type !== 'dictionary-loaded' && type !== 'storage-map-loaded',
        ),
      ).toEqual(['scan-started', 'overview-loaded']);
      expect(store.getState().status).toEqual({ scan: 1, state: 'scanning' });
    });
  });

  describe('positive cases', () => {
    it('asks the storage map with the first overview, before anything is stale', async () => {
      mockCore({
        ...sampleAgreementAnswers,
        getDictionary: () => DEMO_DICTIONARY,
        getOverview: () => IDLE,
        ...sampleStorageAnswers,
      });
      const store = createCoreStore();

      connectCore(store);

      await vi.waitFor(() => {
        expect(store.getState().storageMap).toStrictEqual(SAMPLE_STORAGE_MAP);
      });
      expect(store.getState().failure).toBeNull();
    });

    it('renders again when the state changes', () => {
      const store = createCoreStore();
      render(
        <CoreContext value={store}>
          <Status />
        </CoreContext>,
      );
      expect(screen.getByText('idle')).toBeInTheDocument();

      act(() => {
        store.dispatch({ modules: [], scan: 1, type: 'scan-started' });
      });

      expect(screen.getByText('scanning')).toBeInTheDocument();
    });

    it('stops telling a listener that unsubscribed', () => {
      const store = createCoreStore();
      const listener = vi.fn();
      const unsubscribe = store.subscribe(listener);

      store.dispatch({ modules: [], scan: 1, type: 'scan-started' });
      unsubscribe();
      store.dispatch({ modules: [], scan: 2, type: 'scan-started' });

      expect(listener).toHaveBeenCalledOnce();
    });
  });
});
