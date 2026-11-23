import type { CoreStore } from './store';

import { setCoreDictionary } from '../i18n';
import {
  failureOf,
  getAccessStatus,
  getAgreement,
  getDictionary,
  getOverview,
  getSettings,
  getStorageMap,
  startScan,
  subscribe,
} from '../ipc';
import { systemClock } from './clock';
import { throttleProgress } from './progress';
import { createOverviewRefresher } from './refresh';

// The overview at most 4 times a second during a scan; the progress at most 10 times a second.
const OVERVIEW_INTERVAL_MS = 250;
const PROGRESS_INTERVAL_MS = 100;

const connected = new WeakSet<CoreStore>();

const failed =
  (store: CoreStore) =>
  (error: unknown): void => {
    store.dispatch({ failure: failureOf(error), type: 'command-failed' });
  };

// Connects the store to the core once: the events, the dictionary, the state the engine is in, and the refresher
// that keeps the overview's totals current while findings arrive.
export const connectCore = (store: CoreStore): void => {
  if (connected.has(store)) {
    return;
  }
  connected.add(store);

  getDictionary('en')
    .then((dictionary) => {
      setCoreDictionary(dictionary);
      store.dispatch({ dictionary, type: 'dictionary-loaded' });
    })
    .catch(failed(store));
  // A failed load is not the frame's error: the gate asks again when a cleanup needs the agreement.
  getAgreement()
    .then((view) => {
      store.dispatch({ type: 'agreement-loaded', view });
    })
    .catch(() => undefined);
  createOverviewRefresher(store, { clock: systemClock, interval: OVERVIEW_INTERVAL_MS });
  // The overview and the storage map are asked for once the events flow, so no change of the engine falls between.
  const events = subscribe(
    throttleProgress(store.dispatch, { clock: systemClock, interval: PROGRESS_INTERVAL_MS }),
  ).catch(failed(store));
  events
    .then(getOverview)
    .then((overview) => {
      store.dispatch({ overview, type: 'overview-loaded' });
    })
    .catch(failed(store));
  events
    .then(getStorageMap)
    .then((storageMap) => {
      store.dispatch({ storageMap, type: 'storage-map-loaded' });
    })
    .catch(failed(store));
};

// Asks the core for a scan; its findings arrive as events, a failure is kept for the screen.
export const requestScan = (store: CoreStore): void => {
  startScan().catch(failed(store));
};

// What is being asked for now, per store: a screen that asks again while the answer is on its way asks for nothing.
const askingSettings = new WeakSet<CoreStore>();
const askingAccess = new WeakSet<CoreStore>();

const askOnce = <T>(
  asking: WeakSet<CoreStore>,
  store: CoreStore,
  ask: () => Promise<T>,
  loaded: (answer: T) => void,
): void => {
  if (asking.has(store)) {
    return;
  }
  asking.add(store);

  ask().then(loaded, (error: unknown) => {
    // A failed question may be asked again when a screen opens next.
    asking.delete(store);
    failed(store)(error);
  });
};

// Asks the core for the settings when the store has none; `setting-changed` events keep them current.
export const requestSettings = (store: CoreStore): void => {
  if (store.getState().settings === null) {
    askOnce(askingSettings, store, getSettings, (settings) => {
      store.dispatch({ settings, type: 'settings-loaded' });
    });
  }
};

// Asks the core for the permissions when the store has none; `access-changed` events keep them current.
export const requestAccess = (store: CoreStore): void => {
  if (store.getState().access === null) {
    askOnce(askingAccess, store, getAccessStatus, (status) => {
      store.dispatch({ status, type: 'access-loaded' });
    });
  }
};
