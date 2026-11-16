import { type JSX, useEffect, useRef, useState } from 'react';

import type { OwnData } from '../../ipc/bindings.generated';

import { Dialog } from '../../design/layout/dialog/dialog';
import { Surface } from '../../design/layout/surface/surface';
import { Button } from '../../design/primitives/button/button';
import { t } from '../../i18n';
import { failureOf, getOwnData, resetSettings } from '../../ipc';
import { useCoreStore } from '../../state/store';
import styles from './settings-screen.module.css';

type OwnDataGroupProps = {
  readonly headingId: string;
  readonly readOnly: boolean;
};

// Fresher's own data: what its folder takes against its budget, and "Reset all settings…", confirmed in a dialog.
export const OwnDataGroup = ({ headingId, readOnly }: OwnDataGroupProps): JSX.Element => {
  const store = useCoreStore();
  const own = useOwnData();
  const [asking, setAsking] = useState(false);
  const keepRef = useRef<HTMLButtonElement>(null);

  const reset = (): void => {
    setAsking(false);
    resetSettings().catch((error: unknown) => {
      store.dispatch({ failure: failureOf(error), type: 'command-failed' });
    });
  };

  return (
    <section aria-labelledby={headingId} className={styles.group}>
      <h2 className={styles.heading} id={headingId}>
        {t('ui.screen.settings.group.own-data')}
      </h2>
      <Surface elevation={1} padding="none">
        <div className={styles.rows}>
          {own === null ? null : (
            <p className={styles.line}>
              {t('ui.screen.settings.own-data.size', {
                budget: { kind: 'size', value: own.budget },
                size: { kind: 'size', value: own.total },
              })}
            </p>
          )}
          <div className={styles.line}>
            <span className={styles.lineText}>{t('ui.screen.settings.own-data.reset.text')}</span>
            <Button
              disabled={readOnly}
              onClick={() => {
                setAsking(true);
              }}
              size="small"
              variant="secondary"
            >
              {t('ui.screen.settings.own-data.reset.action')}
            </Button>
          </div>
        </div>
      </Surface>
      <Dialog
        closeLabel={t('ui.screen.frame.close')}
        footer={
          <>
            <Button
              onClick={() => {
                setAsking(false);
              }}
              ref={keepRef}
              variant="secondary"
            >
              {t('ui.screen.settings.own-data.reset.keep')}
            </Button>
            <Button onClick={reset} variant="primary">
              {t('ui.screen.settings.own-data.reset.confirm')}
            </Button>
          </>
        }
        initialFocus={keepRef}
        onClose={() => {
          setAsking(false);
        }}
        open={asking}
        title={t('ui.screen.settings.own-data.reset.title')}
      >
        <p className={styles.groupText}>{t('ui.screen.settings.own-data.reset.question')}</p>
      </Dialog>
    </section>
  );
};

// The size of the data folder, `null` until (or unless) the core tells it.
const useOwnData = (): null | OwnData => {
  const [own, setOwn] = useState<null | OwnData>(null);

  useEffect(() => {
    let current = true;

    getOwnData().then(
      (answer) => {
        if (current) {
          setOwn(answer);
        }
      },
      () => undefined,
    );

    return (): void => {
      current = false;
    };
  }, []);

  return own;
};
