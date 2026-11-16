import type { Clock } from './clock';
import type { CoreStore } from './store';

import { failureOf, getOverview } from '../ipc';

// Keeps the store's overview current: when the state turns stale it asks `get_overview` at most once per `interval`
// (4 times a second at 250 ms), never two requests at once. `scan-finished` marks the state stale, so one more request
// always follows the end of a scan. Returns the function that stops it.
export const createOverviewRefresher = (
  store: CoreStore,
  { clock, interval }: { readonly clock: Clock; readonly interval: number },
): (() => void) => {
  let pending = false;
  let stopped = false;
  let askedAt = Number.NEGATIVE_INFINITY;
  let cancelTimer: (() => void) | null = null;

  const ask = (): void => {
    cancelTimer = null;
    pending = true;
    askedAt = clock.now();
    store.dispatch({ type: 'overview-requested' });
    void getOverview()
      .then(
        (overview) => {
          store.dispatch({ overview, type: 'overview-loaded' });
        },
        (error: unknown) => {
          store.dispatch({ failure: failureOf(error), type: 'command-failed' });
        },
      )
      .finally(() => {
        pending = false;
        check();
      });
  };

  function check(): void {
    if (stopped || !store.getState().stale || pending || cancelTimer !== null) {
      return;
    }

    const wait = askedAt + interval - clock.now();

    if (wait <= 0) {
      ask();
    } else {
      cancelTimer = clock.setTimeout(ask, wait);
    }
  }

  const unsubscribe = store.subscribe(check);
  check();

  return (): void => {
    stopped = true;
    unsubscribe();
    cancelTimer?.();
    cancelTimer = null;
  };
};
