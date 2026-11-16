import { useEffect } from 'react';

import { getSettings } from '../ipc';
import { useCore, useCoreStore } from '../state/store';

// The `core.motion` setting on the page: `reduced` sets `data-motion="reduced"` on <html>, which the motion helpers
// read; `system` (the default) leaves it off, so macOS's "Reduce motion" decides. The settings are asked for once, at start.
export const useMotionSetting = (): void => {
  const store = useCoreStore();
  const motion = useCore((state) => state.settings?.values['core.motion']);
  const reduced = motion?.kind === 'choice' && motion.value === 'reduced';

  useEffect(() => {
    // Asked quietly: without an answer the system's motion setting stays in charge, and the settings screen asks again.
    if (store.getState().settings === null) {
      getSettings().then(
        (settings) => {
          if (store.getState().settings === null) {
            store.dispatch({ settings, type: 'settings-loaded' });
          }
        },
        () => undefined,
      );
    }
  }, [store]);

  useEffect(() => {
    const root = document.documentElement;

    if (reduced) {
      root.dataset.motion = 'reduced';
    } else if (root.dataset.motion === 'reduced') {
      delete root.dataset.motion;
    }
  }, [reduced]);
};
