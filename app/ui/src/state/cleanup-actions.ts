import type { AgreementView, AppId, Method, UninstallChoice } from '../ipc/bindings.generated';
import type { CoreStore } from './store';

import {
  acceptAgreement,
  cancelCleanup,
  confirmCleanup,
  discardCleanup,
  failureOf,
  getAgreement,
  isAgreementChanged,
  prepareCleanup,
  prepareUninstall,
  quitAfterCleanup,
  revealInTrash,
} from '../ipc';
import { type AgreementNext, isAccepted } from './agreement';
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

const startPrepare = (store: CoreStore): void => {
  store.dispatch({ origin: 'selection', type: 'cleanup-preparing' });
  prepareCleanup(cleanupSelectionOf(store.getState().selection)).then((plan) => {
    store.dispatch({ plan, type: 'cleanup-prepared' });
  }, refused(store));
};

const startUninstall = (store: CoreStore, app: AppId, choice: UninstallChoice): void => {
  store.dispatch({ app, origin: 'uninstall', type: 'cleanup-preparing' });
  prepareUninstall(app, choice).then((plan) => {
    store.dispatch({ plan, type: 'cleanup-prepared' });
  }, refused(store));
};

const run = (store: CoreStore, next: AgreementNext): void => {
  if (next.kind === 'prepare') {
    startPrepare(store);
  } else {
    startUninstall(store, next.app, next.choice);
  }
};

// Goes on with `next` when the agreement is accepted; otherwise the frame asks for it and nothing else happens. The
// view is asked for first when the store has none. An unreachable core is the frame's error.
const afterAgreement = (store: CoreStore, next: AgreementNext): void => {
  const { view } = store.getState().agreement;
  const decide = (known: AgreementView): void => {
    if (isAccepted(known)) {
      run(store, next);
    } else {
      store.dispatch({ next, type: 'agreement-asked' });
    }
  };

  if (view !== null) {
    decide(view);

    return;
  }
  getAgreement().then((loaded) => {
    store.dispatch({ type: 'agreement-loaded', view: loaded });
    decide(loaded);
  }, failed(store));
};

// Asks the core to check what is selected; its plan opens the confirmation. Nothing is removed. Before the agreement
// is accepted it asks for the agreement instead.
export const prepare = (store: CoreStore): void => {
  afterAgreement(store, { kind: 'prepare' });
};

// Asks the core for the plan that uninstalls `app` with `choice`; its plan opens the same confirmation. Nothing is
// removed, and the selection is not touched. Before the agreement is accepted it asks for the agreement instead.
export const prepareUninstallPlan = (store: CoreStore, app: AppId, choice: UninstallChoice): void => {
  afterAgreement(store, { app, choice, kind: 'uninstall' });
};

// The stores whose acceptance is on its way to the core: a second click before the answer accepts nothing more.
const accepting = new WeakSet<CoreStore>();

// "I understand": the core records the acceptance, then the step that asked goes on. A refusal because the text
// changed loads the new one and keeps the sheet open; any other refusal is the frame's toast and keeps it open too.
// While one acceptance is on its way, another click does nothing, so the step is prepared once.
export const acceptAgreementAndGoOn = (store: CoreStore): void => {
  const { asking, view } = store.getState().agreement;

  if (asking === null || view === null || accepting.has(store)) {
    return;
  }
  accepting.add(store);
  acceptAgreement(view.version)
    .finally(() => {
      accepting.delete(store);
    })
    .then(
      (accepted) => {
        store.dispatch({ type: 'agreement-loaded', view: accepted });
        store.dispatch({ type: 'agreement-dismissed' });
        run(store, asking);
      },
      (error: unknown) => {
        const failure = failureOf(error);

        if (failure.kind === 'refused' && isAgreementChanged(failure.refusal)) {
          getAgreement().then((current) => {
            store.dispatch({ type: 'agreement-loaded', view: current });
          }, failed(store));
        } else {
          store.dispatch({ failure, type: 'command-failed' });
          if (failure.kind === 'unreachable') {
            store.dispatch({ type: 'agreement-dismissed' });
          }
        }
      },
    );
};

// "Not now": the sheet closes and nothing is prepared.
export const declineAgreement = (store: CoreStore): void => {
  store.dispatch({ type: 'agreement-dismissed' });
};

// Sends back exactly what the confirmation showed, with the method the user chose and whether they ticked that a
// High item may be the only copy.
export const confirm = (store: CoreStore, method: Method, highRiskUnderstood: boolean): void => {
  const { cleanup } = store.getState();

  if (cleanup.kind !== 'confirming') {
    return;
  }

  store.dispatch({ method, type: 'cleanup-confirming' });
  confirmCleanup(confirmationOf(cleanup.plan, method, highRiskUnderstood)).then((id) => {
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
