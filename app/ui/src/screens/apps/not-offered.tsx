import type { JSX } from 'react';

import type { AppDetail, AppId, AppOffer } from '../../ipc/bindings.generated';

import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { failureOf, revealApp } from '../../ipc';
import { useCoreStore } from '../../state/store';
import styles from './app-screen.module.css';

type NotOfferedProps = {
  readonly app: AppId;
  readonly detail: AppDetail;
  readonly offer: Extract<AppOffer, { readonly kind: 'not-offered' }>;
};

// Why an app cannot be uninstalled from here, in the core's words, and the one thing the user can do instead: show it in
// Finder when it needs an administrator, or the manager's command, as text to copy, when a manager installed it.
export const NotOffered = ({ app, detail, offer }: NotOfferedProps): JSX.Element => {
  const store = useCoreStore();

  return (
    <section aria-label={t('ui.screen.app.not-offered')} className={styles.part}>
      <p className={styles.reason}>{tText(offer.text)}</p>
      {offer.reason === 'needs-administrator' ? (
        <Button
          onClick={() => {
            revealApp(app).catch((error: unknown) => {
              store.dispatch({ failure: failureOf(error), type: 'command-failed' });
            });
          }}
        >
          {t('ui.screen.app.show-in-finder')}
        </Button>
      ) : null}
      {offer.reason === 'installed-by' && detail.command !== null ? (
        <div className={styles.command}>
          <p>{t('ui.screen.app.command')}</p>
          <code className={styles.code}>{detail.command}</code>
        </div>
      ) : null}
    </section>
  );
};
