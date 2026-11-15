import type { CoreStore } from './store';

import { setCoreDictionary } from '../i18n';
import { failureOf, getDictionary, getOverview, startScan, subscribe } from '../ipc';

const connected = new WeakSet<CoreStore>();

const failed =
  (store: CoreStore) =>
  (error: unknown): void => {
    store.dispatch({ failure: failureOf(error), type: 'command-failed' });
  };

// Connects the store to the core once: the events, the dictionary, and the state the engine is in.
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
  // The overview is asked for once the events flow, so no change of the engine falls between the two.
  subscribe(store.dispatch)
    .catch(failed(store))
    .then(getOverview)
    .then((overview) => {
      store.dispatch({ overview, type: 'overview-loaded' });
    })
    .catch(failed(store));
};

// Asks the core for a scan; its findings arrive as events, a failure is kept for the screen.
export const requestScan = (store: CoreStore): void => {
  startScan().catch(failed(store));
};
