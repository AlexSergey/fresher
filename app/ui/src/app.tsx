import { type JSX, useEffect, useState } from 'react';

import type { Failure } from './ipc';
import type { CategoryGroup, CoreState } from './state/reduce';

import { t, tText } from './i18n';
import { formatSize } from './i18n/format-size';
import { getAppVersion } from './ipc';
import { connectCore, requestScan } from './state/connect';
import { CoreContext, type CoreStore, useCore, useCoreStore } from './state/store';

// A text of the core's dictionary with no arguments, by the key its convention gives.
const textOf = (key: string): string => tText({ args: {}, key });

const ScanState = ({ outcome, status }: Pick<CoreState, 'outcome' | 'status'>): JSX.Element => {
  if (status.state === 'scanning') {
    return <p className="app-version">{t('ui.scan.running')}</p>;
  }

  if (outcome === null) {
    return <p className="app-version">{t('ui.scan.ready')}</p>;
  }

  return (
    <p className="app-version">
      {textOf(`core.scan.outcome.${outcome.kind}`)}
      {outcome.kind === 'failed' ? ` ${tText(outcome.reason)}` : null}
    </p>
  );
};

const Category = ({ group }: { readonly group: CategoryGroup }): JSX.Element => (
  <section aria-label={textOf(`${group.category}.name`)}>
    <h2>{textOf(`${group.category}.name`)}</h2>
    <ul>
      {group.findings.map((finding) => (
        <li key={finding.id}>
          {tText(finding.title)} · {formatSize(finding.sizes.allocated, finding.sizes.completeness)} ·{' '}
          {textOf(`core.danger.${finding.danger}.name`)} · {tText(finding.reason)}
        </li>
      ))}
    </ul>
  </section>
);

// The core's text for a refusal, as it sent it; the UI's own text when the core could not be reached.
const failureText = (failure: Failure): string =>
  failure.kind === 'refused' ? tText(failure.refusal.text) : t('ui.core.unreachable');

const Screen = (): JSX.Element => {
  const store = useCoreStore();
  const [version, setVersion] = useState<null | string>(null);
  // The texts come with the dictionary: the screen renders again once it is loaded.
  useCore((state) => state.dictionary);
  const status = useCore((state) => state.status);
  const outcome = useCore((state) => state.outcome);
  const groups = useCore((state) => state.groups);
  const failure = useCore((state) => state.failure);

  useEffect(() => {
    connectCore(store);
    getAppVersion()
      .then(setVersion)
      .catch(() => {
        setVersion(null);
      });
  }, [store]);

  return (
    <main className="app">
      <h1 className="app-title">{t('ui.app.title')}</h1>
      <p className="app-version">
        {version === null ? t('ui.app.connecting') : t('ui.app.version', { version: { kind: 'text', value: version } })}
      </p>
      <p>
        <button
          onClick={() => {
            requestScan(store);
          }}
          type="button"
        >
          {t('ui.scan.start')}
        </button>
      </p>
      <ScanState outcome={outcome} status={status} />
      {failure === null ? null : <p role="alert">{failureText(failure)}</p>}
      {groups.map((group) => (
        <Category group={group} key={group.category} />
      ))}
    </main>
  );
};

export const App = ({ store }: { readonly store: CoreStore }): JSX.Element => (
  <CoreContext value={store}>
    <Screen />
  </CoreContext>
);
