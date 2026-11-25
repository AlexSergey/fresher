import type { JSX } from 'react';

import { ProgressBar } from '../../design/primitives/progress-bar/progress-bar';
import { t } from '../../i18n';
import { useCore } from '../../state/store';
import styles from './apps-screen.module.css';

// "Measuring your apps": how many bundles of how many the core measured so far, with its bar, while it tells; the line
// alone before it tells and once every bundle is measured (the list comes next).
export const AppsMeasuring = (): JSX.Element => {
  const measuring = useCore((state) => state.appsMeasuring);

  if (measuring === null || measuring.done >= measuring.total) {
    return <p className={styles.state}>{t('ui.screen.apps.measuring')}</p>;
  }

  const said = t('ui.screen.apps.measuring-count', {
    done: { kind: 'count', value: measuring.done },
    total: { kind: 'count', value: measuring.total },
  });

  return (
    <div className={styles.state}>
      <p className={styles.measuring}>{said}</p>
      <div className={styles.bar}>
        <ProgressBar label={said} value={(measuring.done / measuring.total) * 100} />
      </div>
    </div>
  );
};
