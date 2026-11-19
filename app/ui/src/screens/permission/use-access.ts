import { useEffect } from 'react';

import type { AccessStatus, Coverage } from '../../ipc/bindings.generated';

import { requestAccess } from '../../state/connect';
import { useCore, useCoreStore } from '../../state/store';

// Every capability and its state, asked of the core once and kept current by `access-changed` events.
export const useAccessStatus = (): AccessStatus | null => {
  const store = useCoreStore();

  useEffect(() => {
    requestAccess(store);
  }, [store]);

  return useCore((state) => state.access);
};

// What the result on screen did not see; `null` while a scan runs or when no scan is known.
export const useCoverage = (): Coverage | null => useCore((state) => state.coverage);
