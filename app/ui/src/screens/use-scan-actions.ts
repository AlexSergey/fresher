import { useMemo } from 'react';

import { cancelScan, failureOf } from '../ipc';
import { requestScan } from '../state/connect';
import { useCoreStore } from '../state/store';

export type ScanActions = {
  // Asks the core for a scan; its events update the store, a refusal becomes the frame's error toast.
  readonly scan: () => void;
  // Asks the core to stop the running scan; it ends as partial, with what it found so far.
  readonly stop: () => void;
};

// The Scan and Stop buttons of every screen's toolbar.
export const useScanActions = (): ScanActions => {
  const store = useCoreStore();

  return useMemo(
    () => ({
      scan: (): void => {
        requestScan(store);
      },
      stop: (): void => {
        cancelScan().catch((error: unknown) => {
          store.dispatch({ failure: failureOf(error), type: 'command-failed' });
        });
      },
    }),
    [store],
  );
};
