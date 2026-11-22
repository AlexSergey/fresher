import { type JSX, useState } from 'react';

import type { UnusedFor } from '../../ipc/bindings.generated';

import { PageHeader } from '../../design/layout/page-header/page-header';
import { Surface } from '../../design/layout/surface/surface';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { Button } from '../../design/primitives/button/button';
import { SegmentedControl } from '../../design/primitives/segmented-control/segmented-control';
import { t, tText } from '../../i18n';
import { AppRow } from '../../product/app-row/app-row';
import { useNavigation } from '../../state/navigation';
import { useCore } from '../../state/store';
import { CoverageNotices } from '../permission/coverage-notices';
import { useScanActions } from '../use-scan-actions';
import styles from './apps-screen.module.css';
import { useApps } from './use-apps';

type Filter = 'any' | UnusedFor;

const unusedOf = (filter: Filter): null | UnusedFor => (filter === 'any' ? null : filter);

// "What can go?": the installed apps, largest first, as the core lists and describes them. The filter "Not used for"
// is the core's: it applies it and answers the list.
export const AppsScreen = (): JSX.Element => {
  const navigation = useNavigation();
  const { scan } = useScanActions();
  const [filter, setFilter] = useState<Filter>('any');
  const { refusal, view } = useApps(unusedOf(filter));
  const known = useCore((state) => state.overview !== null);
  const scanned = useCore((state) => state.overview?.lastScan != null);
  const scanning = useCore((state) => state.status.state === 'scanning');

  return (
    <>
      <Toolbar />
      <PageHeader title={t('ui.screen.apps.title')} />
      <div className={styles.content}>
        <CoverageNotices />
        {known && !scanned && !scanning ? (
          <div className={styles.state}>
            <p>{t('ui.screen.apps.no-scan')}</p>
            <Button onClick={scan} variant="primary">
              {t('ui.scan-hero.scan')}
            </Button>
          </div>
        ) : (
          <>
            <SegmentedControl<Filter>
              label={t('ui.screen.apps.filter.label')}
              onChange={setFilter}
              options={[
                { label: t('ui.screen.apps.filter.any'), value: 'any' },
                { label: t('ui.screen.apps.filter.three-months'), value: 'three-months' },
                { label: t('ui.screen.apps.filter.six-months'), value: 'six-months' },
                { label: t('ui.screen.apps.filter.a-year'), value: 'a-year' },
              ]}
              value={filter}
            />
            {refusal !== null ? (
              <p className={styles.state}>{tText(refusal.text)}</p>
            ) : view === null ? (
              <p className={styles.state}>{t('ui.screen.apps.measuring')}</p>
            ) : (
              <>
                {view.notice === null ? null : <p className={styles.note}>{tText(view.notice)}</p>}
                {view.apps.length === 0 ? (
                  <p className={styles.state}>{t('ui.screen.apps.empty')}</p>
                ) : (
                  <Surface elevation={1} padding="none" radius="lg">
                    <div aria-label={t('ui.screen.apps.list')} className={styles.list} role="list">
                      {view.apps.map((summary) => (
                        <AppRow
                          key={`${String(summary.app.state)}:${String(summary.app.index)}`}
                          onOpen={() => {
                            navigation.navigate({ app: summary.app, kind: 'app' });
                          }}
                          summary={summary}
                        />
                      ))}
                    </div>
                  </Surface>
                )}
              </>
            )}
          </>
        )}
      </div>
    </>
  );
};
