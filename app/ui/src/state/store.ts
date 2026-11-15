import { createContext, use, useSyncExternalStore } from 'react';

import { type CoreAction, type CoreState, INITIAL_STATE, reduce } from './reduce';

// The UI's one store of what the core sent; components read it with `useCore`.
export type CoreStore = {
  readonly dispatch: (action: CoreAction) => void;
  readonly getState: () => CoreState;
  readonly subscribe: (listener: () => void) => () => void;
};

export const createCoreStore = (): CoreStore => {
  let state = INITIAL_STATE;
  const listeners = new Set<() => void>();

  return {
    dispatch: (action): void => {
      const next = reduce(state, action);

      if (next !== state) {
        state = next;
        for (const listener of listeners) {
          listener();
        }
      }
    },
    getState: (): CoreState => state,
    subscribe: (listener): (() => void) => {
      listeners.add(listener);

      return (): void => {
        listeners.delete(listener);
      };
    },
  };
};

export const CoreContext = createContext<CoreStore | null>(null);

export const useCoreStore = (): CoreStore => {
  const store = use(CoreContext);

  if (store === null) {
    throw new Error('useCore needs a CoreContext around it');
  }

  return store;
};

// A part of the store's state; the component renders again when it changes. The selector must return a part of the
// state as it is (not a new object), so that an unchanged state gives the same value.
export const useCore = <T>(selector: (state: CoreState) => T): T => {
  const store = useCoreStore();

  return useSyncExternalStore(store.subscribe, () => selector(store.getState()));
};
