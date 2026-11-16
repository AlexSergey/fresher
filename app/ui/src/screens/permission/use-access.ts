import { useEffect } from 'react';

import type { Access } from '../../ipc/bindings.generated';

import { requestAccess } from '../../state/connect';
import { useCore, useCoreStore } from '../../state/store';

// Whether the app has Full Disk Access: asked of the core once, then kept current by `access-changed` events. `null`
// until the core has answered.
export const useAccess = (): Access | null => {
  const store = useCoreStore();

  useEffect(() => {
    requestAccess(store);
  }, [store]);

  return useCore((state) => state.access?.fullDiskAccess ?? null);
};
