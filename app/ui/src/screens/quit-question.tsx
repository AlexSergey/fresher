import { type JSX, useRef } from 'react';

import { Dialog } from '../design/layout/dialog/dialog';
import { Button } from '../design/primitives/button/button';
import { t } from '../i18n';
import { quitAfter } from '../state/cleanup-actions';
import { useCore, useCoreStore } from '../state/store';
import styles from './app-frame.module.css';

// The window was closed during a cleanup: quit after the item in hand, or keep cleaning. Keep cleaning has the first
// focus, and Escape or the close button keep cleaning too: nothing changes unless the user chooses to quit.
export const QuitQuestion = (): JSX.Element | null => {
  const store = useCoreStore();
  const asked = useCore((state) => state.quitAsked);
  const keepRef = useRef<HTMLButtonElement>(null);

  const keep = (): void => {
    store.dispatch({ type: 'quit-answered' });
  };

  return (
    <Dialog
      closeLabel={t('ui.screen.frame.close')}
      footer={
        <>
          <Button onClick={keep} ref={keepRef} variant="secondary">
            {t('ui.quit.keep')}
          </Button>
          <Button
            onClick={() => {
              quitAfter(store);
            }}
            variant="primary"
          >
            {t('ui.quit.stop')}
          </Button>
        </>
      }
      initialFocus={keepRef}
      onClose={keep}
      open={asked}
      title={t('ui.quit.title')}
    >
      <p className={styles.quitText}>{t('ui.quit.text')}</p>
    </Dialog>
  );
};
