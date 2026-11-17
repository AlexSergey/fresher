import { useEffect, useState } from 'react';

import type { AppInfo } from '../ipc/bindings.generated';

import { getAppInfo } from '../ipc';

// What the core says about the app (its version and its build), `null` until (or unless) it tells.
export const useAppInfo = (): AppInfo | null => {
  const [info, setInfo] = useState<AppInfo | null>(null);

  useEffect(() => {
    let current = true;

    getAppInfo().then(
      (answer) => {
        if (current) {
          setInfo(answer);
        }
      },
      () => undefined,
    );

    return (): void => {
      current = false;
    };
  }, []);

  return info;
};
