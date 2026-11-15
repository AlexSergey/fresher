import { type JSX, useEffect, useState } from 'react';

import { getAppVersion } from './ipc';

export const App = (): JSX.Element => {
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    getAppVersion()
      .then(setVersion)
      .catch(() => {
        setVersion(null);
      });
  }, []);

  return (
    <main className="app">
      <h1 className="app-title">Mac Fresher</h1>
      <p className="app-version">{version === null ? 'Connecting to the core…' : `Version ${version}`}</p>
    </main>
  );
};
