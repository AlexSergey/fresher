import { useCallback, useEffect, useState } from 'react';

import type { SettingsView, SettingValue } from '../../ipc/bindings.generated';

import { tText } from '../../i18n';
import { failureOf, updateSetting } from '../../ipc';
import { requestSettings } from '../../state/connect';
import { useCore, useCoreStore } from '../../state/store';

export type Settings = {
  // Sends a new value; the row shows it once the core's `setting-changed` event arrives.
  readonly change: (key: string, value: SettingValue) => void;
  // The core's text of the last refused change, per setting.
  readonly refusals: Readonly<Record<string, string>>;
  // `null` until the core has answered.
  readonly view: null | SettingsView;
};

// The settings as the core keeps them: asked for once, kept current by the store, changed through the core.
export const useSettings = (): Settings => {
  const store = useCoreStore();
  const view = useCore((state) => state.settings);
  const [refusals, setRefusals] = useState<Readonly<Record<string, string>>>({});

  useEffect(() => {
    requestSettings(store);
  }, [store]);

  const change = useCallback(
    (key: string, value: SettingValue): void => {
      setRefusals((current) => Object.fromEntries(Object.entries(current).filter(([refused]) => refused !== key)));
      updateSetting({ key, value }).catch((error: unknown) => {
        const failure = failureOf(error);

        if (failure.kind === 'refused') {
          setRefusals((current) => ({ ...current, [key]: tText(failure.refusal.text) }));
        } else {
          store.dispatch({ failure, type: 'command-failed' });
        }
      });
    },
    [store],
  );

  return { change, refusals, view };
};
