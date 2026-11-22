import type { AppId, Method, UninstallChoice } from '../ipc/bindings.generated';
import type { CoreStore } from './store';

import {
  cancelCleanup,
  confirmCleanup,
  discardCleanup,
  failureOf,
  prepareCleanup,
  prepareUninstall,
  quitAfterCleanup,
  revealInTrash,
} from '../ipc';
import { confirmationOf } from './cleanup';
import { cleanupSelectionOf } from './selection';

// A failed cleanup command: a refusal is shown with the cleanup, with the core's text; an unreachable core is the
// frame's error, and nothing is shown of the cleanup.
const refused =
  (store: CoreStore) =>
  (error: unknown): void => {
    const failure = failureOf(error);

    if (failure.kind === 'refused') {
      store.dispatch({ refusal: failure.refusal, type: 'cleanup-refused' });
    } else {
      store.dispatch({ failure, type: 'command-failed' });
      store.dispatch({ type: 'cleanup-dismissed' });
    }
  };

const failed =
  (store: CoreStore) =>
  (error: unknown): void => {
    store.dispatch({ failure: failureOf(error), type: 'command-failed' });
  };

// Asks the core to check what is selected; its plan opens the confirmation. Nothing is removed.
export const prepare = (store: CoreStore): void => {
  store.dispatch({ origin: 'selection', type: 'cleanup-preparing' });
  prepareCleanup(cleanupSelectionOf(store.getState().selection)).then((plan) => {
    store.dispatch({ plan, type: 'cleanup-prepared' });
  }, refused(store));
};

// Asks the core for the plan that uninstalls `app` with `choice`; its plan opens the same confirmation. Nothing is
// removed, and the selection is not touched.
export const prepareUninstallPlan = (store: CoreStore, app: AppId, choice: UninstallChoice): void => {
  store.dispatch({ app, origin: 'uninstall', type: 'cleanup-preparing' });
  prepareUninstall(app, choice).then((plan) => {
    store.dispatch({ plan, type: 'cleanup-prepared' });
  }, refused(store));
};

// Sends back exactly what the confirmation showed, with the method the user chose.
export const confirm = (store: CoreStore, method: Method): void => {
  const { cleanup } = store.getState();

  if (cleanup.kind !== 'confirming') {
    return;
  }

  store.dispatch({ method, type: 'cleanup-confirming' });
  confirmCleanup(confirmationOf(cleanup.plan, method)).then((id) => {
    store.dispatch({ cleanup: id, type: 'cleanup-confirmed' });
  }, refused(store));
};

// The confirmation was cancelled: the core forgets its plan. A plan already gone is gone either way.
export const discard = (store: CoreStore): void => {
  const { cleanup } = store.getState();

  store.dispatch({ type: 'cleanup-dismissed' });

  if (cleanup.kind === 'confirming') {
    discardCleanup(cleanup.plan.plan).catch((error: unknown) => {
      if (failureOf(error).kind === 'unreachable') {
        failed(store)(error);
      }
    });
  }
};

// Stop, once: the cleanup ends after the item in hand.
export const stop = (store: CoreStore): void => {
  const { cleanup } = store.getState();

  if (cleanup.kind !== 'running' || cleanup.stopping) {
    return;
  }

  store.dispatch({ type: 'cleanup-stopping' });
  cancelCleanup().catch(failed(store));
};

// The quit question answered yes: the cleanup stops after the item in hand, then the app quits.
export const quitAfter = (store: CoreStore): void => {
  store.dispatch({ type: 'quit-answered' });
  quitAfterCleanup().catch(failed(store));
};

// Shows the first item the finished cleanup moved to the Trash; a refusal is the frame's toast with the core's text.
export const showInTrash = (store: CoreStore): void => {
  const { cleanup } = store.getState();

  if (cleanup.kind === 'finished' && cleanup.firstMoved !== null) {
    revealInTrash({ cleanup: cleanup.cleanup, item: cleanup.firstMoved }).catch(failed(store));
  }
};
