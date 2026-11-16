import { useEffect, useState } from 'react';

import { getAppVersion } from '../../ipc';

// The app's version, `null` until (or unless) the core tells it.
export const useAppVersion = (): null | string => {
  const [version, setVersion] = useState<null | string>(null);

  useEffect(() => {
    let current = true;

    getAppVersion().then(
      (answer) => {
        if (current) {
          setVersion(answer);
        }
      },
      () => undefined,
    );

    return (): void => {
      current = false;
    };
  }, []);

  return version;
};
