import type { JSX } from 'react';

import { EmptyState } from '../../design/feedback/empty-state/empty-state';
import { Notice } from '../../design/feedback/notice/notice';
import { History } from '../../design/icons/icons';
import { PageHeader } from '../../design/layout/page-header/page-header';
import { Surface } from '../../design/layout/surface/surface';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { CleanupRow } from '../../product/cleanup-row/cleanup-row';
import { useNavigation } from '../../state/navigation';
import styles from './cleanups-screen.module.css';
import { useCleanups } from './use-cleanups';

// "What did I clean, and can I get it back?": every cleanup, newest first, under what still waits in the Trash.
export const CleanupsScreen = (): JSX.Element => {
  const { navigate } = useNavigation();
  const { more, page } = useCleanups();

  return (
    <>
      <Toolbar />
      <PageHeader subtitle={t('ui.screen.cleanups.subtitle')} title={t('ui.screen.frame.nav.cleanups')} />
      <div className={styles.content}>
        {page?.blocked == null ? null : (
          <Notice title={t('ui.screen.cleanups.read-only')} tone="warning">
            {tText({ args: {}, key: `core.cleanup.blocked.${page.blocked}` })}
          </Notice>
        )}
        {page === null ? null : page.total === 0 ? (
          <EmptyState
            icon={History}
            text={t('ui.screen.cleanups.empty.text')}
            title={t('ui.screen.cleanups.empty.title')}
          />
        ) : (
          <>
            <div className={styles.waiting}>
              <p className={styles.waitingSize}>
                {t('ui.screen.cleanups.waiting', { size: { kind: 'size', value: page.waiting } })}
              </p>
              <p className={styles.note}>{t('ui.screen.cleanups.waiting-note')}</p>
            </div>
            <Surface elevation={1} padding="none" radius="lg">
              <ul aria-label={t('ui.screen.cleanups.list')} className={styles.list}>
                {page.cleanups.map((summary) => (
                  <CleanupRow
                    key={summary.cleanup}
                    onOpen={() => {
                      navigate({ cleanup: summary.cleanup, kind: 'cleanup' });
                    }}
                    summary={summary}
                  />
                ))}
              </ul>
            </Surface>
            {page.cleanups.length < page.total ? (
              <div className={styles.more}>
                <Button onClick={more}>{t('ui.screen.cleanups.more')}</Button>
              </div>
            ) : null}
          </>
        )}
      </div>
    </>
  );
};
